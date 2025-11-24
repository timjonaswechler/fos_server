use {
    crate::{
        AppScope, CleanupPending, ClientState, ConnectionConfig, RequestConnect, RequestDisconnect,
        RequestRetryConnect,
    },
    aeronet_io::{connection::Disconnect, Session, SessionEndpoint},
    aeronet_webtransport::client::{ClientConfig, WebTransportClient},
    bevy::prelude::*,
};

#[derive(Component)]
pub struct ClientConnection;

// Starts the connection process
pub fn on_request_connect(
    _: On<RequestConnect>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>,
    mut conn_state: ResMut<NextState<ClientState>>,
    config: Res<ConnectionConfig>,
) {
    scope.set(AppScope::Client);
    conn_state.set(ClientState::Connecting);

    connect_impl(&mut commands, &config.target_ip);
}

pub fn on_request_disconnect(
    _: On<RequestDisconnect>,
    mut commands: Commands,
    mut conn_state: ResMut<NextState<ClientState>>,
    // We disconnect all sessions (should only be one)
    sessions: Query<Entity, (With<Session>, With<ClientConnection>)>,
    endpoints: Query<Entity, (With<SessionEndpoint>, With<ClientConnection>)>,
    all_connections: Query<Entity, With<ClientConnection>>,
) {
    conn_state.set(ClientState::Disconnecting);

    let mut found = false;
    // Disconnect running sessions
    for entity in &sessions {
        commands.trigger(Disconnect::new(entity, "Disconnect Button clicked"));
        found = true;
    }

    // If no session is active (e.g. still connecting), we must cleanup manually.
    // But checking endpoints directly is safer.
    if !found {
        for entity in &endpoints {
             // We mark for cleanup instead of despawn to be safe
             commands.entity(entity).insert(CleanupPending);
             found = true;
        }
    }

    // Fallback
    if !found {
        for entity in &all_connections {
            commands.entity(entity).insert(CleanupPending);
        }
    }
}

pub fn on_request_retry(
    _: On<RequestRetryConnect>,
    mut commands: Commands,
    mut conn_state: ResMut<NextState<ClientState>>,
    config: Res<ConnectionConfig>,
    // Cleanup old attempts
    old_sessions: Query<Entity, With<ClientConnection>>,
) {
    // Clean up everything old immediately if it exists (should be gone by now via CleanupPending)
    for entity in &old_sessions {
        commands.entity(entity).despawn();
    }

    conn_state.set(ClientState::Connecting);
    connect_impl(&mut commands, &config.target_ip);
}

// Helper function to avoid code duplication
fn connect_impl(commands: &mut Commands, ip_str: &str) {
    // Port Handling: Default 25565 if missing
    let target_url = if ip_str.contains(':') {
        format!("https://{}", ip_str)
    } else {
        format!("https://{}:25565", ip_str)
    };

    info!("Connecting to {}...", target_url);

    // IMPORTANT: For LAN Development we must disable cert validation,
    // as the server uses self-signed certs.
    let client_config = ClientConfig::builder()
        .with_bind_default()
        .with_no_cert_validation()
        .build();

    let name = format!("Connection to {}", target_url);

    // We spawn an Entity with the WebTransportClient Component.
    // Aeronet handles the rest.
    commands
        .spawn((Name::new(name), ClientConnection))
        .queue(WebTransportClient::connect(client_config, target_url));
}