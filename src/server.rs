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
};

pub struct ServerLogicPlugin;

impl Plugin for ServerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
            commands.trigger(ServerVisibilityEvent::RequestTransitionTo(
                ServerVisibilityState::GoingPublic,
            ));
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
    ])
    .expect("all given SANs should be valid DNS names");
    let cert = &identity.certificate_chain().as_slice()[0];
    let spki_fingerprint = cert::spki_fingerprint_b64(cert).expect("should be a valid certificate");
    let cert_hash = cert::hash_to_b64(cert.hash());
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
    info!("WebTransport Server is running");
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
        aeronet_webtransport::server::{SessionRequest, SessionResponse},
        bevy::prelude::*,
    };

    pub(super) fn handle_server_accept_connection(
        client: Entity,
        server: Entity,
        mut trigger: On<SessionRequest>,
    ) {
        info!("{{client}} connecting to {{server}} with headers:");
        for (header_key, header_value) in &trigger.headers {
            info!("  {{header_key}}: {{header_value}}");
        }

        trigger.respond(SessionResponse::Accepted);
    }

    pub(super) fn handle_server_reject_connection() {
        // TODO: client UUID or Name is on the server's blacklist
        // TODO: Server password is incorrect
        // TODO: Server is full
        todo!("Implement on_server_shutdown_notify_clients")
    }

    pub mod ports {
        pub(in crate::server) fn is_server_port_available() {
            //todo: Detect if port is already in use
            todo!("Implement is_server_port_available")
        }
        pub(in crate::server) fn bind_test() {
            // TODO: Test binding to a port
            todo!("Implement bind_test")
        }
        pub fn find_free_port() {
            // TODO: Find a free port
            todo!("Implement find_free_port")
        }
        pub fn validate_port_range() {
            // TODO: Validate port range
            todo!("Implement validate_port_range")
        }
    }
}
