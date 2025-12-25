pub(super) mod client;
pub(super) mod server;
pub(super) mod singleplayer;

use {
    crate::status_management::SingleplayerShutdownStep,
    bevy::prelude::*,
    client::{ClientStatus, ClientStatusPlugin},
    server::{ServerStatusPlugin, ServerVisibility},
    singleplayer::{SingleplayerStatus, SingleplayerStatusPlugin},
};

pub(super) struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClientStatusPlugin,
            ServerStatusPlugin,
            SingleplayerStatusPlugin,
        ))
        .init_state::<SessionType>()
        .add_computed_state::<SessionLifecycle>()
        .add_sub_state::<SessionStatus>()
        .add_sub_state::<PauseMenu>()
        .add_computed_state::<PhysicsSimulation>()
        .add_systems(
            Update,
            toggle_game_menu.run_if(in_state(SessionLifecycle::Active)),
        )
        .add_observer(handle_pause_menu_nav);
    }
}

/// Events for actions in the in-game pause menu
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuEvent {
    Resume,
    Settings,
    Save,
    Load,
    Exit,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum SessionType {
    #[default]
    None,
    Singleplayer,
    Client,
}

#[derive(SubStates, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[source(SessionLifecycle = SessionLifecycle::Active)]
pub enum SessionStatus {
    #[default]
    Playing,
    Paused,
}

#[derive(SubStates, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[source(SessionStatus = SessionStatus::Paused)]
pub enum PauseMenu {
    #[default]
    Overview,
    Settings,
    Save,
    Load,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SessionLifecycle {
    None,
    Initializing,
    Loading,
    Active,
    Cleanup,
}

impl ComputedStates for SessionLifecycle {
    type SourceStates = (
        SessionType,
        Option<SingleplayerStatus>,
        Option<ClientStatus>,
    );

    fn compute(
        (session_type, sp_status, client_status): (
            SessionType,
            Option<SingleplayerStatus>,
            Option<ClientStatus>,
        ),
    ) -> Option<Self> {
        match session_type {
            SessionType::None => Some(SessionLifecycle::None),
            SessionType::Singleplayer => {
                sp_status.map_or(Some(SessionLifecycle::None), |status| match status {
                    SingleplayerStatus::Starting => Some(SessionLifecycle::Loading),
                    SingleplayerStatus::Running => Some(SessionLifecycle::Active),
                    SingleplayerStatus::Stopping => Some(SessionLifecycle::Cleanup),
                })
            }
            SessionType::Client => client_status.map_or(Some(SessionLifecycle::None), |status| {
                // Client logic
                match status {
                    ClientStatus::Running => Some(SessionLifecycle::Active),
                    ClientStatus::Connecting | ClientStatus::Connected | ClientStatus::Syncing => {
                        Some(SessionLifecycle::Loading)
                    }
                    ClientStatus::Disconnecting => Some(SessionLifecycle::Cleanup),
                }
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum PhysicsSimulation {
    Running,
    Paused,
}

impl ComputedStates for PhysicsSimulation {
    type SourceStates = (
        SessionLifecycle,
        SessionStatus,
        SessionType,
        ServerVisibility,
    );

    fn compute(
        (lifecycle, mode, session_type, visibility): (
            SessionLifecycle,
            SessionStatus,
            SessionType,
            ServerVisibility,
        ),
    ) -> Option<Self> {
        if lifecycle != SessionLifecycle::Active {
            return Some(PhysicsSimulation::Paused);
        }

        if mode == SessionStatus::Playing {
            return Some(PhysicsSimulation::Running);
        }

        match session_type {
            SessionType::Client => Some(PhysicsSimulation::Running),
            SessionType::Singleplayer => match visibility {
                ServerVisibility::Private => Some(PhysicsSimulation::Paused),
                _ => Some(PhysicsSimulation::Running),
            },
            _ => Some(PhysicsSimulation::Paused),
        }
    }
}

// --- LOGIC ---

fn toggle_game_menu(
    current_mode: Res<State<SessionStatus>>,
    mut next_mode: ResMut<NextState<SessionStatus>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match current_mode.get() {
            SessionStatus::Playing => next_mode.set(SessionStatus::Paused),
            SessionStatus::Paused => next_mode.set(SessionStatus::Playing),
        }
    }
}
fn handle_pause_menu_nav(
    trigger: On<PauseMenuEvent>,
    mut next_pause_menu: ResMut<NextState<PauseMenu>>,
    mut next_ingame_mode: ResMut<NextState<SessionStatus>>,
    session_type: Res<State<SessionType>>,
    mut next_client_status: ResMut<NextState<ClientStatus>>,
    mut next_sp_status: ResMut<NextState<SingleplayerStatus>>,
    mut next_singleplayer_shutdown_step: ResMut<NextState<SingleplayerShutdownStep>>,
) {
    match trigger.event() {
        PauseMenuEvent::Resume => {
            next_ingame_mode.set(SessionStatus::Playing);
        }
        PauseMenuEvent::Settings => {
            next_pause_menu.set(PauseMenu::Settings);
        }
        PauseMenuEvent::Save => {
            next_pause_menu.set(PauseMenu::Save);
        }
        PauseMenuEvent::Load => {
            next_pause_menu.set(PauseMenu::Load);
        }
        PauseMenuEvent::Exit => match session_type.get() {
            SessionType::Singleplayer => {
                next_sp_status.set(SingleplayerStatus::Stopping);
                next_singleplayer_shutdown_step
                    .set(SingleplayerShutdownStep::DisconnectRemoteClients);
            }
            SessionType::Client => {
                next_client_status.set(ClientStatus::Disconnecting);
            }
            SessionType::None => {}
        },
    }
}
