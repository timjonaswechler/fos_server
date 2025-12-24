use {
    super::main::MainMenuContext,
    crate::{
        client::ClientTarget,
        notifications::NotifyError,
        status_management::{ClientStatus, ServerVisibility, SessionType, SingleplayerStatus},
    },
    bevy::prelude::*,
};

pub(super) struct MultiplayerMenuPlugin;

impl Plugin for MultiplayerMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MultiplayerSetup>()
            .add_sub_state::<HostNewGameMenuScreen>()
            .add_sub_state::<HostSavedGameMenuScreen>()
            .add_sub_state::<JoinGameMenuScreen>()
            .add_observer(handle_overview_nav)
            .add_observer(handle_host_new_game_nav)
            .add_observer(handle_host_saved_game_nav)
            .add_observer(handle_join_game_nav);
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetMultiplayerMenu {
    Navigate(MultiplayerSetup),
    Back,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetNewHostGame {
    Next,
    Previous,
    Confirm,
    Back,
    Cancel,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSavedHostGame {
    Next,
    Previous,
    Confirm,
    Back,
    Cancel,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetJoinGame {
    Next,
    Previous,
    Confirm,
    Back,
    Cancel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuContext = MainMenuContext::Multiplayer)]
pub enum MultiplayerSetup {
    #[default]
    Overview,
    HostNewGame,
    HostSavedGame,
    JoinGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerSetup = MultiplayerSetup::HostNewGame)]
pub enum HostNewGameMenuScreen {
    #[default]
    ConfigServer,
    ConfigWorld,
    ConfigSave,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerSetup = MultiplayerSetup::HostSavedGame)]
pub enum HostSavedGameMenuScreen {
    #[default]
    Overview,
    ConfigServer,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerSetup = MultiplayerSetup::JoinGame)]
pub enum JoinGameMenuScreen {
    #[default]
    Overview,
}

// --- LOGIC HANDLERS ---

fn handle_overview_nav(
    trigger: On<SetMultiplayerMenu>,
    current_setup: Res<State<MultiplayerSetup>>,
    mut next_setup: ResMut<NextState<MultiplayerSetup>>,
    mut next_main_menu: ResMut<NextState<MainMenuContext>>,
) {
    if *current_setup.get() != MultiplayerSetup::Overview {
        return;
    }

    match trigger.event() {
        SetMultiplayerMenu::Navigate(target) => next_setup.set(*target),
        SetMultiplayerMenu::Back => next_main_menu.set(MainMenuContext::Main),
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_host_new_game_nav(
    trigger: On<SetNewHostGame>,
    current_screen: Res<State<HostNewGameMenuScreen>>,
    mut next_screen: ResMut<NextState<HostNewGameMenuScreen>>,
    mut next_setup: ResMut<NextState<MultiplayerSetup>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_singleplayer_state: ResMut<NextState<SingleplayerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    current_setup: Res<State<MultiplayerSetup>>,
) {
    if *current_setup.get() != MultiplayerSetup::HostNewGame {
        return;
    }

    match trigger.event() {
        SetNewHostGame::Next => match current_screen.get() {
            HostNewGameMenuScreen::ConfigServer => {
                next_screen.set(HostNewGameMenuScreen::ConfigWorld)
            }
            HostNewGameMenuScreen::ConfigWorld => {
                next_screen.set(HostNewGameMenuScreen::ConfigSave)
            }
            HostNewGameMenuScreen::ConfigSave => {}
        },
        SetNewHostGame::Previous => match current_screen.get() {
            HostNewGameMenuScreen::ConfigServer => next_setup.set(MultiplayerSetup::Overview),
            HostNewGameMenuScreen::ConfigWorld => {
                next_screen.set(HostNewGameMenuScreen::ConfigServer)
            }
            HostNewGameMenuScreen::ConfigSave => {
                next_screen.set(HostNewGameMenuScreen::ConfigWorld)
            }
        },
        SetNewHostGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_singleplayer_state.set(SingleplayerStatus::Starting);
            next_server_state.set(ServerVisibility::PendingPublic);
        }
        SetNewHostGame::Cancel => next_setup.set(MultiplayerSetup::Overview),
        SetNewHostGame::Back => next_setup.set(MultiplayerSetup::Overview),
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_host_saved_game_nav(
    trigger: On<SetSavedHostGame>,
    current_screen: Res<State<HostSavedGameMenuScreen>>,
    mut next_screen: ResMut<NextState<HostSavedGameMenuScreen>>,
    mut next_setup: ResMut<NextState<MultiplayerSetup>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_singleplayer_state: ResMut<NextState<SingleplayerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    current_setup: Res<State<MultiplayerSetup>>,
) {
    if *current_setup.get() != MultiplayerSetup::HostSavedGame {
        return;
    }

    match trigger.event() {
        SetSavedHostGame::Next => {
            if *current_screen.get() == HostSavedGameMenuScreen::Overview {
                next_screen.set(HostSavedGameMenuScreen::ConfigServer);
            }
        }
        SetSavedHostGame::Previous => {
            if *current_screen.get() == HostSavedGameMenuScreen::ConfigServer {
                next_screen.set(HostSavedGameMenuScreen::Overview);
            }
        }
        SetSavedHostGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_singleplayer_state.set(SingleplayerStatus::Starting);
            next_server_state.set(ServerVisibility::PendingPublic);
        }
        SetSavedHostGame::Cancel => next_setup.set(MultiplayerSetup::Overview),
        SetSavedHostGame::Back => next_setup.set(MultiplayerSetup::Overview),
    }
}

fn handle_join_game_nav(
    trigger: On<SetJoinGame>,
    current_setup: Res<State<MultiplayerSetup>>,
    client_target: Option<Res<ClientTarget>>,
    mut commands: Commands,
    mut next_setup: ResMut<NextState<MultiplayerSetup>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_client_state: ResMut<NextState<ClientStatus>>,
) {
    if *current_setup.get() != MultiplayerSetup::JoinGame {
        return;
    }

    match trigger.event() {
        SetJoinGame::Back => next_setup.set(MultiplayerSetup::Overview),
        SetJoinGame::Confirm => {
            let Some(client_target) = client_target else {
                commands.trigger(NotifyError::new("⚠️ Bitte Server-Adresse eingeben!"));
                return;
            };
            if !client_target.is_valid {
                return; // Fehler schon beim Setzen gezeigt
            }
            info!(
                "✅ Server validiert: {}:{}",
                client_target.ip, client_target.port
            );

            next_session_type.set(SessionType::Client);
            next_client_state.set(ClientStatus::Connecting);
        }
        SetJoinGame::Cancel => next_setup.set(MultiplayerSetup::Overview),
        _ => {}
    }
}
