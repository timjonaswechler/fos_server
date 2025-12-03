use bevy::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppScope>()
            .add_sub_state::<MenuState>()
            .add_sub_state::<SingleplayerMenuState>()
            .add_sub_state::<MultiplayerMenuState>()
            .add_sub_state::<WikiMenuState>()
            .add_sub_state::<InGameState>()
            .add_sub_state::<SingleplayerState>()
            .add_sub_state::<ServerVisibilityState>()
            .add_sub_state::<ClientState>()
            // Transition Handling
            .add_observer(on_main_menu_event)
            .add_observer(on_singleplayer_menu_event)
            .add_observer(on_multiplayer_menu_event)
            .add_observer(on_ingame_event);
    }
}

fn on_ingame_event(
    event: On<InGameEvent>,
    app_state: Res<State<AppScope>>,
    singleplayer_menu_state: Res<State<SingleplayerMenuState>>,
    multiplayer_menu_state: Res<State<MultiplayerMenuState>>,
    mut next_app_state: ResMut<NextState<AppScope>>,
    mut next_singleplayer_state: ResMut<NextState<SingleplayerState>>,
    mut next_server_state: ResMut<NextState<ServerVisibilityState>>,
    mut next_client_state: ResMut<NextState<ClientState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    match *event {
        InGameEvent::RequestTransitionTo(InGameState::Singleplayer) => {
            if *app_state.get() == AppScope::Menu
                && (*singleplayer_menu_state.get() == SingleplayerMenuState::NewGame
                    || *singleplayer_menu_state.get() == SingleplayerMenuState::LoadGame)
            {
                next_app_state.set(AppScope::InGame);
                next_in_game_state.set(InGameState::Singleplayer);
                next_singleplayer_state.set(SingleplayerState::Starting);
                next_server_state.set(ServerVisibilityState::Local);
            }
        }
        InGameEvent::RequestTransitionTo(InGameState::Host) => {
            if *app_state.get() == AppScope::Menu
                && (*multiplayer_menu_state.get() == MultiplayerMenuState::HostNewGame
                    || *multiplayer_menu_state.get() == MultiplayerMenuState::HostSavedGame)
            {
                next_app_state.set(AppScope::InGame);
                next_in_game_state.set(InGameState::Singleplayer);
                next_singleplayer_state.set(SingleplayerState::Starting);
                next_server_state.set(ServerVisibilityState::GoingPublic);
            }
        }
        InGameEvent::RequestTransitionTo(InGameState::Client) => {
            if *app_state.get() == AppScope::Menu
                && (*multiplayer_menu_state.get() == MultiplayerMenuState::JoinPublicGame
                    || *multiplayer_menu_state.get() == MultiplayerMenuState::JoinLocalGame)
            {
                next_app_state.set(AppScope::InGame);
                next_in_game_state.set(InGameState::Client);
                next_client_state.set(ClientState::Connecting);
            }
        }
    }
}

fn on_main_menu_event(
    event: On<MainMenuEvent>,
    mut state: ResMut<NextState<AppScope>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    in_game_state: Option<Res<State<InGameState>>>,
) {
    if in_game_state.is_none() {
        match *event {
            MainMenuEvent::RequestTransitionTo(MenuState::Main) => {
                state.set(AppScope::Menu);
                menu_state.set(MenuState::Main);
            }
            MainMenuEvent::RequestTransitionTo(MenuState::Singleplayer) => {
                state.set(AppScope::Menu);
                menu_state.set(MenuState::Singleplayer);
            }
            MainMenuEvent::RequestTransitionTo(MenuState::Multiplayer) => {
                state.set(AppScope::Menu);
                menu_state.set(MenuState::Multiplayer);
            }
            MainMenuEvent::RequestTransitionTo(MenuState::Wiki) => {
                state.set(AppScope::Menu);
                menu_state.set(MenuState::Wiki);
            }
            MainMenuEvent::RequestTransitionTo(MenuState::Settings) => {
                state.set(AppScope::Menu);
                menu_state.set(MenuState::Settings);
            }
        }
    }
}

fn on_singleplayer_menu_event(
    event: On<SingleplayerMenuEvent>,
    app_state: Res<State<AppScope>>,
    in_game_state: Option<Res<State<InGameState>>>,
    mut singleplayer_menu_state: ResMut<NextState<SingleplayerMenuState>>,
) {
    if in_game_state.is_none() && *app_state.get() == AppScope::Menu {
        match *event {
            SingleplayerMenuEvent::RequestTransitionTo(SingleplayerMenuState::Overview) => {
                singleplayer_menu_state.set(SingleplayerMenuState::Overview);
            }
            SingleplayerMenuEvent::RequestTransitionTo(SingleplayerMenuState::NewGame) => {
                singleplayer_menu_state.set(SingleplayerMenuState::NewGame);
            }
            SingleplayerMenuEvent::RequestTransitionTo(SingleplayerMenuState::LoadGame) => {
                singleplayer_menu_state.set(SingleplayerMenuState::LoadGame);
            }
        }
    }
}

fn on_multiplayer_menu_event(
    event: On<MultiplayerMenuEvent>,
    app_state: Res<State<AppScope>>,
    in_game_state: Option<Res<State<InGameState>>>,
    mut multiplayer_menu_state: ResMut<NextState<MultiplayerMenuState>>,
) {
    if in_game_state.is_none() && *app_state.get() == AppScope::Menu {
        match *event {
            MultiplayerMenuEvent::RequestTransitionTo(MultiplayerMenuState::Overview) => {
                multiplayer_menu_state.set(MultiplayerMenuState::Overview);
            }
            MultiplayerMenuEvent::RequestTransitionTo(MultiplayerMenuState::HostNewGame) => {
                multiplayer_menu_state.set(MultiplayerMenuState::HostNewGame);
            }
            MultiplayerMenuEvent::RequestTransitionTo(MultiplayerMenuState::HostSavedGame) => {
                multiplayer_menu_state.set(MultiplayerMenuState::HostSavedGame);
            }
            MultiplayerMenuEvent::RequestTransitionTo(MultiplayerMenuState::JoinPublicGame) => {
                multiplayer_menu_state.set(MultiplayerMenuState::JoinPublicGame);
            }
            MultiplayerMenuEvent::RequestTransitionTo(MultiplayerMenuState::JoinLocalGame) => {
                multiplayer_menu_state.set(MultiplayerMenuState::JoinLocalGame);
            }
        }
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub enum MainMenuEvent {
    RequestTransitionTo(MenuState),
}

#[derive(Event, Debug, Clone, Copy)]
pub enum SingleplayerMenuEvent {
    RequestTransitionTo(SingleplayerMenuState),
}

#[derive(Event, Debug, Clone, Copy)]
pub enum MultiplayerMenuEvent {
    RequestTransitionTo(MultiplayerMenuState),
}

#[derive(Event, Debug, Clone, Copy)]
pub enum InGameEvent {
    RequestTransitionTo(InGameState),
}

// --- STATE DEFINITIONS ---

/// Der oberste Scope der Anwendung.
#[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum AppScope {
    #[default]
    Menu,
    InGame,
}

// --- MENU STRUKTUR ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Menu)]
pub enum MenuState {
    #[default]
    Main,
    Singleplayer,
    Multiplayer,
    Wiki,
    Settings,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuState = MenuState::Singleplayer)]
pub enum SingleplayerMenuState {
    #[default]
    Overview,
    NewGame,
    LoadGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuState = MenuState::Multiplayer)]
pub enum MultiplayerMenuState {
    #[default]
    Overview,
    HostNewGame,
    HostSavedGame,
    JoinPublicGame,
    JoinLocalGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuState = MenuState::Wiki)]
pub enum WikiMenuState {
    #[default]
    Overview,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuState = MenuState::Settings)]
pub enum SettingsMenuState {
    #[default]
    Overview,
}

// --- INGAME STRUKTUR ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::InGame)]
pub enum InGameState {
    #[default]
    Singleplayer,
    Host,
    Client,
}

// --- SINGLEPLAYER / HOST SUBSTATES ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(InGameState = InGameState::Singleplayer)]
pub enum SingleplayerState {
    #[default]
    Starting,
    Running,
    Paused,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(InGameState = InGameState::Singleplayer)]
pub enum ServerVisibilityState {
    #[default]
    Local,
    GoingPublic,
    Public,
    GoingPrivate,
    Failed,
}

// --- CLIENT SUBSTATES ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(InGameState = InGameState::Client)]
pub enum ClientState {
    #[default]
    Connecting,
    Connected,
    Syncing,
    Running,
    Disconnecting,
    Failed,
}
