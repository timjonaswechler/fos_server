//! Demo app where clients can connect to a server and control a box with the
//! arrow keys.
//!
//! Box positions are synced between clients and servers using [`bevy_replicon`]
//! with the [`aeronet_replicon`] backend.
//!
//! This example currently runs the following IO layers at once:
//! - [`aeronet_websocket`] on port `25570`
//! - [`aeronet_webtransport`] on port `25571`
//!
//! Based on <https://github.com/projectharmonia/bevy_replicon_renet/blob/master/examples/simple_box.rs>.
//!
//! # Usage
//!
//! ## Server
//!
//! ```sh
//! cargo run --bin move_box_server
//! ```
//!
//! ## Client
//!
//! Native:
//!
//! ```sh
//! cargo run --bin move_box_client
//! ```
//!
//! WASM:
//!
//! ```sh
//! cargo install wasm-server-runner
//! cargo run --bin move_box_client --target wasm32-unknown-unknown
//! ```
//!
//! You must use a Chromium browser to try the demo:
//! - Currently, the WASM client demo doesn't run on Firefox, due to an issue
//!   with how `xwt` handles getting the reader for the incoming datagram
//!   stream. This results in the backend task erroring whenever a connection
//!   starts.
//! - WebTransport is not supported on Safari.
//!
//! Eventually, when Firefox is supported but you still have problems running
//! the client under Firefox (especially LibreWolf), check:
//! - `privacy.resistFingerprinting` is disabled, or Enhanced Tracking
//!   Protection is disabled for the website (see [winit #3345])
//! - `webgl.disabled` is set to `false`, so that Bevy can use the GPU
//!
//! [winit #3345]: https://github.com/rust-windowing/winit/issues/3345
//!
//! ## Connecting
//!
//! ### WebTransport
//!
//! The server binds to `0.0.0.0` by default. To connect to the server from the
//! client, you must specify an HTTPS address. For a local server, this will be
//! `https://[::1]:PORT`.
//!
//! By default, you will not be able to connect to the server, because it uses a
//! self-signed certificate which your client (native or browser) will treat as
//! invalid. To get around this, you must manually provide SHA-256 digest of the
//! certificate's DER as a base 64 string.
//!
//! When starting the server, it outputs the *certificate hash* as a base 64
//! string (it also outputs the *SPKI fingerprint*, which is different and is
//! not necessary here). Copy this string and enter it into the "certificate
//! hash" field of the client before connecting. The client will then ignore
//! certificate validation errors for this specific certificate, and allow a
//! connection to be established.
//!
//! In the browser, egui may not let you paste in the hash. You can get around
//! this by:
//! 1. clicking into the certificate hash text box
//! 2. clicking outside of the bevy window (i.e. into the white space)
//! 3. pressing Ctrl+V
//!
//! In the native client, if you leave the certificate hash field blank, the
//! client will simply not validate certificates. **This is dangerous** and
//! should not be done in your actual app, which is why it's locked behind the
//! `dangerous-configuration` flag, but is done for convenience in this example.
//!
//! ### WebSocket
//!
//! The server binds to `0.0.0.0` without encryption. You will need to connect
//! using a URL which uses the `ws` protocol (not `wss`).
//!
//! [`aeronet_webtransport`]: https://docs.rs/aeronet_webtransport
//! [`aeronet_websocket`]: https://docs.rs/aeronet_websocket
//! [`bevy_replicon`]: https://docs.rs/bevy_replicon
//! [`aeronet_replicon`]: https://docs.rs/aeronet_replicon

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
            .add_client_message::<PlayerInput>(Channel::Unreliable)
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

/// Server-side player input buffer that keeps track of all inputs received
/// since the previous simulation tick.
#[derive(Debug, Default, Component)]
pub struct PlayerInputState {
    current: PlayerInput,
    pending: VecDeque<TimedInput>,
    last_simulated_at: f64,
}

#[derive(Debug, Clone)]
struct TimedInput {
    received_at: f64,
    input: PlayerInput,
}

impl PlayerInputState {
    fn queue_input(&mut self, input: PlayerInput, received_at: f64) {
        self.pending.push_back(TimedInput { received_at, input });
    }

    fn mark_simulated(&mut self, now: f64) {
        self.last_simulated_at = now;
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
}

fn recv_input(
    mut inputs: MessageReader<FromClient<PlayerInput>>,
    time: Res<Time>,
    mut players: Query<&mut PlayerInputState>,
) {
    let now = time.elapsed_secs_f64();

    for &FromClient {
        client_id,
        message: ref new_input,
    } in inputs.read()
    {
        let ClientId::Client(client_entity) = client_id else {
            continue;
        };

        let Ok(mut state) = players.get_mut(client_entity) else {
            continue;
        };

        state.queue_input(new_input.clone(), now);
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
    // Startpunkt für die Simulation: entweder das Ende des letzten Ticks oder "jetzt - tick_dt"
    let mut last_time = state.last_simulated_at.max(now - tick_dt);

    // (1) Alle neuen Eingaben chronologisch durchlaufen
    while let Some(next) = state.pending.front().cloned() {
        if next.received_at > now {
            break;
        }

        let segment_dt = (next.received_at - last_time).max(0.0) as f32;
        apply_single_input(position, &state.current, segment_dt);

        // neue Eingabe aktivieren und weitermachen
        state.current = next.input;
        state.pending.pop_front();
        last_time = next.received_at;
    }

    // (2) Rest der Zeitspanne bis zum aktuellen Tick-Ende mit dem zuletzt aktiven Input integrieren
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

        state.queue_input(make_input(1.0, 0.0), 1.0 / 3.0);
        state.queue_input(make_input(0.0, 1.0), 2.0 / 3.0);

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
}
