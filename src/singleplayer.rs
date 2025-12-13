use {
    crate::{
        local::*,
        states::{AppScope, ChangeAppScope, SetSingleplayerStatus, SingleplayerState},
    },
    aeronet::io::{connection::Disconnect, server::Close},
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::prelude::*,
};

pub struct SingleplayerLogicPlugin;

impl Plugin for SingleplayerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChannelIoPlugin)
            .add_systems(
                OnEnter(SingleplayerState::Starting),
                on_singleplayer_starting,
            )
            .add_observer(on_singleplayer_ready)
            .add_systems(OnEnter(SingleplayerState::Running), on_singleplayer_running)
            .add_systems(
                Update,
                singleplayer_stopping.run_if(in_state(SingleplayerState::Stopping)),
            );
    }
}

pub fn on_singleplayer_starting(mut commands: Commands) {
    info!("Starting Singleplayer");

    let server_entity = commands
        .spawn((Name::new("Local Server"), LocalSession, LocalServer))
        .id();
    let client_entity = commands
        .spawn((Name::new("Local Client"), LocalSession, LocalClient))
        .id();

    commands.queue(ChannelIo::open(server_entity, client_entity));
}

pub fn on_singleplayer_ready(_: On<Add, LocalClient>, mut commands: Commands) {
    commands.trigger(SetSingleplayerStatus {
        transition: SingleplayerState::Running,
    });
    info!("Singleplayer is ready");
}

pub fn on_singleplayer_running(mut _commands: Commands) {
    debug!("Singleplayer is running");
}

pub fn singleplayer_stopping(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    local_client_query: Query<Entity, With<LocalClient>>,
    local_bot_query: Query<Entity, With<LocalBot>>,
    local_server_query: Query<Entity, With<LocalServer>>,
) {
    // TODO: Save world state
    // after saving, we can disconnect clients

    // first tick clients will be disconnected
    if !client_query.is_empty() {
        for client in &client_query {
            commands.trigger(Disconnect::new(client, "Singleplayer closing"));
        }
        return;
    }
    // second tick server will be closed
    if let Ok(server_entity) = server_query.single() {
        commands.trigger(Close::new(server_entity, "Singleplayer closing"));
        return;
    }
    // third tick bots will be despawned
    if !local_bot_query.is_empty() {
        for bot in &local_bot_query {
            if let Ok(mut bot_entity) = commands.get_entity(bot) {
                bot_entity.despawn();
            }
        }
        return;
    }
    // fourth tick client will be despawned
    if let Ok(client_entity) = local_client_query.single() {
        if let Ok(mut client_entity) = commands.get_entity(client_entity) {
            client_entity.despawn();
        }
        return;
    }
    // fifth tick server will be despawned
    if let Ok(server_entity) = local_server_query.single() {
        if let Ok(mut server_entity) = commands.get_entity(server_entity) {
            server_entity.despawn();
        }
    }
    // sixth tick request Main Menu
    commands.trigger(ChangeAppScope {
        transition: AppScope::Menu,
    });
}

#[cfg(test)]
mod tests {
    use std::thread::current;

    use super::*;
    use crate::{
        states::{
            GameMenuScreen, GameMode, InGameMode, MenuScreen, NavigateGameMenu,
            SingleplayerMenuScreen,
        },
        ChangeGameMode, FOSServerPlugin,
    };

    /// Helper function to setup the app with necessary plugins and states
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::input::InputPlugin,
            bevy::state::app::StatesPlugin,
            FOSServerPlugin,
        ));
        app
    }

    fn set_in_new_game_state(mut app: App) -> App {
        // NOTE: FOSServerPlugin already initializes states via StatesPlugin.

        println!(
            "Initial State: {:?}",
            app.world().resource::<State<AppScope>>()
        );

        // 1. Trigger transition to Singleplayer via ChangeGameMode
        // This emulates clicking "Start Game" in the menu.
        // The observer `on_game_mode_event` in states.rs handles:
        // - Setting AppScope::InGame
        // - Setting GameMode::Singleplayer
        // - Setting SingleplayerState::Starting
        //
        // However, `on_game_mode_event` checks if we come from a valid Menu state.
        // So we must ensure we are in AppScope::Menu (default) and a valid sub-menu state.

        // Emulate being in the "New Game" menu
        app.world_mut()
            .resource_mut::<NextState<MenuScreen>>()
            .set(MenuScreen::Singleplayer);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<SingleplayerMenuScreen>>()
            .set(SingleplayerMenuScreen::NewGame);
        app.update();

        println!(
            "Pre-Trigger State: AppScope={:?}, Menu={:?}, SingleMenu={:?}",
            app.world().resource::<State<AppScope>>(),
            app.world().resource::<State<MenuScreen>>(),
            app.world().resource::<State<SingleplayerMenuScreen>>()
        );
        app
    }

    fn set_singleplayer_running(mut app: App) -> App {
        app.world_mut().trigger(ChangeGameMode {
            transition: GameMode::Singleplayer,
        });

        app.world_mut()
            .resource_mut::<NextState<InGameMode>>()
            .set(InGameMode::Playing);

        app.update();
        app.update();
        app
    }

    fn set_singleplayer_stopping(mut app: App) -> App {
        app.world_mut().trigger(SetSingleplayerStatus {
            transition: SingleplayerState::Stopping,
        });

        app.update();
        app.update();
        app
    }

    #[test]
    fn test_singleplayer_startup() {
        let mut app = setup_app();

        app = set_in_new_game_state(app);
        app = set_singleplayer_running(app);

        println!("Post-Update State:");
        println!(
            "  AppScope: {:?}",
            app.world().resource::<State<AppScope>>()
        );
        if let Some(gm) = app.world().get_resource::<State<GameMode>>() {
            println!("  GameMode: {:?}", gm);
        }
        if let Some(sp) = app.world().get_resource::<State<SingleplayerState>>() {
            println!("  SingleplayerState: {:?}", sp);
        }

        // Verify entities are spawned
        assert!(
            app.world_mut()
                .query::<&LocalServer>()
                .iter(app.world())
                .next()
                .is_some(),
            "LocalServer should be spawned"
        );
        assert!(
            app.world_mut()
                .query::<&LocalClient>()
                .iter(app.world())
                .next()
                .is_some(),
            "LocalClient should be spawned"
        );
    }

    #[test]
    fn test_singleplayer_ready_transition() {
        let mut app = setup_app();

        app = set_in_new_game_state(app);
        app = set_singleplayer_running(app);

        // Check if state requested transition to Running
        let state = app.world().resource::<State<SingleplayerState>>();
        match state.get() {
            SingleplayerState::Running => assert!(true),
            _ => panic!("Expected SingleplayerState to be Running"),
        }
    }

    #[test]
    fn test_singleplayer_stopping_sequence() {
        let mut app = setup_app();

        app = set_in_new_game_state(app);
        app = set_singleplayer_running(app);

        app.update(); // Process event
        app = set_singleplayer_stopping(app);

        let next_scope = app.world().resource::<NextState<AppScope>>();
        match next_scope {
            NextState::Pending(scope) => assert_eq!(*scope, AppScope::Menu),
            _ => panic!("Expected AppScope to be pending transition to Menu"),
        }
    }

    #[test]
    fn test_game_menu_toggle() {
        let mut app = setup_app();
        app = set_in_new_game_state(app);
        app = set_singleplayer_running(app);

        // Verify Playing
        assert_eq!(
            *app.world().resource::<State<InGameMode>>().get(),
            InGameMode::Playing
        );

        app.update();
        app.update();

        // === SIMULATE TOGGLE 1: Direkt NextState setzen ===
        app.world_mut()
            .resource_mut::<NextState<InGameMode>>()
            .set(InGameMode::GameMenu);
        app.update(); // State wechselt
        assert_eq!(
            *app.world().resource::<State<InGameMode>>().get(),
            InGameMode::GameMenu
        );

        // === SIMULATE TOGGLE 2: Direkt NextState setzen ===
        app.world_mut()
            .resource_mut::<NextState<InGameMode>>()
            .set(InGameMode::Playing);
        app.update();
        assert_eq!(
            *app.world().resource::<State<InGameMode>>().get(),
            InGameMode::Playing
        );
    }

    #[test]
    fn test_singleplayer_exit_from_menu() {
        let mut app = setup_app();
        app = set_in_new_game_state(app);
        app = set_singleplayer_running(app);

        // Open the menu → DIREKT NextState!
        app.world_mut()
            .resource_mut::<NextState<InGameMode>>()
            .set(InGameMode::GameMenu);
        app.update();

        // Trigger Exit (bleibt gleich, funktioniert)
        app.world_mut().trigger(NavigateGameMenu {
            transition: GameMenuScreen::Exit,
        });
        app.update();
        app.update();

        // Rest unverändert...
        let sp_state = app.world().resource::<State<SingleplayerState>>();
        assert_eq!(
            *sp_state.get(),
            SingleplayerState::Stopping,
            "Exit from menu should trigger SingleplayerState::Stopping"
        );

        for _ in 0..10 {
            app.update();
        }

        let app_scope = app.world().resource::<State<AppScope>>();
        assert_eq!(
            *app_scope.get(),
            AppScope::Menu,
            "Should return to AppScope::Menu after stopping sequence"
        );
    }
}
