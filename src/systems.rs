use {
    crate::{
        states::{AppScope, ClientState, HostState, ServerVisibility},
        ErrorMessage, HostServerConfig, HostServerConnectionConfig, LocalSession,
    },
    aeronet_channel::ChannelIo,
    aeronet_io::{
        connection::Disconnect,
        server::{Close, Server},
    },
    aeronet_webtransport::{
        cert,
        server::{ServerConfig, WebTransportServer, WebTransportServerClient},
    },
    bevy::prelude::*,
    core::time::Duration,
};

pub fn on_host_starting(
    mut commands: Commands,
    channel_entities: Query<Entity, With<LocalSession>>,
    mut next_state: ResMut<NextState<HostState>>,
) {
    // Initialize the server state
    if channel_entities.is_empty() {
        // Create Entities with Tag
        let server_entity = commands
            .spawn((Name::new("Local Server"), LocalSession))
            .id();
        let client_entity = commands
            .spawn((Name::new("Local Client"), LocalSession))
            .id();

        // Connect them via ChannelIo
        commands.queue(ChannelIo::open(server_entity, client_entity));
    } else {
        next_state.set(HostState::Running);
    }
}

pub fn on_host_running_private(
    mut _commands: Commands,
    mut _next_state: ResMut<NextState<HostState>>,
) {
    // Initialize the server state
}

pub fn on_host_stopping(
    mut commands: Commands,
    mut host_state: ResMut<NextState<AppScope>>,
    server_query: Query<Entity, With<WebTransportServer>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    channel_entities: Query<Entity, With<LocalSession>>,
) {
    let mut check = false;
    if client_query.is_empty() {
        if server_query.is_empty() {
            check = true;
        } else {
            for server in &server_query {
                commands.trigger(Close::new(server, "show's over, go home"));
            }
        }
    } else {
        for client in &client_query {
            commands.trigger(Disconnect::new(client, "Server closing"));
        }
    }
    if check && channel_entities.is_empty() {
        host_state.set(AppScope::Menu);
    } else if check && !channel_entities.is_empty() {
        for entity in &channel_entities {
            if let Ok(mut entity_cmd) = commands.get_entity(entity) {
                entity_cmd.despawn();
            }
        }
    }
}

pub fn on_host_going_public(
    mut commands: Commands,
    host_server_info: Res<HostServerConfig>,
    mut host_server_connection_config: ResMut<HostServerConnectionConfig>,
    mut error_msg: ResMut<ErrorMessage>,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    
    if server_query.is_empty() {
        let identity = aeronet_webtransport::wtransport::Identity::self_signed([
            "localhost",
            "127.0.0.1",
            "::1",
        ])
        .expect("all given SANs should be valid DNS names");
        let cert = &identity.certificate_chain().as_slice()[0];
        let spki_fingerprint =
            cert::spki_fingerprint_b64(cert).expect("should be a valid certificate");
        let cert_hash = cert::hash_to_b64(cert.hash());
        info!("************************");
        info!("SPKI FINGERPRINT");
        info!("  {spki_fingerprint}");
        info!("CERTIFICATE HASH");
        info!("  {cert_hash}");
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
    } else {
        next_state.set(ServerVisibility::Public);
    }
}

pub fn on_host_running_public(
    mut _commands: Commands,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    // Initialize the server state
    next_state.set(ServerVisibility::Public);
}

pub fn on_host_going_private(
    mut commands: Commands,
    server_query: Query<Entity, With<Server>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    info!("Server closing");
    if client_query.is_empty() && server_query.is_empty() {
        next_state.set(ServerVisibility::Local);
    } else if !server_query.is_empty() {
        for server in &server_query {
            commands.trigger(Close::new(server, "Server closing"));
        }
    }
    //  else if !client_query.is_empty() {
    //     for client in &client_query {
    //         commands.trigger(Disconnect::new(client, "Server closing"));
    //     }
    // }
}

pub fn on_client_connecting(
    mut _commands: Commands,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    // Initialize the server state
    next_state.set(ClientState::Connected);
}

pub fn on_client_connected(
    mut _commands: Commands,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    // Initialize the server state
    next_state.set(ClientState::Connected);
}

pub fn on_client_disconnecting(
    mut _commands: Commands,
    mut next_state: ResMut<NextState<AppScope>>,
) {
    // Initialize the server state
    next_state.set(AppScope::Menu);
}
