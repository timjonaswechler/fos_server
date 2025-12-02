use {
    crate::{
        server::events::{RequestSingleplayerGoPrivate, RequestSingleplayerGoPublic},
        states::ServerVisibility,
    },
    aeronet_io::{
        connection::Disconnect,
        server::{Close, Closed, Server, ServerEndpoint},
    },
    aeronet_webtransport::{
        cert,
        server::{SessionRequest, WebTransportServer, WebTransportServerClient},
    },
    bevy::prelude::*,
    core::time::Duration,
};

pub fn on_server_going_public(
    _: On<RequestSingleplayerGoPublic>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    next_state.set(ServerVisibility::GoingPublic);
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
    // info!("************************");
    // info!("SPKI FINGERPRINT");
    // info!("  {spki_fingerprint}");
    // info!("CERTIFICATE HASH");
    // info!("  {cert_hash}");
    // info!("************************");
    info!("WebTransport Server starting");
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

pub fn server_is_public(
    _commands: Commands,
    server_query: Query<Entity, (With<Server>, With<ServerEndpoint>)>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    if let Ok(_) = server_query.single() {
        info!("WebTransport Server is ready");
        next_state.set(ServerVisibility::Public);
    }
}
pub fn server_running(
    _commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    if server_query.is_empty() {
        next_state.set(ServerVisibility::GoingPrivate);
        return;
    }
    info!("WebTransport Server is running");
}

pub fn on_server_going_private(
    _: On<RequestSingleplayerGoPrivate>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    next_state.set(ServerVisibility::GoingPrivate);
    info!("WebTransport Server set to go down");
}

pub fn server_going_private(
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
        info!("Disconnect all clients");
        for client in client_query.iter() {
            commands.trigger(Disconnect::new(client, "Server closing"));
        }
        return;
    }
    if let Ok(server) = server_query.single() {
        info!("Close server");
        commands.trigger(Close::new(server, "Server closing"));
        return;
    }
    if client_query.is_empty() && server_query.is_empty() {
        info!("Server is down");
        next_state.set(ServerVisibility::Local);
    }
}

pub fn on_server_is_private(_: On<Closed>) {
    info!("Closed is triggered");
}

pub fn on_server_session_request(trigger: On<SessionRequest>, clients: Query<&ChildOf>) {
    let client = trigger.event_target();
    let Ok(&ChildOf(server)) = clients.get(client) else {
        return;
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
        info!("{client} connecting to {server} with headers:");
        for (header_key, header_value) in &trigger.headers {
            info!("  {header_key}: {header_value}");
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

pub mod events {
    use bevy::prelude::*;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSingleplayerGoPublic;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSingleplayerGoPrivate;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSetServerPassword;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSetServerName;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSetServerMaxPlayers;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestBanPlayer;
}
