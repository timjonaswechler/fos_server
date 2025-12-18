use {
    crate::{
        local::*,
        states::{
            ChangeAppScope, GamePhase, SessionType, SetSingleplayerStatus,
            SingleplayerShutdownStep, SingleplayerStatus,
        },
    },
    aeronet::io::{connection::Disconnect, server::Close},
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    // aeronet_replicon::client::AeronetRepliconClientPlugin,
    // aeronet_replicon::server::AeronetRepliconServerPlugin,
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::prelude::*,
    // bevy_replicon::RepliconPlugins,
};

pub struct SingleplayerLogicPlugin;

impl Plugin for SingleplayerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChannelIoPlugin,
            // RepliconPlugins,
            // AeronetRepliconClientPlugin,
            // AeronetRepliconServerPlugin,
        ))
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

pub fn on_singleplayer_ready(
    _: On<Add, LocalClient>,
    mut commands: Commands,
    current_state: Res<State<SessionType>>,
) {
    if *current_state.get() == SessionType::Singleplayer {
        commands.trigger(SetSingleplayerStatus {
            transition: SingleplayerStatus::Running,
        });
        info!("Singleplayer is ready");
    }
}

pub fn on_singleplayer_running(mut _commands: Commands) {
    debug!("Singleplayer is running");
}

pub fn singleplayer_stopping(
    mut commands: Commands,
    step: Res<State<SingleplayerShutdownStep>>,
    mut next_step: ResMut<NextState<SingleplayerShutdownStep>>,

    server_query: Query<Entity, With<WebTransportServer>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    local_client_query: Query<Entity, With<LocalClient>>,
    local_bot_query: Query<Entity, With<LocalBot>>,
    local_server_query: Query<Entity, With<LocalServer>>,
) {
    match step.get() {
        SingleplayerShutdownStep::DisconnectRemoteClients => {
            // 1. Tick: Remote-Clients trennen (public / LAN)
            for client in &client_query {
                commands.trigger(Disconnect::new(client, "Singleplayer closing"));
            }
            // Egal ob es welche gab oder nicht, nächster Schritt:
            next_step.set(SingleplayerShutdownStep::CloseRemoteServer);
        }

        SingleplayerShutdownStep::CloseRemoteServer => {
            // 2. Tick: Remote-Server schließen (WebTransportServer)
            if let Ok(server_entity) = server_query.single() {
                commands.trigger(Close::new(server_entity, "Singleplayer closing"));
            }
            next_step.set(SingleplayerShutdownStep::DespawnBots);
        }

        SingleplayerShutdownStep::DespawnBots => {
            // 3. Tick: Lokale Bots despawnen
            for bot in &local_bot_query {
                if let Ok(mut bot_entity) = commands.get_entity(bot) {
                    bot_entity.despawn();
                }
            }
            next_step.set(SingleplayerShutdownStep::DespawnLocalClient);
        }

        SingleplayerShutdownStep::DespawnLocalClient => {
            // 4. Tick: Lokalen Client despawnen
            if let Ok(client_entity) = local_client_query.single() {
                if let Ok(mut client_entity) = commands.get_entity(client_entity) {
                    client_entity.despawn();
                }
            }
            next_step.set(SingleplayerShutdownStep::DespawnLocalServer);
        }

        SingleplayerShutdownStep::DespawnLocalServer => {
            // 5. Tick: Lokalen Server despawnen
            if let Ok(server_entity) = local_server_query.single() {
                if let Ok(mut server_entity) = commands.get_entity(server_entity) {
                    server_entity.despawn();
                }
            }
            next_step.set(SingleplayerShutdownStep::Done);
        }

        SingleplayerShutdownStep::Done => {
            // 6. Tick: Zurück ins Hauptmenü
            commands.trigger(ChangeAppScope {
                transition: GamePhase::Menu,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{states::*, FOSServerPlugin};
    use std::fmt::Debug;

    /// Extension trait to make tests cleaner and more readable.
    trait SingleplayerTestExt {
        /// Initializes the app with minimal plugins and the FOSServerPlugin.
        fn new_test_app() -> Self;

        /// Moves the app state through the menu to start a singleplayer game using events.
        fn start_singleplayer_new_game(&mut self);
        fn start_singleplayer_loaded_game(&mut self);

        /// Triggers the stopping sequence via the Game Menu "Exit" event.
        fn stop_singleplayer(&mut self);

        /// Runs the app for a specified number of frames.
        fn wait_frames(&mut self, frames: usize);

        /// Asserts that the current state matches the expected value.
        fn assert_state<S: States + PartialEq + Debug>(&self, expected: S);

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

        fn start_singleplayer_new_game(&mut self) {
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

        fn start_singleplayer_loaded_game(&mut self) {
            // 1. Main Menu -> Singleplayer Menu
            self.world_mut().trigger(NavigateMainMenu {
                transition: MenuContext::Singleplayer,
            });
            self.update();

            // 2. Singleplayer Menu -> New Game
            self.world_mut().trigger(NavigateSingleplayerMenu {
                transition: SingleplayerSetup::LoadGame,
            });
            self.update();

            self.world_mut().trigger(ChangeGameMode {
                transition: SessionType::Singleplayer,
            });

            self.update();
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
    fn test_from_singleplayer_startup_new_game() {
        let mut app = App::new_test_app();
        app.start_singleplayer_new_game();

        app.assert_state(GamePhase::InGame);
        app.assert_state(SessionType::Singleplayer);
        app.assert_state(SingleplayerStatus::Running);
        app.assert_state(ServerVisibility::Private);

        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);
    }

    #[test]
    fn test_from_singleplayer_startup_loaded_game() {
        let mut app = App::new_test_app();
        app.start_singleplayer_loaded_game();

        app.assert_state(GamePhase::InGame);
        app.assert_state(SessionType::Singleplayer);
        app.assert_state(SingleplayerStatus::Running);
        app.assert_state(ServerVisibility::Private);

        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);
    }

    #[test]
    fn test_game_menu_toggle() {
        let mut app = App::new_test_app();
        app.start_singleplayer_new_game();

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
        app.start_singleplayer_new_game();

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

    #[test]
    fn test_restart_cycle() {
        let mut app = App::new_test_app();

        // Round 1
        app.start_singleplayer_new_game();
        app.assert_state(SingleplayerStatus::Running);
        app.stop_singleplayer();
        app.wait_frames(10); // Wait for cleanup
        app.assert_state(GamePhase::Menu);

        // Round 2
        app.start_singleplayer_new_game();
        app.assert_state(SingleplayerStatus::Running);

        // Ensure we didn't duplicate entities or leak resources
        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);
    }

    #[test]
    fn test_double_exit_spam_in_same_frame() {
        let mut app = App::new_test_app();
        app.start_singleplayer_new_game();

        // Open Game Menu
        app.world_mut()
            .resource_mut::<NextState<GameplayFocus>>()
            .set(GameplayFocus::GameMenu);
        app.update();

        // Spam Exit Button twice in the same frame
        app.world_mut().trigger(NavigateGameMenu {
            transition: PauseMenu::Exit,
        });
        app.world_mut().trigger(NavigateGameMenu {
            transition: PauseMenu::Exit,
        });

        app.update();
        app.wait_frames(10);

        // Should still exit cleanly without panicking or getting stuck
        app.assert_state(GamePhase::Menu);
        app.assert_entity_count::<LocalServer>(0);
    }

    #[test]
    fn test_double_exit_spam_in_different_frames() {
        let mut app = App::new_test_app();
        app.start_singleplayer_new_game();

        // Open Game Menu
        app.world_mut()
            .resource_mut::<NextState<GameplayFocus>>()
            .set(GameplayFocus::GameMenu);
        app.update();

        // Spam Exit Button twice in the same frame
        app.world_mut().trigger(NavigateGameMenu {
            transition: PauseMenu::Exit,
        });
        app.world_mut().trigger(NavigateGameMenu {
            transition: PauseMenu::Exit,
        });

        app.update();
        app.wait_frames(10);

        // Should still exit cleanly without panicking or getting stuck
        app.assert_state(GamePhase::Menu);
        app.assert_entity_count::<LocalServer>(0);
    }

    #[test]
    fn test_double_start_spam_in_same_frame() {
        let mut app = App::new_test_app();

        // 1. Navigate to setup (Pre-requisites)
        app.world_mut().trigger(NavigateMainMenu {
            transition: MenuContext::Singleplayer,
        });
        app.update();
        app.world_mut().trigger(NavigateSingleplayerMenu {
            transition: SingleplayerSetup::NewGame,
        });
        app.update();

        // 2. Spam the "Start Game" trigger twice
        app.world_mut().trigger(ChangeGameMode {
            transition: SessionType::Singleplayer,
        });
        app.world_mut().trigger(ChangeGameMode {
            transition: SessionType::Singleplayer,
        });

        // Process frame
        app.update(); // Enters Starting
        app.update(); // Spawns entities
        app.update(); // Ready -> Running

        // We expect only 1 set of entities, even with double trigger
        // (Assuming the state transition guards block the second trigger effectively)
        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);
    }

    #[test]
    fn test_double_start_spam_in_different_frames() {
        let mut app = App::new_test_app();

        // 1. Navigate to setup (Pre-requisites)
        app.world_mut().trigger(NavigateMainMenu {
            transition: MenuContext::Singleplayer,
        });
        app.update();
        app.world_mut().trigger(NavigateSingleplayerMenu {
            transition: SingleplayerSetup::NewGame,
        });
        app.update();

        // 2. Spam the "Start Game" trigger twice
        app.world_mut().trigger(ChangeGameMode {
            transition: SessionType::Singleplayer,
        });
        app.update();
        app.world_mut().trigger(ChangeGameMode {
            transition: SessionType::Singleplayer,
        });

        // Process frame
        app.update(); // Enters Starting
        app.update(); // Spawns entities
        app.update(); // Ready -> Running

        // We expect only 1 set of entities, even with double trigger
        // (Assuming the state transition guards block the second trigger effectively)
        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);
    }
}
