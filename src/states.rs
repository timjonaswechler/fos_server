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
            .add_observer(on_in_game_mode_event)
            .add_observer(on_game_menu_event)
            .add_systems(Update, toggle_game_menu.run_if(in_state(AppScope::InGame)));
    }
}

fn on_appscope_event(
    event: On<AppScopeEvent>,
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
    event: On<MainMenuEvent>,
    mut state: ResMut<NextState<AppScope>>,
    mut menu_state: ResMut<NextState<MenuScreen>>,
    in_game_state: Option<Res<State<GameMode>>>,
) {
    if in_game_state.is_none() {
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
    event: On<SingleplayerMenuEvent>,
    app_state: Res<State<AppScope>>,
    in_game_state: Option<Res<State<GameMode>>>,
    mut singleplayer_menu_state: ResMut<NextState<SingleplayerMenuScreen>>,
) {
    if in_game_state.is_some() || *app_state.get() != AppScope::Menu {
        return;
    }

    match event.transition {
        SingleplayerMenuScreen::Overview => {
            singleplayer_menu_state.set(SingleplayerMenuScreen::Overview);
        }
        SingleplayerMenuScreen::NewGame => {
            singleplayer_menu_state.set(SingleplayerMenuScreen::NewGame);
        }
        SingleplayerMenuScreen::LoadGame => {
            singleplayer_menu_state.set(SingleplayerMenuScreen::LoadGame);
        }
    }
}

fn on_singleplayer_new_game_screen_event(
    event: On<NewGameSetupEvent>,
    mut commands: Commands,
    app_state: Res<State<AppScope>>,
    game_mode_state: Option<Res<State<GameMode>>>,
    singleplayer_menu_state: Res<State<SingleplayerMenuScreen>>,
    new_game_menu_state: Res<State<NewGameMenuScreen>>,
    mut next_new_game_menu_state: ResMut<NextState<NewGameMenuScreen>>,
) {
    if game_mode_state.is_none() && *app_state.get() == AppScope::Menu {
        return;
    }

    match singleplayer_menu_state.get() {
        SingleplayerMenuScreen::NewGame => match *event {
            NewGameSetupEvent::Start => {
                next_new_game_menu_state.set(NewGameMenuScreen::ConfigPlayer);
            }
            NewGameSetupEvent::Next => match new_game_menu_state.get() {
                NewGameMenuScreen::ConfigPlayer => {
                    next_new_game_menu_state.set(NewGameMenuScreen::ConfigWorld);
                }
                NewGameMenuScreen::ConfigWorld => {
                    next_new_game_menu_state.set(NewGameMenuScreen::ConfigSave);
                }
                NewGameMenuScreen::ConfigSave => {
                    commands.trigger(GameModeEvent {
                        transition: GameMode::Singleplayer,
                    });
                }
            },
            NewGameSetupEvent::Confirm => {
                commands.trigger(GameModeEvent {
                    transition: GameMode::Singleplayer,
                });
            }
            NewGameSetupEvent::Cancel => {
                commands.trigger(SingleplayerMenuEvent {
                    transition: SingleplayerMenuScreen::Overview,
                });
            }
            NewGameSetupEvent::Back => match new_game_menu_state.get() {
                NewGameMenuScreen::ConfigPlayer => {
                    commands.trigger(SingleplayerMenuEvent {
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
    event: On<LoadGameSetupEvent>,
    mut commands: Commands,
    app_state: Res<State<AppScope>>,
    game_mode_state: Option<Res<State<GameMode>>>,
    singleplayer_menu_state: Res<State<SingleplayerMenuScreen>>,
    load_game_menu_state: Res<State<LoadGameMenuScreen>>,
    mut next_load_game_menu_state: ResMut<NextState<LoadGameMenuScreen>>,
) {
    if game_mode_state.is_none() && *app_state.get() == AppScope::Menu {
        return;
    }

    match singleplayer_menu_state.get() {
        SingleplayerMenuScreen::LoadGame => match *event {
            LoadGameSetupEvent::Start => {
                next_load_game_menu_state.set(LoadGameMenuScreen::SelectSaveGame);
            }
            LoadGameSetupEvent::Next => match load_game_menu_state.get() {
                _ => {}
            },
            LoadGameSetupEvent::Confirm => {
                commands.trigger(GameModeEvent {
                    transition: GameMode::Singleplayer,
                });
            }
            LoadGameSetupEvent::Cancel => {
                commands.trigger(SingleplayerMenuEvent {
                    transition: SingleplayerMenuScreen::Overview,
                });
            }
            LoadGameSetupEvent::Back => match load_game_menu_state.get() {
                LoadGameMenuScreen::SelectSaveGame => {
                    commands.trigger(SingleplayerMenuEvent {
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
    event: On<MultiplayerMenuEvent>,
    app_state: Res<State<AppScope>>,
    in_game_state: Option<Res<State<GameMode>>>,
    mut multiplayer_menu_state: ResMut<NextState<MultiplayerMenuScreen>>,
) {
    if in_game_state.is_none() && *app_state.get() == AppScope::Menu {
        return;
    }
    match event.transition {
        MultiplayerMenuScreen::Overview => {
            multiplayer_menu_state.set(MultiplayerMenuScreen::Overview);
        }
        MultiplayerMenuScreen::HostNewGame => {
            multiplayer_menu_state.set(MultiplayerMenuScreen::HostNewGame);
        }
        MultiplayerMenuScreen::HostSavedGame => {
            multiplayer_menu_state.set(MultiplayerMenuScreen::HostSavedGame);
        }
        MultiplayerMenuScreen::JoinPublicGame => {
            multiplayer_menu_state.set(MultiplayerMenuScreen::JoinPublicGame);
        }
        MultiplayerMenuScreen::JoinLocalGame => {
            multiplayer_menu_state.set(MultiplayerMenuScreen::JoinLocalGame);
        }
    }
}

fn on_wiki_menu_screen_event(
    event: On<WikiMenuEvent>,
    mut wiki_menu_state: ResMut<NextState<WikiMenuScreen>>,
) {
    match event.transition {
        WikiMenuScreen::Overview => {
            wiki_menu_state.set(WikiMenuScreen::Overview);
        }
    }
}

fn on_settings_menu_screen_event(
    event: On<SettingsMenuEvent>,
    mut settings_menu_state: ResMut<NextState<SettingsMenuScreen>>,
) {
    match event.transition {
        SettingsMenuScreen::Overview => {
            settings_menu_state.set(SettingsMenuScreen::Overview);
        }
    }
}

fn on_game_mode_event(
    event: On<GameModeEvent>,
    app_state: Res<State<AppScope>>,
    singleplayer_menu_screen_opt: Option<Res<State<SingleplayerMenuScreen>>>,
    multiplayer_menu_screen_opt: Option<Res<State<MultiplayerMenuScreen>>>,
    mut next_app_state: ResMut<NextState<AppScope>>,
    mut next_singleplayer_state: ResMut<NextState<SingleplayerState>>,
    mut next_server_state: ResMut<NextState<ServerVisibilityState>>,
    mut next_client_state: ResMut<NextState<ClientState>>,
    mut next_game_mode: ResMut<NextState<GameMode>>,
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
                next_app_state.set(AppScope::InGame);
                next_game_mode.set(GameMode::Client);
                next_client_state.set(ClientState::Connecting);
            }
        }
    }
}

fn on_singleplayer_state_event(
    event: On<SingleplayerStateEvent>,
    mut singleplayer_state: ResMut<NextState<SingleplayerState>>,
) {
    match event.transition {
        SingleplayerState::Starting => {
            singleplayer_state.set(SingleplayerState::Starting);
        }
        SingleplayerState::Running => {
            singleplayer_state.set(SingleplayerState::Running);
        }
        SingleplayerState::Stopping => {
            singleplayer_state.set(SingleplayerState::Stopping);
        }
        SingleplayerState::Failed => {
            singleplayer_state.set(SingleplayerState::Failed);
        }
    }
}

fn on_server_visibility_event(
    event: On<ServerVisibilityEvent>,
    mut next_state: ResMut<NextState<ServerVisibilityState>>,
) {
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}

fn on_client_state_event(
    event: On<ClientStateEvent>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}

fn toggle_game_menu(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    current_mode: Option<Res<State<InGameMode>>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Some(mode) = current_mode {
            match mode.get() {
                InGameMode::Playing => {
                    commands.trigger(InGameModeEvent {
                        transition: InGameMode::GameMenu,
                    });
                }
                InGameMode::GameMenu => {
                    commands.trigger(InGameModeEvent {
                        transition: InGameMode::Playing,
                    });
                }
            }
        }
    }
}

fn on_in_game_mode_event(
    event: On<InGameModeEvent>,
    mut next_in_game_mode: ResMut<NextState<InGameMode>>,
    mut next_game_menu_screen: ResMut<NextState<GameMenuScreen>>,
) {
    match event.transition {
        InGameMode::Playing => {
            next_in_game_mode.set(InGameMode::Playing);
        }
        InGameMode::GameMenu => {
            next_in_game_mode.set(InGameMode::GameMenu);
            // Default to Overview when opening the menu
            next_game_menu_screen.set(GameMenuScreen::Overview);
        }
    }
}

fn on_game_menu_event(
    event: On<GameMenuEvent>,
    mut next_game_menu_screen: ResMut<NextState<GameMenuScreen>>,
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
            // Logic to actually exit/disconnect would go here or be triggered by entering this state
            commands.trigger(MainMenuEvent {
                transition: MenuScreen::Main,
            });
        }
        GameMenuScreen::Resume => {
            commands.trigger(InGameModeEvent {
                transition: InGameMode::Playing,
            });
        }
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct AppScopeEvent {
    pub transition: AppScope,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct MainMenuEvent {
    pub transition: MenuScreen,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SingleplayerMenuEvent {
    pub transition: SingleplayerMenuScreen,
}

#[derive(Event, Default, Debug, Clone, Copy)]
pub enum NewGameSetupEvent {
    #[default]
    Start,
    Next,
    Confirm,
    Cancel,
    Reset,
    Back,
}

#[derive(Event, Default, Debug, Clone, Copy)]
pub enum LoadGameSetupEvent {
    #[default]
    Start,
    Next,
    Confirm,
    Cancel,
    Reset,
    Back,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct MultiplayerMenuEvent {
    pub transition: MultiplayerMenuScreen,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct WikiMenuEvent {
    pub transition: WikiMenuScreen,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SettingsMenuEvent {
    pub transition: SettingsMenuScreen,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct GameModeEvent {
    pub transition: GameMode,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SingleplayerStateEvent {
    pub transition: SingleplayerState,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ServerVisibilityEvent {
    pub transition: ServerVisibilityState,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ClientStateEvent {
    pub transition: ClientState,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct InGameModeEvent {
    pub transition: InGameMode,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct GameMenuEvent {
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
