use {
    bevy::color::Color,
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    serde::{Deserialize, Serialize},
    std::collections::VecDeque,
};

/// Port to run the WebSocket server on.
pub const WEB_SOCKET_PORT: u16 = 25570;

/// Port to run the WebTransport server.
pub const WEB_TRANSPORT_PORT: u16 = 25571;

/// How many units a player may move in a single second.
const MOVE_SPEED: f32 = 250.0;

/// How many times per second we will replicate entity components.
pub const TICK_RATE: u16 = 20;

/// Maximum number of fallback inputs synthesized while filling gaps in the
/// sequence of client-provided samples.
const MAX_INPUT_GAP_FILL: u32 = 8;

/// Maximum number of samples from a single payload that will be considered by
/// the server. Older samples beyond this limit are ignored to keep the per-tick
/// processing cost bounded.
const MAX_INPUT_SAMPLES: usize = 3;

/// Sets up replication and basic game systems.
pub struct MoveBoxPlugin;

/// Whether the game is currently being simulated or not.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    /// Game is not being simulated.
    #[default]
    None,
    /// Game is being simulated.
    Playing,
}

impl Plugin for MoveBoxPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .replicate::<Player>()
            .replicate::<PlayerPosition>()
            .replicate::<PlayerColor>()
            .add_client_message::<PlayerInputPayload>(Channel::Unreliable)
            .add_systems(
                FixedUpdate,
                (recv_input, apply_movement)
                    .chain()
                    .run_if(in_state(ClientState::Disconnected)),
            );
    }
}

/// Marker component for a player in the game.
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
#[require(DespawnOnExit::<GameState>(GameState::Playing))]
pub struct Player;

/// Player's box position.
#[derive(Debug, Clone, Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct PlayerPosition(pub Vec2);

/// Player's box color.
#[derive(Debug, Clone, Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct PlayerColor(pub Color);

/// Player's inputs that they send to control their box.
#[derive(Debug, Clone, Default, Message, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Lateral movement vector.
    ///
    /// The client has full control over this field, and may send an
    /// unnormalized vector! Authorities must ensure that they normalize or
    /// zero this vector before using it for movement updates.
    pub movement: Vec2,
}

/// Ein einzelnes Input-Sample, das vom Client gesendet wird.
#[derive(Clone, Serialize, Deserialize)]
pub struct InputSample {
    /// Sequenznummer, monoton steigend pro Client-Verbindung.
    pub sequence: u32,
    /// Zeitstempel auf Client-Seite (z. B. Sekunden seit Session-Start).
    pub sent_at: f64,
    /// Nutzlast des Samples.
    pub input: PlayerInput,
}

/// Datagramm, das mehrere Samples enthält.
#[derive(Message, Clone, Serialize, Deserialize)]
pub struct PlayerInputPayload {
    /// Neueste Probe zuerst. Der Server verarbeitet sie rückwärts, um die
    /// chronologische Reihenfolge beizubehalten.
    pub samples: Vec<InputSample>,
}

/// Server-side player input buffer that keeps track of all inputs received
/// since the previous simulation tick.
#[derive(Debug, Default, Component)]
pub struct PlayerInputState {
    current: PlayerInput,
    pending: VecDeque<TimedInput>,
    last_simulated_at: f64,
    next_expected_seq: Option<u32>,
}

#[derive(Debug, Clone)]
struct TimedInput {
    sequence: u32,
    received_at: f64,
    sent_at: f64,
    input: PlayerInput,
}

impl PlayerInputState {
    fn mark_simulated(&mut self, now: f64) {
        self.last_simulated_at = now;
    }

    fn expect_next(&mut self, sequence: u32) {
        self.next_expected_seq = Some(sequence);
    }

    fn next_expected(&self) -> Option<u32> {
        self.next_expected_seq
    }

    fn update_expected_after(&mut self, sequence: u32) {
        self.next_expected_seq = Some(sequence.wrapping_add(1));
    }

    fn enqueue_sample(&mut self, sample: TimedInput) {
        self.pending.push_back(sample);
    }

    fn fill_gap_until(&mut self, mut expected: u32, sequence: u32, now: f64) -> u32 {
        let mut filled = 0;
        while expected != sequence && filled < MAX_INPUT_GAP_FILL {
            self.pending.push_back(TimedInput {
                sequence: expected,
                received_at: now,
                sent_at: now,
                input: self.current.clone(),
            });
            expected = expected.wrapping_add(1);
            filled += 1;
        }

        if expected != sequence {
            let remaining = sequence.wrapping_sub(expected);
            warn!(
                filled,
                remaining,
                expected,
                sequence,
                max_fill = MAX_INPUT_GAP_FILL,
                "input gap fill limit exceeded; dropping remaining client samples"
            );
        }

        self.next_expected_seq = Some(expected);
        expected
    }

    #[cfg(test)]
    fn pending_len(&self) -> usize {
        self.pending.len()
    }

    #[cfg(test)]
    fn current_movement(&self) -> Vec2 {
        self.current.movement
    }

    #[cfg(test)]
    fn set_current(&mut self, input: PlayerInput) {
        self.current = input;
    }

    #[cfg(test)]
    fn queue_input(&mut self, sequence: u32, timestamp: f64, input: PlayerInput) {
        self.pending.push_back(TimedInput {
            sequence,
            received_at: timestamp,
            sent_at: timestamp,
            input,
        });
    }
}

fn recv_input(
    mut inputs: MessageReader<FromClient<PlayerInputPayload>>,
    time: Res<Time>,
    mut players: Query<&mut PlayerInputState>,
) {
    let now = time.elapsed_secs_f64();

    for FromClient { client_id, message } in inputs.read() {
        let ClientId::Client(entity) = client_id else {
            continue;
        };
        let Ok(mut state) = players.get_mut(*entity) else {
            continue;
        };

        for sample in message.samples.iter().take(MAX_INPUT_SAMPLES).rev() {
            let sequence = sample.sequence;

            if state.next_expected().is_none() {
                state.expect_next(sequence);
            }

            let expected = state.next_expected().unwrap();
            if sequence < expected {
                continue;
            }

            let expected_after_fill = if sequence > expected {
                state.fill_gap_until(expected, sequence, now)
            } else {
                expected
            };

            if expected_after_fill != sequence {
                continue;
            }

            state.enqueue_sample(TimedInput {
                sequence,
                received_at: now,
                sent_at: sample.sent_at,
                input: sample.input.clone(),
            });

            state.update_expected_after(sequence);
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut players: Query<(&mut PlayerInputState, &mut PlayerPosition)>,
) {
    let now = time.elapsed_secs_f64();
    let delta_time = time.delta_secs_f64();

    for (mut state, mut position) in &mut players {
        integrate_player_inputs(&mut state, &mut position, now, delta_time);
    }
}

fn integrate_player_inputs(
    state: &mut PlayerInputState,
    position: &mut PlayerPosition,
    now: f64,
    tick_dt: f64,
) {
    if tick_dt <= f64::EPSILON {
        return;
    }

    let mut last_time = state.last_simulated_at.max(now - tick_dt);

    while let Some(next) = state.pending.front().cloned() {
        if next.received_at > now {
            break;
        }

        let segment_dt = (next.received_at - last_time).max(0.0) as f32;
        apply_single_input(position, &state.current, segment_dt);

        state.current = next.input;
        state.pending.pop_front();
        last_time = next.received_at;
    }

    let remaining_dt = (now - last_time).max(0.0) as f32;
    apply_single_input(position, &state.current, remaining_dt);

    state.mark_simulated(now);
}

fn apply_single_input(position: &mut PlayerPosition, input: &PlayerInput, delta_time: f32) {
    if delta_time <= f32::EPSILON {
        return;
    }

    if let Some(direction) = input.movement.try_normalize() {
        **position += direction * delta_time * MOVE_SPEED;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(x: f32, y: f32) -> PlayerInput {
        PlayerInput {
            movement: Vec2::new(x, y),
        }
    }

    #[test]
    fn integrates_pending_inputs_and_updates_state() {
        let mut state = PlayerInputState::default();
        let mut position = PlayerPosition(Vec2::ZERO);

        state.queue_input(0, 1.0 / 3.0, make_input(1.0, 0.0));
        state.queue_input(1, 2.0 / 3.0, make_input(0.0, 1.0));

        integrate_player_inputs(&mut state, &mut position, 1.0, 1.0);

        let expected_delta = MOVE_SPEED / 3.0;
        assert!((position.x - expected_delta).abs() < 1e-5);
        assert!((position.y - expected_delta).abs() < 1e-5);
        assert_eq!(state.pending_len(), 0);

        let current = state.current_movement();
        assert!(current.x.abs() < 1e-5);
        assert!((current.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn uses_last_known_input_when_no_new_samples() {
        let mut state = PlayerInputState::default();
        state.set_current(make_input(0.0, 2.0));

        let mut position = PlayerPosition(Vec2::ZERO);
        integrate_player_inputs(&mut state, &mut position, 0.5, 0.5);

        let expected = MOVE_SPEED * 0.5;
        assert!(position.x.abs() < 1e-5);
        assert!((position.y - expected).abs() < 1e-5);
        assert_eq!(state.pending_len(), 0);

        let current = state.current_movement();
        assert!((current.y - 2.0).abs() < 1e-5);
    }

    #[test]
    fn fills_sequence_gaps_up_to_limit() {
        let mut state = PlayerInputState::default();
        let time_now = 1.0;

        state.expect_next(0);
        let filled_target = state.fill_gap_until(0, 3, time_now);

        assert_eq!(filled_target, 3);
        assert_eq!(state.pending_len(), MAX_INPUT_GAP_FILL.min(3) as usize);
        assert_eq!(state.next_expected(), Some(3));
    }
}
