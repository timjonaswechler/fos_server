use {
    crate::states::{ServerVisibilityEvent, ServerVisibilityState, SingleplayerState},
    aeronet_io::{
        connection::Disconnect,
        server::{Close, Server, ServerEndpoint},
    },
    aeronet_webtransport::{
        cert,
        server::{SessionRequest, WebTransportServer, WebTransportServerClient},
    },
    bevy::prelude::*,
    core::time::Duration,
    helpers::DiscoveryServerPlugin,
};

pub struct ServerLogicPlugin;

impl Plugin for ServerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DiscoveryServerPlugin)
            .add_systems(
                Update,
                server_pending_going_public.run_if(in_state(ServerVisibilityState::PendingPublic)),
            )
            .add_systems(
                OnEnter(ServerVisibilityState::GoingPublic),
                on_server_going_public,
            )
            .add_systems(
                Update,
                check_is_server_public.run_if(in_state(ServerVisibilityState::GoingPublic)),
            )
            .add_systems(OnEnter(ServerVisibilityState::Public), on_server_is_running)
            .add_systems(
                Update,
                server_is_running.run_if(in_state(ServerVisibilityState::Public)),
            )
            .add_systems(
                OnEnter(ServerVisibilityState::GoingPrivate),
                on_server_going_private,
            )
            .add_systems(
                Update,
                check_is_server_private.run_if(in_state(ServerVisibilityState::GoingPrivate)),
            )
            .add_observer(on_server_session_request);
    }
}

pub fn server_pending_going_public(
    mut commands: Commands,
    singleplayer_state: Res<State<SingleplayerState>>,
) {
    if *singleplayer_state.get() == SingleplayerState::Running {
        {
            info!("Singleplayer Running detected, requesting Public transition.");
            commands.trigger(ServerVisibilityEvent {
                transition: ServerVisibilityState::GoingPublic,
            });
        }
    }
}

pub fn on_server_going_public(
    mut commands: Commands,
    mut next_state: ResMut<NextState<ServerVisibilityState>>,
) {
    next_state.set(ServerVisibilityState::GoingPublic);
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
    info!("************************");
    info!("SPKI FINGERPRINT");
    info!("  {{spki_fingerprint}}");
    info!("CERTIFICATE HASH");
    info!("  {{cert_hash}}");
    info!("************************");

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

pub fn check_is_server_public(
    _commands: Commands,
    server_query: Query<Entity, (With<Server>, With<ServerEndpoint>)>,
    mut next_state: ResMut<NextState<ServerVisibilityState>>,
) {
    if let Ok(_) = server_query.single() {
        {
            info!("WebTransport Server is ready");
            next_state.set(ServerVisibilityState::Public);
        }
    }
}

pub fn on_server_is_running(_: Commands) {
    info!("WebTransport Server is running");
}

pub fn server_is_running(
    _commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibilityState>>,
) {
    if server_query.is_empty() {
        {
            next_state.set(ServerVisibilityState::GoingPrivate);
            return;
        }
    }
}

pub fn on_server_going_private(
    mut commands: Commands,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibilityState>>,
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
            next_state.set(ServerVisibilityState::Private);
        }
    }
}

pub fn check_is_server_private(
    _commands: Commands,
    server_query: Query<Entity, (With<Server>, With<ServerEndpoint>)>,
) {
    if server_query.is_empty() {
        {
            info!("Closed is triggered");
        }
    }
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
        crate::states::ServerVisibilityState,
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
            app.add_systems(
                OnEnter(ServerVisibilityState::Public),
                insert_discovery_socket,
            );
            app.add_systems(
                OnExit(ServerVisibilityState::Public),
                remove_discovery_socket,
            );

            app.add_systems(
                Update,
                discovery_server_system.run_if(in_state(ServerVisibilityState::Public)),
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
