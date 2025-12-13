use bevy::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppScope>()
            .add_sub_state::<MenuScreen>()
            .add_sub_state::<SingleplayerMenuScreen>()
            .add_sub_state::<MultiplayerMenuScreen>()
            .add_sub_state::<WikiMenuScreen>()
            .add_sub_state::<GameMode>()
            .add_sub_state::<InGameMode>()
            .add_sub_state::<GameMenuScreen>()
            .add_sub_state::<SingleplayerState>()
            .add_sub_state::<ServerVisibilityState>()
            .add_sub_state::<ClientState>()
            .add_computed_state::<SimulationState>()
            // Transition Handling
            .add_observer(on_appscope_event)
            .add_observer(on_main_menu_event)
            .add_observer(on_singleplayer_menu_screen_event)
            .add_observer(on_singleplayer_new_game_screen_event)
            .add_observer(on_singleplayer_load_game_screen_event)
            .add_observer(on_multiplayer_menu_screen_event)
            .add_observer(on_wiki_menu_screen_event)
            .add_observer(on_settings_menu_screen_event)
            .add_observer(on_game_mode_event)
            .add_observer(on_singleplayer_state_event)
            .add_observer(on_server_visibility_event)
            .add_observer(on_client_state_event)
            // .add_observer(on_in_game_mode_event)
            .add_observer(on_game_menu_event)
            .add_systems(Update, toggle_game_menu.run_if(in_state(AppScope::InGame)));
    }
}

fn on_appscope_event(
    event: On<ChangeAppScope>,
    mut state: ResMut<NextState<AppScope>>,
    mut menu_state: ResMut<NextState<MenuScreen>>,
) {
    match event.transition {
        AppScope::Menu => {
            state.set(AppScope::Menu);
            menu_state.set(MenuScreen::Main);
        }
        _ => {}
    }
}

fn on_main_menu_event(
    event: On<NavigateMainMenu>,
    mut state: ResMut<NextState<AppScope>>,
    mut menu_state: ResMut<NextState<MenuScreen>>,
    in_game_state: Option<Res<State<GameMode>>>,
) {
    if !in_game_state.is_none() {
        return;
    }

    match event.transition {
        MenuScreen::Main => {
            state.set(AppScope::Menu);
            menu_state.set(MenuScreen::Main);
        }
        MenuScreen::Singleplayer => {
            state.set(AppScope::Menu);
            menu_state.set(MenuScreen::Singleplayer);
        }
        MenuScreen::Multiplayer => {
            state.set(AppScope::Menu);
            menu_state.set(MenuScreen::Multiplayer);
        }
        MenuScreen::Wiki => {
            state.set(AppScope::Menu);
            menu_state.set(MenuScreen::Wiki);
        }
        MenuScreen::Settings => {
            state.set(AppScope::Menu);
            menu_state.set(MenuScreen::Settings);
        }
    }
}

fn on_singleplayer_menu_screen_event(
    event: On<NavigateSingleplayerMenu>,
    app_state: Res<State<AppScope>>,
    game_mode_state: Option<Res<State<GameMode>>>,
    mut singleplayer_menu_state: ResMut<NextState<SingleplayerMenuScreen>>,
) {
    if game_mode_state.is_some() || *app_state.get() != AppScope::Menu {
        return;
    }

    match event.transition {
        state => {
            singleplayer_menu_state.set(state);
        }
    }
}

fn on_singleplayer_new_game_screen_event(
    event: On<ControlNewGameSetup>,
    mut commands: Commands,
    app_state: Res<State<AppScope>>,
    game_mode_state: Option<Res<State<GameMode>>>,
    singleplayer_menu_state: Res<State<SingleplayerMenuScreen>>,
    new_game_menu_state: Res<State<NewGameMenuScreen>>,
    mut next_new_game_menu_state: ResMut<NextState<NewGameMenuScreen>>,
) {
    if game_mode_state.is_some() || *app_state.get() != AppScope::Menu {
        return;
    }

    match singleplayer_menu_state.get() {
        SingleplayerMenuScreen::NewGame => match *event {
            ControlNewGameSetup::Start => {
                next_new_game_menu_state.set(NewGameMenuScreen::ConfigPlayer);
            }
            ControlNewGameSetup::Next => match new_game_menu_state.get() {
                NewGameMenuScreen::ConfigPlayer => {
                    next_new_game_menu_state.set(NewGameMenuScreen::ConfigWorld);
                }
                NewGameMenuScreen::ConfigWorld => {
                    next_new_game_menu_state.set(NewGameMenuScreen::ConfigSave);
                }
                NewGameMenuScreen::ConfigSave => {
                    commands.trigger(ChangeGameMode {
                        transition: GameMode::Singleplayer,
                    });
                }
            },
            ControlNewGameSetup::Confirm => {
                commands.trigger(ChangeGameMode {
                    transition: GameMode::Singleplayer,
                });
            }
            ControlNewGameSetup::Cancel => {
                commands.trigger(NavigateSingleplayerMenu {
                    transition: SingleplayerMenuScreen::Overview,
                });
            }
            ControlNewGameSetup::Back => match new_game_menu_state.get() {
                NewGameMenuScreen::ConfigPlayer => {
                    commands.trigger(NavigateSingleplayerMenu {
                        transition: SingleplayerMenuScreen::Overview,
                    });
                }
                NewGameMenuScreen::ConfigWorld => {
                    next_new_game_menu_state.set(NewGameMenuScreen::ConfigPlayer);
                }
                NewGameMenuScreen::ConfigSave => {
                    next_new_game_menu_state.set(NewGameMenuScreen::ConfigWorld);
                }
            },
            _ => {}
        },
        _ => {}
    }
}

fn on_singleplayer_load_game_screen_event(
    event: On<ControlLoadGameSetup>,
    mut commands: Commands,
    app_state: Res<State<AppScope>>,
    game_mode_state: Option<Res<State<GameMode>>>,
    singleplayer_menu_state: Res<State<SingleplayerMenuScreen>>,
    load_game_menu_state: Res<State<LoadGameMenuScreen>>,
    mut next_load_game_menu_state: ResMut<NextState<LoadGameMenuScreen>>,
) {
    if game_mode_state.is_some() || *app_state.get() != AppScope::Menu {
        return;
    }

    match singleplayer_menu_state.get() {
        SingleplayerMenuScreen::LoadGame => match *event {
            ControlLoadGameSetup::Start => {
                next_load_game_menu_state.set(LoadGameMenuScreen::SelectSaveGame);
            }
            ControlLoadGameSetup::Next => match load_game_menu_state.get() {
                _ => {}
            },
            ControlLoadGameSetup::Confirm => {
                commands.trigger(ChangeGameMode {
                    transition: GameMode::Singleplayer,
                });
            }
            ControlLoadGameSetup::Cancel => {
                commands.trigger(NavigateSingleplayerMenu {
                    transition: SingleplayerMenuScreen::Overview,
                });
            }
            ControlLoadGameSetup::Back => match load_game_menu_state.get() {
                LoadGameMenuScreen::SelectSaveGame => {
                    commands.trigger(NavigateSingleplayerMenu {
                        transition: SingleplayerMenuScreen::Overview,
                    });
                }
            },
            _ => {}
        },
        _ => {}
    }
}

fn on_multiplayer_menu_screen_event(
    event: On<NavigateMultiplayerMenu>,
    app_state: Res<State<AppScope>>,
    game_mode_state: Option<Res<State<GameMode>>>,
    mut multiplayer_menu_state: ResMut<NextState<MultiplayerMenuScreen>>,
) {
    if game_mode_state.is_some() || *app_state.get() != AppScope::Menu {
        return;
    }
    match event.transition {
        state => {
            multiplayer_menu_state.set(state);
        }
    }
}

fn on_wiki_menu_screen_event(
    event: On<NavigateWiki>,
    mut next_state: ResMut<NextState<WikiMenuScreen>>,
) {
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}

fn on_settings_menu_screen_event(
    event: On<NavigateSettings>,
    mut next_state: ResMut<NextState<SettingsMenuScreen>>,
) {
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}

fn on_game_mode_event(
    event: On<ChangeGameMode>,
    mut _commands: Commands,
    app_state: Res<State<AppScope>>,
    singleplayer_menu_screen_opt: Option<Res<State<SingleplayerMenuScreen>>>,
    multiplayer_menu_screen_opt: Option<Res<State<MultiplayerMenuScreen>>>,
    mut next_app_state: ResMut<NextState<AppScope>>,
    mut next_singleplayer_state: ResMut<NextState<SingleplayerState>>,
    mut next_server_state: ResMut<NextState<ServerVisibilityState>>,
    mut next_game_mode: ResMut<NextState<GameMode>>,
    mut next_client_state: ResMut<NextState<ClientState>>,
) {
    match event.transition {
        GameMode::Singleplayer => {
            // Check Singleplayer Source
            if let Some(singleplayer_menu_screen) = singleplayer_menu_screen_opt {
                if *app_state.get() == AppScope::Menu
                    && (*singleplayer_menu_screen.get() == SingleplayerMenuScreen::NewGame
                        || *singleplayer_menu_screen.get() == SingleplayerMenuScreen::LoadGame)
                {
                    next_app_state.set(AppScope::InGame);
                    next_game_mode.set(GameMode::Singleplayer);
                    next_singleplayer_state.set(SingleplayerState::Starting);
                    next_server_state.set(ServerVisibilityState::Private);

                    return;
                }
            }

            // Check Multiplayer Source (Host)
            if let Some(multiplayer_menu_screen) = multiplayer_menu_screen_opt {
                if *app_state.get() == AppScope::Menu
                    && (*multiplayer_menu_screen.get() == MultiplayerMenuScreen::HostNewGame
                        || *multiplayer_menu_screen.get() == MultiplayerMenuScreen::HostSavedGame)
                {
                    next_app_state.set(AppScope::InGame);
                    next_game_mode.set(GameMode::Singleplayer);
                    next_singleplayer_state.set(SingleplayerState::Starting);
                    // Start as PendingPublic, a system will upgrade this to GoingPublic once Singleplayer is Running
                    next_server_state.set(ServerVisibilityState::PendingPublic);

                    return;
                }
            }

            warn!("Cannot transition to Singleplayer: Invalid source state or menu not active");
        }
        GameMode::Client => {
            let multiplayer_menu_screen = match multiplayer_menu_screen_opt {
                Some(screen) => screen,
                None => {
                    warn!("Multiplayer menu screen not found");
                    return;
                }
            };

            if *app_state.get() == AppScope::Menu
                && (*multiplayer_menu_screen.get() == MultiplayerMenuScreen::JoinPublicGame
                    || *multiplayer_menu_screen.get() == MultiplayerMenuScreen::JoinLocalGame)
            {
                info!("Transitioning to client");
                next_app_state.set(AppScope::InGame);
                next_game_mode.set(GameMode::Client);
                next_client_state.set(ClientState::Connecting);
            }
        }
    }
}

fn on_singleplayer_state_event(
    event: On<SetSingleplayerStatus>,
    mut next_state: ResMut<NextState<SingleplayerState>>,
    mut next_in_game_mode: ResMut<NextState<InGameMode>>,
) {
    match event.transition {
        SingleplayerState::Running => {
            next_state.set(SingleplayerState::Running);
            next_in_game_mode.set(InGameMode::Playing);
        }
        state => {
            next_state.set(state);
        }
    }
}

fn on_server_visibility_event(
    event: On<SetServerVisibility>,
    mut next_state: ResMut<NextState<ServerVisibilityState>>,
) {
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}

fn on_client_state_event(
    event: On<SetClientStatus>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}

fn toggle_game_menu(
    // ← NORMALES SYSTEM
    mut next_state: ResMut<NextState<InGameMode>>,
    keys: Res<ButtonInput<KeyCode>>,
    current_mode: Res<State<InGameMode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match *current_mode.get() {
            InGameMode::Playing => next_state.set(InGameMode::GameMenu),
            InGameMode::GameMenu => next_state.set(InGameMode::Playing),
        }
    }
}

// fn on_in_game_mode_event(
//     event: On<ToggleInGameMode>,
//     mut next_in_game_mode: ResMut<NextState<InGameMode>>,
//     mut next_game_menu_screen: ResMut<NextState<GameMenuScreen>>,
// ) {
//     match event.transition {
//         InGameMode::Playing => {
//             next_in_game_mode.set(InGameMode::Playing);
//         }
//         InGameMode::GameMenu => {
//             next_in_game_mode.set(InGameMode::GameMenu);
//             // Default to Overview when opening the menu
//             next_game_menu_screen.set(GameMenuScreen::Overview);
//         }
//     }
// }

fn on_game_menu_event(
    event: On<NavigateGameMenu>,
    mut next_game_menu_screen: ResMut<NextState<GameMenuScreen>>,
    mut next_in_game_mode: ResMut<NextState<InGameMode>>,
    mut commands: Commands,
) {
    match event.transition {
        GameMenuScreen::Overview => {
            next_game_menu_screen.set(GameMenuScreen::Overview);
        }
        GameMenuScreen::Settings => {
            next_game_menu_screen.set(GameMenuScreen::Settings);
        }
        GameMenuScreen::Save => {
            next_game_menu_screen.set(GameMenuScreen::Save);
        }
        GameMenuScreen::Load => {
            next_game_menu_screen.set(GameMenuScreen::Load);
        }
        GameMenuScreen::Exit => {
            next_game_menu_screen.set(GameMenuScreen::Exit);
            commands.trigger(SetSingleplayerStatus {
                transition: SingleplayerState::Stopping,
            });
        }
        GameMenuScreen::Resume => {
            next_in_game_mode.set(InGameMode::Playing);
        }
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ChangeAppScope {
    pub transition: AppScope,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateMainMenu {
    pub transition: MenuScreen,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateSingleplayerMenu {
    pub transition: SingleplayerMenuScreen,
}

#[derive(Event, Default, Debug, Clone, Copy)]
pub enum ControlNewGameSetup {
    #[default]
    Start,
    Next,
    Confirm,
    Cancel,
    Reset,
    Back,
}

#[derive(Event, Default, Debug, Clone, Copy)]
pub enum ControlLoadGameSetup {
    #[default]
    Start,
    Next,
    Confirm,
    Cancel,
    Reset,
    Back,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateMultiplayerMenu {
    pub transition: MultiplayerMenuScreen,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateWiki {
    pub transition: WikiMenuScreen,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateSettings {
    pub transition: SettingsMenuScreen,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ChangeGameMode {
    pub transition: GameMode,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SetSingleplayerStatus {
    pub transition: SingleplayerState,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SetServerVisibility {
    pub transition: ServerVisibilityState,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SetClientStatus {
    pub transition: ClientState,
}

// #[derive(Event, Debug, Clone, Copy)]
// pub struct ToggleInGameMode {
//     pub transition: InGameMode,
// }

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateGameMenu {
    pub transition: GameMenuScreen,
}

// --- STATE DEFINITIONS ---

/// Der oberste Scope der Anwendung.
#[derive(Default, States, Copy, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum AppScope {
    #[default]
    Menu,
    InGame,
}

// --- MENU STRUKTUR ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Menu)]
pub enum MenuScreen {
    #[default]
    Main,
    Singleplayer,
    Multiplayer,
    Wiki,
    Settings,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuScreen = MenuScreen::Singleplayer)]
pub enum SingleplayerMenuScreen {
    #[default]
    Overview,
    NewGame,
    LoadGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SingleplayerMenuScreen = SingleplayerMenuScreen::NewGame)]
pub enum NewGameMenuScreen {
    #[default]
    ConfigPlayer,
    ConfigWorld,
    ConfigSave,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SingleplayerMenuScreen = SingleplayerMenuScreen::LoadGame)]
pub enum LoadGameMenuScreen {
    #[default]
    SelectSaveGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuScreen = MenuScreen::Multiplayer)]
pub enum MultiplayerMenuScreen {
    #[default]
    Overview,
    HostNewGame,
    HostSavedGame,
    JoinPublicGame,
    JoinLocalGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuScreen = MenuScreen::Wiki)]
pub enum WikiMenuScreen {
    #[default]
    Overview,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuScreen = MenuScreen::Settings)]
pub enum SettingsMenuScreen {
    #[default]
    Overview,
}

// --- INGAME STRUKTUR ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::InGame)]
pub enum GameMode {
    #[default]
    Singleplayer,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::InGame)]
pub enum InGameMode {
    #[default]
    Playing,
    GameMenu,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(InGameMode = InGameMode::GameMenu)]
pub enum GameMenuScreen {
    #[default]
    Overview,
    Resume,
    Settings,
    Save,
    Load,
    Exit,
}

// --- SINGLEPLAYER / HOST SUBSTATES ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(GameMode = GameMode::Singleplayer)]
pub enum SingleplayerState {
    #[default]
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(GameMode = GameMode::Singleplayer)]
pub enum ServerVisibilityState {
    #[default]
    Private,
    PendingPublic,
    GoingPublic,
    Public,
    GoingPrivate,
    Failed,
}

// --- CLIENT SUBSTATES ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(GameMode = GameMode::Client)]
pub enum ClientState {
    #[default]
    Connecting,
    Connected,
    Syncing,
    Running,
    Disconnecting,
    Failed,
}

// --- COMPUTED STATES ---

/// Dieser State abstrahiert, ob die Spielsimulation (Physik, Zeit, etc.) tatsächlich läuft
/// oder angehalten ist, unabhängig davon, welches Menü gerade offen ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SimulationState {
    Running,
    Paused,
}

impl ComputedStates for SimulationState {
    type SourceStates = (InGameMode, GameMode, ServerVisibilityState);

    fn compute(
        (in_game_mode, game_mode, server_visibility): (InGameMode, GameMode, ServerVisibilityState),
    ) -> Option<Self> {
        // Wenn wir nicht "InGame" sind (also kein InGameMode existiert), ist die Simulation irrelevant oder pausiert.
        // Wir geben hier einfach None zurück oder Paused, je nach gewünschtem Verhalten beim State-Wechsel.
        // Bevy Computed States werden nur berechnet, wenn sich die Source States ändern.
        // Wenn eine Source None ist (weil der SuperState nicht aktiv ist), können wir oft auch None zurückgeben.
        match in_game_mode {
            InGameMode::Playing => Some(SimulationState::Running),
            InGameMode::GameMenu => {
                match game_mode {
                    GameMode::Client => {
                        // Client läuft im Multiplayer immer weiter, auch im Menü
                        Some(SimulationState::Running)
                    }
                    GameMode::Singleplayer => {
                        match server_visibility {
                            // Im lokalen Singleplayer pausiert das Menü das Spiel
                            ServerVisibilityState::Private => Some(SimulationState::Paused),
                            // Wenn der Server öffentlich ist, läuft das Spiel weiter (wie Multiplayer)
                            _ => Some(SimulationState::Running),
                        }
                    }
                }
            }
        }
    }
}
