use {
    crate::states::{ServerVisibility, SetServerVisibility, SingleplayerStatus},
    aeronet::io::{
        connection::Disconnect,
        server::{Close, Closed, Server, ServerEndpoint},
    },
    // aeronet_replicon::client::AeronetRepliconClient,
    // aeronet_replicon::server::AeronetRepliconServer,
    aeronet_webtransport::{
        cert,
        server::{
            SessionRequest, WebTransportServer, WebTransportServerClient, WebTransportServerPlugin,
        },
    },
    bevy::prelude::*,
    core::time::Duration,
    helpers::DiscoveryServerPlugin,
};

pub struct ServerLogicPlugin;

impl Plugin for ServerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WebTransportServerPlugin, DiscoveryServerPlugin))
            .add_systems(
                Update,
                server_pending_going_public.run_if(in_state(ServerVisibility::PendingPublic)),
            )
            .add_systems(
                OnEnter(ServerVisibility::GoingPublic),
                on_server_going_public,
            )
            .add_systems(OnEnter(ServerVisibility::Public), on_server_is_running)
            .add_systems(
                Update,
                server_is_running.run_if(in_state(ServerVisibility::Public)),
            )
            .add_systems(
                OnEnter(ServerVisibility::GoingPrivate),
                on_server_going_private,
            )
            .add_observer(on_check_is_server_private)
            .add_observer(on_server_session_request)
            .add_observer(on_server_is_public);
    }
}

pub fn server_pending_going_public(
    mut commands: Commands,
    singleplayer_state: Res<State<SingleplayerStatus>>,
) {
    if *singleplayer_state.get() == SingleplayerStatus::Running {
        {
            info!("Singleplayer Running detected, requesting Public transition.");
            commands.trigger(SetServerVisibility {
                transition: ServerVisibility::GoingPublic,
            });
        }
    }
}

pub fn on_server_going_public(
    mut commands: Commands,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    next_state.set(ServerVisibility::GoingPublic);
    // TODO: implement error if server cant get started
    // TODO: Implement User interface infos for server
    // TODO: Implement Port usage detection
    let identity = aeronet_webtransport::wtransport::Identity::self_signed([
        "localsingleplayer",
        "127.0.0.1",
        "::1",
        // IPv6
    ])
    .expect("all given SANs should be valid DNS names");
    let cert = &identity.certificate_chain().as_slice()[0];
    let _spki_fingerprint =
        cert::spki_fingerprint_b64(cert).expect("should be a valid certificate");
    let _cert_hash = cert::hash_to_b64(cert.hash());
    println!("************************");
    println!("SPKI FINGERPRINT");
    println!("  {{spki_fingerprint}}");
    println!("CERTIFICATE HASH");
    println!("  {{cert_hash}}");
    println!("************************");

    let config = aeronet_webtransport::wtransport::ServerConfig::builder()
        .with_bind_default(25571)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(1)))
        .max_idle_timeout(Some(Duration::from_secs(5)))
        .expect("should be a valid idle timeout")
        .build();

    commands
        .spawn(Name::new("WebTransportServer"))
        .queue(WebTransportServer::open(config));
}

pub fn on_server_is_public(
    event: On<Add, Server>,
    // mut commands: Commands,
    roots: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    match roots.single() {
        Ok(root) => {
            if root == event.entity {
                // commands.entity(root).insert(AeronetRepliconServer);
                println!("WebTransport server is fully opened");
                next_state.set(ServerVisibility::Public);
                return;
            }
        }
        Err(_) => {
            // Handle error case
            // multiply or None Server entity was found
        }
    }
}

pub fn on_server_is_running(_: Commands) {
    info!("WebTransport Server is running");
}

pub fn server_is_running(
    _commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    if server_query.is_empty() {
        {
            next_state.set(ServerVisibility::GoingPrivate);
            return;
        }
    }
}

pub fn on_server_going_private(
    mut commands: Commands,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    info!(
        "Server going down\n Still {} clients connected\n Servers: {} active",
        client_query.iter().count(),
        server_query.iter().count()
    );
    if !client_query.is_empty() {
        {
            info!("Disconnect all clients");
            for client in client_query.iter() {
                {
                    commands.trigger(Disconnect::new(client, "Server closing"));
                }
            }
            return;
        }
    }
    if let Ok(server) = server_query.single() {
        {
            info!("Close server");
            commands.trigger(Close::new(server, "Server closing"));
            return;
        }
    }
    if client_query.is_empty() && server_query.is_empty() {
        {
            info!("Server is down");
            next_state.set(ServerVisibility::Private);
        }
    }
}

pub fn check_is_server_private(
    mut commands: Commands,
    server_query: Query<Entity, (With<Server>, With<ServerEndpoint>)>,
) {
    if server_query.is_empty() {
        {
            info!("Closed is triggered");
            commands.trigger(SetServerVisibility {
                transition: ServerVisibility::Private,
            });
        }
    }
}

pub fn on_check_is_server_private(_: On<Closed>, mut commands: Commands) {
    info!("Closed is triggered");
    commands.trigger(SetServerVisibility {
        transition: ServerVisibility::Private,
    });
}

pub fn on_server_session_request(trigger: On<SessionRequest>, clients: Query<&ChildOf>) {
    let client = trigger.event_target();
    let Ok(&ChildOf(server)) = clients.get(client) else {
        {
            return;
        }
    };

    helpers::handle_server_accept_connection(client, server, trigger);
}

pub fn on_server_client_timeout() {
    todo!("Implement on_server_client_timeout")
}

pub fn on_server_client_lost() {
    todo!("Implement on_server_client_lost")
}

pub fn on_server_client_graceful_disconnect() {
    todo!("Implement on_server_client_graceful_disconnect")
}

pub fn on_server_shutdown_notify_clients() {
    todo!("Implement on_server_shutdown_notify_clients")
}

pub mod helpers {
    use {
        crate::states::ServerVisibility,
        aeronet_webtransport::server::{SessionRequest, SessionResponse},
        bevy::prelude::*,
        std::net::UdpSocket,
    };

    pub(super) fn handle_server_accept_connection(
        client: Entity,
        server: Entity,
        mut trigger: On<SessionRequest>,
    ) {
        info!("{client} connecting to {server} with headers:");
        for (header_key, header_value) in &trigger.headers {
            info!("  {header_key}: {header_value}");
        }

        trigger.respond(SessionResponse::Accepted);
    }

    pub(super) fn _handle_server_reject_connection() {
        // TODO: client UUID or Name is on the server's blacklist
        // TODO: Server password is incorrect
        // TODO: Server is full
        todo!("Implement on_server_shutdown_notify_clients")
    }

    pub mod ports {
        use std::net::{TcpListener, UdpSocket};

        pub(in crate::server) fn _is_server_port_available(port: u16) -> bool {
            // UDP (für Discovery oder QUIC-ähnliches)
            if UdpSocket::bind(("0.0.0.0", port)).is_err() {
                return false;
            }
            true
        }

        pub(in crate::server) fn _bind_test(port: u16) -> bool {
            TcpListener::bind(("0.0.0.0", port)).is_ok()
        }

        pub fn find_free_port() -> Option<u16> {
            // OS gibt freien Port, wenn 0 gebunden wird
            TcpListener::bind(("127.0.0.1", 0))
                .ok()
                .and_then(|sock| sock.local_addr().ok())
                .map(|addr| addr.port())
        }

        pub fn validate_port_range() {
            // TODO: Validate port range
            todo!("Implement validate_port_range")
        }
    }

    pub const DISCOVERY_PORT: u16 = 30000;
    pub const GAME_PORT: u16 = 25571;
    pub const MAGIC: &[u8] = b"FORGE_DISCOVER_V1";

    #[derive(Resource)]
    struct DiscoverySocket(UdpSocket);

    pub struct DiscoveryServerPlugin;

    impl Plugin for DiscoveryServerPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(ServerVisibility::Public), insert_discovery_socket);
            app.add_systems(OnExit(ServerVisibility::Public), remove_discovery_socket);

            app.add_systems(
                Update,
                discovery_server_system.run_if(in_state(ServerVisibility::Public)),
            );
        }
    }

    fn insert_discovery_socket(mut commands: Commands) {
        commands.insert_resource(setup_discovery_socket());
    }

    fn remove_discovery_socket(mut commands: Commands) {
        commands.remove_resource::<DiscoverySocket>();
    }

    fn setup_discovery_socket() -> DiscoverySocket {
        let socket =
            UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT)).expect("failed to bind discovery socket");
        socket
            .set_broadcast(true)
            .expect("failed to enable broadcast");
        socket
            .set_nonblocking(true)
            .expect("failed to set nonblocking");
        DiscoverySocket(socket)
    }

    fn discovery_server_system(socket: Res<DiscoverySocket>) {
        let mut buf = [0u8; 256];
        // alle eingehenden Pakete abarbeiten
        while let Ok((len, src)) = socket.0.recv_from(&mut buf) {
            if &buf[..len] == MAGIC {
                // minimale Antwort: Magic + Port
                let resp = format!("FORGE_RESP_V1;{}", GAME_PORT);
                let _ = socket.0.send_to(resp.as_bytes(), src);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{local::*, states::*, FOSServerPlugin};
    use std::fmt::Debug;

    /// Extension trait to make tests cleaner and more readable.
    trait ServerVisibilityTestExt {
        /// Initializes the app with minimal plugins and the FOSServerPlugin.
        fn new_test_app() -> Self;

        /// Moves the app state through the menu to start a singleplayer game using events.
        fn start_singleplayer_new_game(&mut self);
        fn start_singleplayer_loaded_game(&mut self);
        fn start_singleplayer_host_new_game(&mut self);
        fn start_singleplayer_host_saved_game(&mut self);

        /// Triggers the stopping sequence via the Game Menu "Exit" event.
        fn stop_singleplayer(&mut self);

        /// Runs the app for a specified number of frames.
        fn wait_frames(&mut self, frames: usize);

        /// Asserts that the current state matches the expected value.
        fn assert_state<S: States + PartialEq + Debug>(&self, expected: S);

        /// Asserts that a specific component type has exactly `count` instances in the world.
        fn assert_entity_count<C: Component>(&mut self, count: usize);

        fn toggle_game_menu(&mut self);
    }

    impl ServerVisibilityTestExt for App {
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

            self.world_mut().trigger(ChangeGameMode {
                transition: SessionType::Singleplayer,
            });
            self.update();
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

        fn start_singleplayer_host_new_game(&mut self) {
            // 1. Main Menu -> Singleplayer Menu
            self.world_mut().trigger(NavigateMainMenu {
                transition: MenuContext::Multiplayer,
            });
            self.update();

            // 2. Singleplayer Menu -> New Game
            self.world_mut().trigger(NavigateMultiplayerMenu {
                transition: MultiplayerSetup::HostNewGame,
            });
            self.update();

            self.world_mut().trigger(ChangeGameMode {
                transition: SessionType::Singleplayer,
            });

            self.update();
            self.update();
            self.update();
        }

        fn start_singleplayer_host_saved_game(&mut self) {
            self.world_mut().trigger(NavigateMainMenu {
                transition: MenuContext::Multiplayer,
            });
            self.update();

            // 2. Singleplayer Menu -> New Game
            self.world_mut().trigger(NavigateMultiplayerMenu {
                transition: MultiplayerSetup::HostSavedGame,
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

        fn toggle_game_menu(&mut self) {
            let current_focus = {
                let current_state = self.world().resource::<State<GameplayFocus>>();
                current_state.get().clone()
            };
            let mut next = self.world_mut().resource_mut::<NextState<GameplayFocus>>();
            match current_focus {
                GameplayFocus::Playing => next.set(GameplayFocus::GameMenu),
                GameplayFocus::GameMenu => next.set(GameplayFocus::Playing),
            }
        }
    }

    #[test]
    fn server_lifecycle_over_game_menu_after_new_game_in_singleplayer_is_started() {
        let mut app = App::new_test_app();
        app.start_singleplayer_new_game();

        app.assert_state(GamePhase::InGame);
        app.assert_state(SessionType::Singleplayer);
        app.assert_state(SingleplayerStatus::Running);
        app.assert_state(ServerVisibility::Private);
        app.assert_state(GameplayFocus::Playing);
        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);

        // server going public sequence
        app.toggle_game_menu();
        app.update();
        app.assert_state(GameplayFocus::GameMenu);
        app.world_mut().trigger(SetServerVisibility {
            transition: ServerVisibility::GoingPublic,
        });
        app.toggle_game_menu();
        app.update();
        app.assert_state(GameplayFocus::Playing);

        app.wait_frames(5);
        println!("waited 5 frames");
        app.assert_entity_count::<WebTransportServer>(1);
        app.assert_entity_count::<ServerEndpoint>(1);
        app.assert_entity_count::<Server>(1);
        app.assert_state(ServerVisibility::Public);
        // Server runs at the moment

        // closing sequnce starts
        app.toggle_game_menu();
        app.update();
        app.assert_state(GameplayFocus::GameMenu);
        app.world_mut().trigger(NavigateGameMenu {
            transition: PauseMenu::Exit,
        });
        app.toggle_game_menu();
        app.update();
        app.assert_state(SingleplayerStatus::Stopping);

        // at the moemnt there are 6 steps in the Singleplayer stopping phase
        app.wait_frames(6);
        app.assert_state(GamePhase::Menu);
        app.assert_entity_count::<WebTransportServer>(0);
        app.assert_entity_count::<ServerEndpoint>(0);
        app.assert_entity_count::<Server>(0);
    }

    #[test]
    fn server_lifecycle_over_game_menu_after_saved_game_in_singleplayer_is_started() {
        let mut app = App::new_test_app();
        app.start_singleplayer_loaded_game();

        app.assert_state(GamePhase::InGame);
        app.assert_state(SessionType::Singleplayer);
        app.assert_state(SingleplayerStatus::Running);
        app.assert_state(ServerVisibility::Private);
        app.assert_state(GameplayFocus::Playing);
        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);

        // server going public sequence
        app.toggle_game_menu();
        app.update();
        app.assert_state(GameplayFocus::GameMenu);
        app.world_mut().trigger(SetServerVisibility {
            transition: ServerVisibility::GoingPublic,
        });
        app.toggle_game_menu();
        app.update();
        app.assert_state(GameplayFocus::Playing);

        app.wait_frames(5);
        app.assert_entity_count::<WebTransportServer>(1);
        app.assert_entity_count::<ServerEndpoint>(1);
        app.assert_entity_count::<Server>(1);
        println!("waited 5 frames");

        app.assert_state(ServerVisibility::Public);

        // Server runs at the moment

        // closing sequnce starts
        app.toggle_game_menu();
        app.update();
        app.assert_state(GameplayFocus::GameMenu);
        app.world_mut().trigger(NavigateGameMenu {
            transition: PauseMenu::Exit,
        });
        app.toggle_game_menu();
        app.update();
        app.assert_state(SingleplayerStatus::Stopping);

        // at the moemnt there are 6 steps in the Singleplayer stopping phase
        app.wait_frames(6);
        app.assert_state(GamePhase::Menu);
        app.assert_entity_count::<WebTransportServer>(0);
        app.assert_entity_count::<ServerEndpoint>(0);
        app.assert_entity_count::<Server>(0);
    }

    #[test]
    fn server_start_with_new_hosted_game() {
        let mut app = App::new_test_app();
        app.start_singleplayer_host_new_game();

        app.assert_state(GamePhase::InGame);
        app.assert_state(SessionType::Singleplayer);
        app.assert_state(SingleplayerStatus::Running);
        app.assert_state(ServerVisibility::GoingPublic);

        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);

        app.wait_frames(3);

        app.assert_state(ServerVisibility::Public);
        app.assert_entity_count::<WebTransportServer>(1);
    }

    #[test]
    fn server_start_with_loaded_hosted_game() {
        let mut app = App::new_test_app();
        app.start_singleplayer_host_saved_game();

        app.assert_state(GamePhase::InGame);
        app.assert_state(SessionType::Singleplayer);
        app.assert_state(SingleplayerStatus::Running);
        app.assert_state(ServerVisibility::GoingPublic);

        app.assert_entity_count::<LocalServer>(1);
        app.assert_entity_count::<LocalClient>(1);

        app.wait_frames(3);

        app.assert_state(ServerVisibility::Public);
        app.assert_entity_count::<WebTransportServer>(1);
    }
}
