use {
    super::main::MainMenuContext,
    crate::status_management::session::{
        server::ServerVisibility, singleplayer::SingleplayerStatus, SessionType,
    },
    bevy::prelude::*,
};

pub(super) struct SingleplayerMenuPlugin;

impl Plugin for SingleplayerMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SingleplayerSetup>()
            .add_sub_state::<NewGameMenuScreen>()
            .add_sub_state::<SavedGameMenuScreen>()
            .add_observer(handle_overview_nav)
            .add_observer(handle_new_game_nav)
            .add_observer(handle_load_game_nav);
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSingleplayerMenu {
    Navigate(SingleplayerSetup),
    Back,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSingleplayerNewGame {
    Next,
    Previous,
    Confirm,
    Back,
    Cancel,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSingleplayerSavedGame {
    Next,
    Previous,
    Confirm,
    Back,
    Cancel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuContext = MainMenuContext::Singleplayer)]
pub enum SingleplayerSetup {
    #[default]
    Overview,
    NewGame,
    LoadGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SingleplayerSetup = SingleplayerSetup::NewGame)]
pub enum NewGameMenuScreen {
    #[default]
    ConfigPlayer,
    ConfigWorld,
    ConfigSave,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SingleplayerSetup = SingleplayerSetup::LoadGame)]
pub enum SavedGameMenuScreen {
    #[default]
    SelectSaveGame,
}

// --- LOGIC HANDLERS ---

fn handle_overview_nav(
    trigger: On<SetSingleplayerMenu>,
    mut next_setup: ResMut<NextState<SingleplayerSetup>>,
    mut next_main_menu: ResMut<NextState<MainMenuContext>>,
    current_setup: Res<State<SingleplayerSetup>>,
) {
    if *current_setup.get() != SingleplayerSetup::Overview {
        return;
    }

    match trigger.event() {
        SetSingleplayerMenu::Navigate(target) => next_setup.set(*target),
        SetSingleplayerMenu::Back => next_main_menu.set(MainMenuContext::Main),
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_new_game_nav(
    trigger: On<SetSingleplayerNewGame>,
    current_screen: Option<Res<State<NewGameMenuScreen>>>,
    mut next_screen: ResMut<NextState<NewGameMenuScreen>>,
    mut next_setup: ResMut<NextState<SingleplayerSetup>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_singleplayer_state: ResMut<NextState<SingleplayerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    current_setup: Res<State<SingleplayerSetup>>,
) {
    if *current_setup.get() != SingleplayerSetup::NewGame {
        return;
    }

    match trigger.event() {
        SetSingleplayerNewGame::Next => {
            if let Some(screen) = current_screen {
                match *screen.get() {
                    NewGameMenuScreen::ConfigPlayer => {
                        next_screen.set(NewGameMenuScreen::ConfigWorld)
                    }
                    NewGameMenuScreen::ConfigWorld => {
                        next_screen.set(NewGameMenuScreen::ConfigSave)
                    }
                    NewGameMenuScreen::ConfigSave => {}
                }
            }
        }
        SetSingleplayerNewGame::Previous => {
            if let Some(screen) = current_screen {
                match *screen.get() {
                    NewGameMenuScreen::ConfigPlayer => next_setup.set(SingleplayerSetup::Overview),
                    NewGameMenuScreen::ConfigWorld => {
                        next_screen.set(NewGameMenuScreen::ConfigPlayer)
                    }
                    NewGameMenuScreen::ConfigSave => {
                        next_screen.set(NewGameMenuScreen::ConfigWorld)
                    }
                }
            }
        }
        SetSingleplayerNewGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_singleplayer_state.set(SingleplayerStatus::Starting);
            next_server_state.set(ServerVisibility::Private);
        }
        SetSingleplayerNewGame::Cancel => next_setup.set(SingleplayerSetup::Overview),
        SetSingleplayerNewGame::Back => {
            next_setup.set(SingleplayerSetup::Overview);
            info!("Back button clicked");
        }
    }
}

fn handle_load_game_nav(
    trigger: On<SetSingleplayerSavedGame>,
    current_screen: Option<Res<State<SavedGameMenuScreen>>>,
    current_setup: Res<State<SingleplayerSetup>>,
    mut next_setup: ResMut<NextState<SingleplayerSetup>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_singleplayer_state: ResMut<NextState<SingleplayerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
) {
    if *current_setup.get() != SingleplayerSetup::LoadGame {
        return;
    }

    match trigger.event() {
        SetSingleplayerSavedGame::Previous => {
            if let Some(screen) = current_screen {
                if *screen.get() == SavedGameMenuScreen::SelectSaveGame {
                    next_setup.set(SingleplayerSetup::Overview);
                }
            }
        }
        SetSingleplayerSavedGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_singleplayer_state.set(SingleplayerStatus::Starting);
            next_server_state.set(ServerVisibility::Private);
        }
        SetSingleplayerSavedGame::Cancel => next_setup.set(SingleplayerSetup::Overview),
        SetSingleplayerSavedGame::Back => next_setup.set(SingleplayerSetup::Overview),
        _ => {}
    }
}
