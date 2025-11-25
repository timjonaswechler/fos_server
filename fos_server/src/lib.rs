mod events;
mod states;
pub use events::*;
pub use states::*;

use {
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    aeronet_io::{
        connection::{Disconnect, DisconnectReason, Disconnected},
        Session, SessionEndpoint,
    },
    aeronet_webtransport::{
        cert,
        client::{ClientConfig, WebTransportClient, WebTransportClientPlugin},
        server::{
            ServerConfig, SessionRequest, SessionResponse, WebTransportServer,
            WebTransportServerPlugin,
        },
        wtransport::Identity,
    },
    bevy::prelude::*,
    std::time::Duration,
};

pub struct FOSServerPlugin;

impl Plugin for FOSServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WebTransportClientPlugin,
            WebTransportServerPlugin,
            ChannelIoPlugin,
        ))
        .init_state::<AppScope>()
        .init_resource::<ErrorMessage>()
        .init_resource::<ConnectionConfig>()
        .init_resource::<LanServerInfo>()
        .add_sub_state::<HostState>()
        .add_sub_state::<ServerVisibility>()
        .add_sub_state::<ClientState>()
        // --- Observers (UI/User Requests) ---
        // Host Control
        .add_observer(handle_host_start)
        .add_observer(handle_host_stop)
        .add_observer(handle_host_go_public)
        .add_observer(handle_host_go_private)
        // Client Control
        .add_observer(handle_client_connect)
        .add_observer(handle_client_disconnect)
        .add_observer(handle_client_retry)
        // Global/Misc
        .add_observer(handle_reset_to_menu)
        // --- Observers (Network Events) ---
        // Client Events
        .add_observer(on_client_connecting)
        .add_observer(on_client_connected)
        .add_observer(on_client_disconnected)
        // Host Events
        .add_observer(on_host_session_request);
    }
}

#[derive(Resource)]
pub struct ConnectionConfig {
    pub target_ip: String,
    pub lan_port: String,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            target_ip: "127.0.0.1".to_string(),
            lan_port: "25565".to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct ErrorMessage(pub String);

// Resource for LAN Server Details
#[derive(Resource, Default)]
pub struct LanServerInfo {
    pub address: String,
    pub cert_hash: String,
}

// --- OBSERVERS (UI Trigger Logic) ---

// Component to mark local sessions for easy cleanup
#[derive(Component)]
pub struct LocalSession;

#[derive(Component)]
pub struct ClientConnection;

// Starts the Host mode with ChannelIO
pub fn handle_host_start(
    _: On<RequestHostStart>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>,
    mut host_state: ResMut<NextState<HostState>>,
) {
    scope.set(AppScope::Host);

    // Create Entities with Tag
    let server_entity = commands
        .spawn((Name::new("Local Server"), LocalSession))
        .id();
    let client_entity = commands
        .spawn((Name::new("Local Client"), LocalSession))
        .id();

    // Connect them via ChannelIo
    commands.queue(ChannelIo::open(server_entity, client_entity));

    // Host mode is active immediately (no handshake delay in channel io usually)
    host_state.set(HostState::Running);
}

// Stops the Host mode and cleans up
pub fn handle_host_stop(
    _: On<RequestHostStop>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>,
    sessions: Query<Entity, With<Session>>,
    local_sessions: Query<Entity, With<LocalSession>>,
) {
    info!("Stopping Host: Cleaning up entities...");
    // Collect entities to avoid duplicates
    let mut entities_to_despawn = std::collections::HashSet::new();
    for e in &sessions {
        entities_to_despawn.insert(e);
    }
    for e in &local_sessions {
        entities_to_despawn.insert(e);
    }

    for entity in entities_to_despawn {
        if let Ok(mut entity_cmd) = commands.get_entity(entity) {
            entity_cmd.despawn();
        }
    }

    // Return to Menu
    scope.set(AppScope::Menu);
}

// Opens the WebTransport Server (Go Public)
pub fn handle_host_go_public(
    _: On<RequestHostGoPublic>,
    mut commands: Commands,
    mut lan_server_info: ResMut<LanServerInfo>,
    config: Res<ConnectionConfig>,
    mut next_visibility: ResMut<NextState<ServerVisibility>>,
    mut error_msg: ResMut<ErrorMessage>,
    existing_server: Query<Entity, With<WebTransportServer>>,
) {
    // If a server is already running, close it first
    for entity in &existing_server {
        commands.entity(entity).despawn();
    }

    let port: u16 = config.lan_port.parse().unwrap_or_else(|_| {
        warn!("Invalid LAN port, using default 25565");
        25565
    });
    let listen_address = format!("0.0.0.0:{}", port);

    // Generate self-signed certificate
    let identity = match Identity::self_signed(["localhost"]) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to generate identity: {}", e);
            error_msg.0 = format!("Cert Error: {}", e);
            next_visibility.set(ServerVisibility::Failed);
            return;
        }
    };

    let cert = &identity.certificate_chain().as_slice()[0];
    let spki_fingerprint = cert::spki_fingerprint_b64(cert).expect("should be a valid certificate");

    info!("WebTransport Server SPKI fingerprint: {}", spki_fingerprint);

    lan_server_info.address = listen_address.clone();
    lan_server_info.cert_hash = spki_fingerprint;

    let server_config = ServerConfig::builder()
        .with_bind_default(port)
        .with_identity(identity)
        .build();

    commands
        .spawn(Name::new("WebTransport Server"))
        .queue(WebTransportServer::open(server_config));

    // Success!
    next_visibility.set(ServerVisibility::Public);
}

// Closes the WebTransport Server (Go Private)
pub fn handle_host_go_private(
    _: On<RequestHostGoPrivate>,
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut lan_server_info: ResMut<LanServerInfo>, // Clear info on stop
    mut next_visibility: ResMut<NextState<ServerVisibility>>,
) {
    for entity in &server_query {
        commands.entity(entity).despawn();
    }
    lan_server_info.address = String::default();
    lan_server_info.cert_hash = String::default();

    next_visibility.set(ServerVisibility::Local);
}

pub fn handle_reset_to_menu(_: On<RequestResetToMenu>, mut scope: ResMut<NextState<AppScope>>) {
    scope.set(AppScope::Menu);
}

// Starts the connection process
pub fn handle_client_connect(
    _: On<RequestClientConnect>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>,
    mut conn_state: ResMut<NextState<ClientState>>,
    config: Res<ConnectionConfig>,
) {
    scope.set(AppScope::Client);
    conn_state.set(ClientState::Connecting);

    connect_impl(&mut commands, &config.target_ip);
}

pub fn handle_client_disconnect(
    _: On<RequestClientDisconnect>,
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
            commands.entity(entity).despawn();
            found = true;
        }
    }

    // Fallback
    if !found {
        for entity in &all_connections {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_client_retry(
    _: On<RequestClientRetry>,
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
        .max_idle_timeout(Some(Duration::from_secs(30)))
        .unwrap()
        .keep_alive_interval(Some(Duration::from_secs(3)))
        .build();

    let name = format!("Connection to {}", target_url);

    // We spawn an Entity with the WebTransportClient Component.
    // Aeronet handles the rest.
    commands
        .spawn((Name::new(name), ClientConnection))
        .queue(WebTransportClient::connect(client_config, target_url));
}

// --- NETWORK EVENT OBSERVERS ---

// When Aeronet creates a session (Handshake start) - CLIENT
fn on_client_connecting(
    _trigger: On<Add, SessionEndpoint>,
    mut state: ResMut<NextState<ClientState>>,
) {
    state.set(ClientState::Connecting);
}

// When Handshake successful - CLIENT
fn on_client_connected(_trigger: On<Add, Session>, mut state: ResMut<NextState<ClientState>>) {
    info!("Aeronet: Connected!");
    state.set(ClientState::Connected);
}

// When connection drops - CLIENT
fn on_client_disconnected(
    trigger: On<Disconnected>,
    mut commands: Commands,
    mut state: ResMut<NextState<ClientState>>,
    mut err: ResMut<ErrorMessage>,
    mut app_scope: ResMut<NextState<AppScope>>,
) {
    let reason = &trigger.event().reason;
    info!("Aeronet: Disconnected: {:?}", reason);

    match reason {
        DisconnectReason::ByUser(_) => {
            // Intentional Disconnect -> Return to Menu
            app_scope.set(AppScope::Menu);
        }
        _ => {
            // Error -> Error Screen
            err.0 = format!("{:?}", reason);
            state.set(ClientState::Failed);
        }
    }

    // FIX: Always despawn the entity on disconnect, regardless of the reason.
    // This prevents "zombie" sessions when reconnecting.
    if let Some(mut entity) = commands.get_entity(trigger.entity) {
        entity.despawn();
    }
}

// --- HOST LOGIC ---

fn on_host_session_request(mut trigger: On<SessionRequest>, clients: Query<&ChildOf>) {
    let client = trigger.event_target();
    let Ok(&ChildOf(server)) = clients.get(client) else {
        return;
    };

    info!("{client} connecting to {server} with headers:");
    for (header_key, header_value) in &trigger.headers {
        info!("  {header_key}: {header_value}");
    }

    trigger.respond(SessionResponse::Accepted);
}
