use {
    crate::{
        states::{AppScope, HostState, ServerVisibility},
        LocalSession,
    },
    aeronet_channel::ChannelIo,
    aeronet_io::{
        connection::Disconnect,
        server::{Close, Server, ServerEndpoint},
    },
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::prelude::*,
    core::time::Duration,
};

pub fn host_starting(
    mut commands: Commands,
    channel_entities: Query<Entity, With<LocalSession>>,
    mut next_state: ResMut<NextState<HostState>>,
) {
    info!("host STARTING");
    // Initialize the server state
    if channel_entities.is_empty() {
        info!("creating channel entities");
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
        info!("channel entities already exist");
        info!("");
        next_state.set(HostState::Running);
    }
}

pub fn host_running_private(
    mut _commands: Commands,
    mut _next_state: ResMut<NextState<HostState>>,
) {
    info!("host running PRIVATE");
    info!("");
}

pub fn host_stopping(
    mut commands: Commands,
    mut host_state: ResMut<NextState<AppScope>>,
    server_query: Query<Entity, With<WebTransportServer>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    channel_entities: Query<Entity, With<LocalSession>>,
) {
    info!("host STOPPING");
    if !client_query.is_empty() {
        info!("disconnecting clients");
        for client in &client_query {
            commands.trigger(Disconnect::new(client, "Host is closing"));
        }
        return;
    }
    if let Ok(server) = server_query.single() {
        info!("shutting down server");
        commands.trigger(Close::new(server, "show's over, go home"));
        return;
    }
    if !channel_entities.is_empty() {
        info!("closing channels");
        for entity in &channel_entities {
            if let Ok(mut entity_cmd) = commands.get_entity(entity) {
                entity_cmd.despawn();
            }
        }
        return;
    }
    info!("host is stopped");
    info!("");
    host_state.set(AppScope::Menu);
}

pub fn host_going_public(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    info!("host going PUBLIC");
    if server_query.is_empty() {
        info!("currently no server detected");

        let identity = aeronet_webtransport::wtransport::Identity::self_signed([
            "localhost",
            "127.0.0.1",
            "::1",
        ])
        .expect("all given SANs should be valid DNS names");

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

        info!("server entity is created");
    } else {
        info!("server is detected");
        info!("");
        next_state.set(ServerVisibility::Public);
    }
}

pub fn host_running_public(mut _commands: Commands) {
    info!("host running PUBLIC");
    info!("");
}

pub fn host_going_private(
    mut commands: Commands,
    server_query: Query<Entity, (With<Server>, With<ServerEndpoint>)>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    info!("host going PRIVATE");

    if !client_query.is_empty() {
        info!("disconnecting clients");
        for client in &client_query {
            commands.trigger(Disconnect::new(client, "Host is closing"));
        }
        return;
    }
    if let Ok(server) = server_query.single() {
        info!("shutting down server");
        commands.trigger(Close::new(server, "show's over, go home"));
        return;
    }
    info!("host is PRIVATE");
    info!("");
    next_state.set(ServerVisibility::Local);
}
