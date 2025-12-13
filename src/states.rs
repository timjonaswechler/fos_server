// rule: no trigger commands only state changes

use bevy::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GamePhase>()
            .add_sub_state::<MenuContext>()
            .add_sub_state::<SingleplayerSetup>()
            .add_sub_state::<MultiplayerSetup>()
            .add_sub_state::<WikiMenuScreen>()
            .add_sub_state::<SessionType>()
            .add_sub_state::<GameplayFocus>()
            .add_sub_state::<PauseMenu>()
            .add_sub_state::<SingleplayerStatus>()
            .add_sub_state::<ServerVisibility>()
            .add_sub_state::<ClientStatus>()
            .add_computed_state::<PhysicsSimulation>()
            // Transition Handling
            .add_observer(on_change_app_scope)
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
            .add_systems(Update, toggle_game_menu.run_if(in_state(GamePhase::InGame)));
    }
}

fn on_change_app_scope(
    event: On<ChangeAppScope>,
    mut state: ResMut<NextState<GamePhase>>,
    mut menu_state: ResMut<NextState<MenuContext>>,
) {
    match event.transition {
        GamePhase::Menu => {
            state.set(GamePhase::Menu);
            menu_state.set(MenuContext::Main);
        }
        _ => {}
    }
}

fn on_main_menu_event(
    event: On<NavigateMainMenu>,
    mut state: ResMut<NextState<GamePhase>>,
    mut menu_state: ResMut<NextState<MenuContext>>,
    in_game_state: Option<Res<State<SessionType>>>,
) {
    if !in_game_state.is_none() {
        return;
    }

    match event.transition {
        MenuContext::Main => {
            state.set(GamePhase::Menu);
            menu_state.set(MenuContext::Main);
        }
        MenuContext::Singleplayer => {
            state.set(GamePhase::Menu);
            menu_state.set(MenuContext::Singleplayer);
        }
        MenuContext::Multiplayer => {
            state.set(GamePhase::Menu);
            menu_state.set(MenuContext::Multiplayer);
        }
        MenuContext::Wiki => {
            state.set(GamePhase::Menu);
            menu_state.set(MenuContext::Wiki);
        }
        MenuContext::Settings => {
            state.set(GamePhase::Menu);
            menu_state.set(MenuContext::Settings);
        }
    }
}

fn on_singleplayer_menu_screen_event(
    event: On<NavigateSingleplayerMenu>,
    app_state: Res<State<GamePhase>>,
    game_mode_state: Option<Res<State<SessionType>>>,
    mut singleplayer_menu_state: ResMut<NextState<SingleplayerSetup>>,
) {
    if game_mode_state.is_some() || *app_state.get() != GamePhase::Menu {
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
    app_state: Res<State<GamePhase>>,
    game_mode_state: Option<Res<State<SessionType>>>,
    singleplayer_menu_state: Res<State<SingleplayerSetup>>,
    new_game_menu_state: Res<State<NewGameMenuScreen>>,
    mut next_new_game_menu_state: ResMut<NextState<NewGameMenuScreen>>,
    mut next_game_mode: ResMut<NextState<SessionType>>,
    mut next_singleplayer_menu: ResMut<NextState<SingleplayerSetup>>,
) {
    if game_mode_state.is_some() || *app_state.get() != GamePhase::Menu {
        return;
    }

    match singleplayer_menu_state.get() {
        SingleplayerSetup::NewGame => match *event {
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
                    next_game_mode.set(SessionType::Singleplayer);
                }
            },
            ControlNewGameSetup::Confirm => {
                next_game_mode.set(SessionType::Singleplayer);
            }
            ControlNewGameSetup::Cancel => {
                next_singleplayer_menu.set(SingleplayerSetup::Overview);
            }
            ControlNewGameSetup::Back => match new_game_menu_state.get() {
                NewGameMenuScreen::ConfigPlayer => {
                    next_singleplayer_menu.set(SingleplayerSetup::Overview);
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
    app_state: Res<State<GamePhase>>,
    game_mode_state: Option<Res<State<SessionType>>>,
    singleplayer_menu_state: Res<State<SingleplayerSetup>>,
    load_game_menu_state: Res<State<LoadGameMenuScreen>>,
    mut next_load_game_menu_state: ResMut<NextState<LoadGameMenuScreen>>,
    mut next_game_mode: ResMut<NextState<SessionType>>,
    mut next_singleplayer_menu: ResMut<NextState<SingleplayerSetup>>,
) {
    if game_mode_state.is_some() || *app_state.get() != GamePhase::Menu {
        return;
    }

    match singleplayer_menu_state.get() {
        SingleplayerSetup::LoadGame => match *event {
            ControlLoadGameSetup::Start => {
                next_load_game_menu_state.set(LoadGameMenuScreen::SelectSaveGame);
            }
            ControlLoadGameSetup::Next => match load_game_menu_state.get() {
                _ => {}
            },
            ControlLoadGameSetup::Confirm => next_game_mode.set(SessionType::Singleplayer),
            ControlLoadGameSetup::Cancel => next_singleplayer_menu.set(SingleplayerSetup::Overview),
            ControlLoadGameSetup::Back => match load_game_menu_state.get() {
                LoadGameMenuScreen::SelectSaveGame => {
                    next_singleplayer_menu.set(SingleplayerSetup::Overview)
                }
            },
            _ => {}
        },
        _ => {}
    }
}

fn on_multiplayer_menu_screen_event(
    event: On<NavigateMultiplayerMenu>,
    app_state: Res<State<GamePhase>>,
    game_mode_state: Option<Res<State<SessionType>>>,
    mut multiplayer_menu_state: ResMut<NextState<MultiplayerSetup>>,
) {
    if game_mode_state.is_some() || *app_state.get() != GamePhase::Menu {
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
    app_state: Res<State<GamePhase>>,
    singleplayer_menu_screen_opt: Option<Res<State<SingleplayerSetup>>>,
    multiplayer_menu_screen_opt: Option<Res<State<MultiplayerSetup>>>,
    mut next_app_state: ResMut<NextState<GamePhase>>,
    mut next_singleplayer_state: ResMut<NextState<SingleplayerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    mut next_game_mode: ResMut<NextState<SessionType>>,
    mut next_client_state: ResMut<NextState<ClientStatus>>,
) {
    match event.transition {
        SessionType::Singleplayer => {
            // Check Singleplayer Source
            if let Some(singleplayer_menu_screen) = singleplayer_menu_screen_opt {
                if *app_state.get() == GamePhase::Menu
                    && (*singleplayer_menu_screen.get() == SingleplayerSetup::NewGame
                        || *singleplayer_menu_screen.get() == SingleplayerSetup::LoadGame)
                {
                    next_app_state.set(GamePhase::InGame);
                    next_game_mode.set(SessionType::Singleplayer);
                    next_singleplayer_state.set(SingleplayerStatus::Starting);
                    next_server_state.set(ServerVisibility::Private);

                    return;
                }
            }

            // Check Multiplayer Source (Host)
            if let Some(multiplayer_menu_screen) = multiplayer_menu_screen_opt {
                if *app_state.get() == GamePhase::Menu
                    && (*multiplayer_menu_screen.get() == MultiplayerSetup::HostNewGame
                        || *multiplayer_menu_screen.get() == MultiplayerSetup::HostSavedGame)
                {
                    next_app_state.set(GamePhase::InGame);
                    next_game_mode.set(SessionType::Singleplayer);
                    next_singleplayer_state.set(SingleplayerStatus::Starting);
                    // Start as PendingPublic, a system will upgrade this to GoingPublic once Singleplayer is Running
                    next_server_state.set(ServerVisibility::PendingPublic);

                    return;
                }
            }

            warn!("Cannot transition to Singleplayer: Invalid source state or menu not active");
        }
        SessionType::Client => {
            let multiplayer_menu_screen = match multiplayer_menu_screen_opt {
                Some(screen) => screen,
                None => {
                    warn!("Multiplayer menu screen not found");
                    return;
                }
            };

            if *app_state.get() == GamePhase::Menu
                && (*multiplayer_menu_screen.get() == MultiplayerSetup::JoinPublicGame
                    || *multiplayer_menu_screen.get() == MultiplayerSetup::JoinLocalGame)
            {
                info!("Transitioning to client");
                next_app_state.set(GamePhase::InGame);
                next_game_mode.set(SessionType::Client);
                next_client_state.set(ClientStatus::Connecting);
            }
        }
    }
}

fn on_singleplayer_state_event(
    event: On<SetSingleplayerStatus>,
    mut next_state: ResMut<NextState<SingleplayerStatus>>,
    mut next_in_game_mode: ResMut<NextState<GameplayFocus>>,
) {
    match event.transition {
        SingleplayerStatus::Running => {
            next_state.set(SingleplayerStatus::Running);
            next_in_game_mode.set(GameplayFocus::Playing);
        }
        state => {
            next_state.set(state);
        }
    }
}

fn on_server_visibility_event(
    event: On<SetServerVisibility>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}

fn on_client_state_event(
    event: On<SetClientStatus>,
    mut next_state: ResMut<NextState<ClientStatus>>,
) {
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}

fn toggle_game_menu(
    mut next_state: ResMut<NextState<GameplayFocus>>,
    keys: Res<ButtonInput<KeyCode>>,
    current_mode: Res<State<GameplayFocus>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match *current_mode.get() {
            GameplayFocus::Playing => next_state.set(GameplayFocus::GameMenu),
            GameplayFocus::GameMenu => next_state.set(GameplayFocus::Playing),
        }
    }
}

fn on_game_menu_event(
    event: On<NavigateGameMenu>,
    mut next_game_menu_screen: ResMut<NextState<PauseMenu>>,
    mut next_in_game_mode: ResMut<NextState<GameplayFocus>>,
    game_mode_state: Res<State<SessionType>>,
    next_client_state: Option<ResMut<NextState<ClientStatus>>>,
    next_singleplayer_state: Option<ResMut<NextState<SingleplayerStatus>>>,
) {
    match event.transition {
        PauseMenu::Overview => {
            next_game_menu_screen.set(PauseMenu::Overview);
        }
        PauseMenu::Settings => {
            next_game_menu_screen.set(PauseMenu::Settings);
        }
        PauseMenu::Save => {
            next_game_menu_screen.set(PauseMenu::Save);
        }
        PauseMenu::Load => {
            next_game_menu_screen.set(PauseMenu::Load);
        }
        PauseMenu::Exit => {
            next_game_menu_screen.set(PauseMenu::Exit);
            match game_mode_state.get() {
                SessionType::Singleplayer => {
                    if let Some(mut singleplayer_state) = next_singleplayer_state {
                        singleplayer_state.set(SingleplayerStatus::Stopping)
                    }
                }
                SessionType::Client => {
                    if let Some(mut client_state) = next_client_state {
                        client_state.set(ClientStatus::Disconnecting)
                    }
                }
            }
        }
        PauseMenu::Resume => {
            next_in_game_mode.set(GameplayFocus::Playing);
        }
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ChangeAppScope {
    pub transition: GamePhase,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateMainMenu {
    pub transition: MenuContext,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateSingleplayerMenu {
    pub transition: SingleplayerSetup,
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
    pub transition: MultiplayerSetup,
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
    pub transition: SessionType,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SetSingleplayerStatus {
    pub transition: SingleplayerStatus,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SetServerVisibility {
    pub transition: ServerVisibility,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SetClientStatus {
    pub transition: ClientStatus,
}

// #[derive(Event, Debug, Clone, Copy)]
// pub struct ToggleInGameMode {
//     pub transition: GameplayFocus,
// }

#[derive(Event, Debug, Clone, Copy)]
pub struct NavigateGameMenu {
    pub transition: PauseMenu,
}

// --- STATE DEFINITIONS ---

/// Der oberste Scope der Anwendung.
#[derive(Default, States, Copy, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum GamePhase {
    #[default]
    Menu,
    InGame,
}

// --- MENU STRUKTUR ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(GamePhase = GamePhase::Menu)]
pub enum MenuContext {
    #[default]
    Main,
    Singleplayer,
    Multiplayer,
    Wiki,
    Settings,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuContext = MenuContext::Singleplayer)]
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
pub enum LoadGameMenuScreen {
    #[default]
    SelectSaveGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuContext = MenuContext::Multiplayer)]
pub enum MultiplayerSetup {
    #[default]
    Overview,
    HostNewGame,
    HostSavedGame,
    JoinPublicGame,
    JoinLocalGame,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuContext = MenuContext::Wiki)]
pub enum WikiMenuScreen {
    #[default]
    Overview,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MenuContext = MenuContext::Settings)]
pub enum SettingsMenuScreen {
    #[default]
    Overview,
}

// --- INGAME STRUKTUR ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(GamePhase = GamePhase::InGame)]
pub enum SessionType {
    #[default]
    Singleplayer,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(GamePhase = GamePhase::InGame)]
pub enum GameplayFocus {
    #[default]
    Playing,
    GameMenu,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(GameplayFocus = GameplayFocus::GameMenu)]
pub enum PauseMenu {
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
#[source(SessionType = SessionType::Singleplayer)]
pub enum SingleplayerStatus {
    #[default]
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SessionType = SessionType::Singleplayer)]
pub enum ServerVisibility {
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
#[source(SessionType = SessionType::Client)]
pub enum ClientStatus {
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
pub enum PhysicsSimulation {
    Running,
    Paused,
}

impl ComputedStates for PhysicsSimulation {
    type SourceStates = (GameplayFocus, SessionType, ServerVisibility);

    fn compute(
        (in_game_mode, game_mode, server_visibility): (
            GameplayFocus,
            SessionType,
            ServerVisibility,
        ),
    ) -> Option<Self> {
        // Wenn wir nicht "InGame" sind (also kein GameplayFocus existiert), ist die Simulation irrelevant oder pausiert.
        // Wir geben hier einfach None zurück oder Paused, je nach gewünschtem Verhalten beim State-Wechsel.
        // Bevy Computed States werden nur berechnet, wenn sich die Source States ändern.
        // Wenn eine Source None ist (weil der SuperState nicht aktiv ist), können wir oft auch None zurückgeben.
        match in_game_mode {
            GameplayFocus::Playing => Some(PhysicsSimulation::Running),
            GameplayFocus::GameMenu => {
                match game_mode {
                    SessionType::Client => {
                        // Client läuft im Multiplayer immer weiter, auch im Menü
                        Some(PhysicsSimulation::Running)
                    }
                    SessionType::Singleplayer => {
                        match server_visibility {
                            // Im lokalen Singleplayer pausiert das Menü das Spiel
                            ServerVisibility::Private => Some(PhysicsSimulation::Paused),
                            // Wenn der Server öffentlich ist, läuft das Spiel weiter (wie Multiplayer)
                            _ => Some(PhysicsSimulation::Running),
                        }
                    }
                }
            }
        }
    }
}
