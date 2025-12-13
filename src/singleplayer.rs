use {
    crate::{
        local::*,
        states::{ChangeAppScope, GamePhase, SetSingleplayerStatus, SingleplayerStatus},
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
                OnEnter(SingleplayerStatus::Starting),
                on_singleplayer_starting,
            )
            .add_observer(on_singleplayer_ready)
            .add_systems(
                OnEnter(SingleplayerStatus::Running),
                on_singleplayer_running,
            )
            .add_systems(
                Update,
                singleplayer_stopping.run_if(in_state(SingleplayerStatus::Stopping)),
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
        transition: SingleplayerStatus::Running,
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
        transition: GamePhase::Menu,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        states::{
            GameplayFocus, MenuContext, NavigateGameMenu, NavigateMainMenu,
            NavigateSingleplayerMenu, PauseMenu, SessionType, SingleplayerSetup,
        },
        ChangeGameMode, FOSServerPlugin,
    };
    use bevy::state::state::FreelyMutableState;
    use std::fmt::Debug;

    /// Extension trait to make tests cleaner and more readable.
    trait SingleplayerTestExt {
        /// Initializes the app with minimal plugins and the FOSServerPlugin.
        fn new_test_app() -> Self;

        /// Moves the app state through the menu to start a singleplayer game using events.
        fn start_singleplayer(&mut self);

        /// Triggers the stopping sequence via the Game Menu "Exit" event.
        fn stop_singleplayer(&mut self);

        /// Runs the app for a specified number of frames.
        fn wait_frames(&mut self, frames: usize);

        /// Asserts that the current state matches the expected value.
        fn assert_state<S: States + PartialEq + Debug>(&self, expected: S);

        /// Asserts that the NextState matches the expected value.
        fn assert_next_state<S: FreelyMutableState + PartialEq + Debug>(&self, expected: S);

        /// Asserts that a specific component type has exactly `count` instances in the world.
        fn assert_entity_count<C: Component>(&mut self, count: usize);
    }

    impl SingleplayerTestExt for App {
        fn new_test_app() -> Self {
            let mut app = App::new();
            app.add_plugins((
                MinimalPlugins,
                bevy::input::InputPlugin,
                bevy::state::app::StatesPlugin,
                FOSServerPlugin,
            ));
            app
        }

        fn start_singleplayer(&mut self) {
            // 1. Main Menu -> Singleplayer Menu
            self.world_mut().trigger(NavigateMainMenu {
                transition: MenuContext::Singleplayer,
            });
            self.update();

            // 2. Singleplayer Menu -> New Game
            self.world_mut().trigger(NavigateSingleplayerMenu {
                transition: SingleplayerSetup::NewGame,
            });
            self.update();

            // 3. New Game -> Start Game (ChangeGameMode)
            // Note: In the real UI, this might be a chain of sub-menus (ConfigPlayer -> ConfigWorld...),
            // but the critical transition to InGame is triggered by ChangeGameMode logic
            // or the final confirmation in `on_singleplayer_new_game_screen_event`.
            // For this integration test, we simulate the final "Start" trigger that the UI would send.

            // To properly simulate the flow as defined in `on_game_mode_event` in states.rs,
            // we need to be in the correct sub-state (NewGame) which we set above.
            self.world_mut().trigger(ChangeGameMode {
                transition: SessionType::Singleplayer,
            });

            // Process the ChangeGameMode event which sets:
            // - GamePhase::InGame
            // - SingleplayerStatus::Starting
            self.update();

            // Process internal transitions (Starting -> Ready -> Running)
            // The `on_singleplayer_starting` system spawns entities, then `on_singleplayer_ready` runs.
            self.update();
            self.update();
        }

        fn stop_singleplayer(&mut self) {
            // To exit, we must be in the Game Menu or able to trigger the exit action.
            // We simulate clicking "Exit" in the pause menu.
            self.world_mut().trigger(NavigateGameMenu {
                transition: PauseMenu::Exit,
            });
            // Initial update to process the trigger
            self.update();
        }

        fn wait_frames(&mut self, frames: usize) {
            for _ in 0..frames {
                self.update();
            }
        }

        fn assert_state<S: States + PartialEq + Debug>(&self, expected: S) {
            let current = self.world().resource::<State<S>>().get();
            assert_eq!(
                current,
                &expected,
                "State mismatch for type {}",
                std::any::type_name::<S>()
            );
        }

        fn assert_next_state<S: FreelyMutableState + PartialEq + Debug>(&self, expected: S) {
            let next = self.world().resource::<NextState<S>>();
            match next {
                NextState::Pending(scope) => assert_eq!(scope, &expected, "NextState mismatch"),
                _ => panic!(
                    "Expected NextState to be Pending({:?}), but it was {:?}",
                    expected, next
                ),
            }
        }

        fn assert_entity_count<C: Component>(&mut self, count: usize) {
            let actual = self.world_mut().query::<&C>().iter(self.world()).len();
            assert_eq!(
                actual,
                count,
                "Entity count mismatch for {}",
                std::any::type_name::<C>()
            );
        }
    }

    #[test]
    fn test_singleplayer_startup() {
        let mut app = App::new_test_app();
        app.start_singleplayer();

        app.assert_state(GamePhase::InGame);
        app.assert_state(SessionType::Singleplayer);
        app.assert_state(SingleplayerStatus::Running);

        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);
    }

    #[test]
    fn test_singleplayer_ready_transition() {
        let mut app = App::new_test_app();
        app.start_singleplayer();

        // Already checked in startup, but explicit here for logic flow
        app.assert_state(SingleplayerStatus::Running);
    }

    #[test]
    fn test_singleplayer_stopping_sequence_full() {
        let mut app = App::new_test_app();
        app.start_singleplayer();

        // Trigger stopping via menu event
        app.stop_singleplayer();

        // Run enough frames for the multi-tick despawn logic to complete
        // (Disconnect -> Close -> Bots -> Client -> Server -> Menu)
        app.wait_frames(10);

        // Verify everything is gone
        app.assert_entity_count::<WebTransportServerClient>(0);
        app.assert_entity_count::<WebTransportServer>(0);
        app.assert_entity_count::<LocalBot>(0);
        app.assert_entity_count::<LocalServer>(0);
        app.assert_entity_count::<LocalClient>(0);

        // Verify return to menu
        // Since we waited multiple frames, the transition should have been applied.
        // So we check the CURRENT state, not NextState.
        app.assert_state(GamePhase::Menu);
    }

    #[test]
    fn test_game_menu_toggle() {
        let mut app = App::new_test_app();
        app.start_singleplayer();

        app.assert_state(GameplayFocus::Playing);

        // --- FIRST TOGGLE: Playing -> GameMenu (Direct NextState set, as in original working code) ---
        app.world_mut()
            .resource_mut::<NextState<GameplayFocus>>()
            .set(GameplayFocus::GameMenu);
        app.update(); // State wechselt
        app.assert_state(GameplayFocus::GameMenu);

        // --- SECOND TOGGLE: GameMenu -> Playing (Direct NextState set, as in original working code) ---
        app.world_mut()
            .resource_mut::<NextState<GameplayFocus>>()
            .set(GameplayFocus::Playing);
        app.update(); // State wechselt
        app.assert_state(GameplayFocus::Playing);
    }

    #[test]
    fn test_singleplayer_exit_from_menu() {
        let mut app = App::new_test_app();
        app.start_singleplayer();

        // Open Game Menu via direct NextState set (as in original working code)
        app.world_mut()
            .resource_mut::<NextState<GameplayFocus>>()
            .set(GameplayFocus::GameMenu);
        app.update();

        // Verify that we are now in GameMenu focus
        app.assert_state(GameplayFocus::GameMenu);

        // Trigger Exit via Event
        app.world_mut().trigger(NavigateGameMenu {
            transition: PauseMenu::Exit,
        });

        // Process trigger
        app.update();

        app.assert_state(SingleplayerStatus::Stopping);

        // Let the stopping sequence run
        app.wait_frames(10);

        app.assert_entity_count::<WebTransportServerClient>(0);
        app.assert_entity_count::<WebTransportServer>(0);
        app.assert_entity_count::<LocalBot>(0);
        app.assert_entity_count::<LocalServer>(0);
        app.assert_entity_count::<LocalClient>(0);

        // Should return to main menu
        app.assert_state(GamePhase::Menu);
    }
}
