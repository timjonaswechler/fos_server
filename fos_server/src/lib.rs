mod connection;

use {
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    aeronet_io::{
        connection::{DisconnectReason, Disconnected},
        Session, SessionEndpoint,
    },
    aeronet_webtransport::{
        cert,
        client::WebTransportClientPlugin,
        server::{
            ServerConfig, SessionRequest, SessionResponse, WebTransportServer,
            WebTransportServerPlugin,
        },
        wtransport::Identity,
    },
    bevy::prelude::*,
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
        // --- Server Visibility Logic (Systems) ---
        .add_systems(OnEnter(ServerVisibility::Opening), perform_open_server)
        .add_systems(OnEnter(ServerVisibility::Closing), perform_close_server)
        // --- Cleanup System ---
        .add_systems(Update, sys_cleanup_pending)
        // --- Observers (UI/User Requests) ---
        .add_observer(on_start_host)
        .add_observer(on_stop_host)
        .add_observer(on_open_server)
        .add_observer(on_close_server)
        .add_observer(connection::on_request_connect)
        .add_observer(connection::on_request_disconnect)
        .add_observer(connection::on_request_retry)
        .add_observer(on_reset_to_menu)
        // --- Observers (Aeronet Network Events) ---
        .add_observer(on_client_session_connecting)
        .add_observer(on_client_session_connected)
        .add_observer(on_client_session_disconnected)
        // --- Observers (WebTransport Server Events) ---
        .add_observer(on_webtransport_session_request);
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

#[derive(Component)]
pub struct CleanupPending;

fn sys_cleanup_pending(mut commands: Commands, query: Query<Entity, With<CleanupPending>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// --- STATES ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppScope {
    #[default]
    Menu,
    Host,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Host)]
pub enum HostState {
    #[default]
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Host)]
pub enum ServerVisibility {
    #[default]
    Local,
    Opening,
    Public,
    Closing,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Client)]
pub enum ClientState {
    #[default]
    Connecting,
    Connected,
    Disconnecting,
    Failed,
}

// --- EVENTS (Requests) ---

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestStartHost;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestStopHost;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestOpenServer;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestCloseServer;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestConnect;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestDisconnect;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestRetryConnect;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestResetToMenu;

// --- OBSERVERS (UI Trigger Logic) ---

// Component to mark local sessions for easy cleanup
#[derive(Component)]
pub struct LocalSession;

// Starts the Host mode with ChannelIO
pub fn on_start_host(
    _: On<RequestStartHost>,
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
pub fn on_stop_host(
    _: On<RequestStopHost>,
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

pub fn on_open_server(
    _: On<RequestOpenServer>,
    mut visibility: ResMut<NextState<ServerVisibility>>,
) {
    visibility.set(ServerVisibility::Opening);
}

pub fn on_close_server(
    _: On<RequestCloseServer>,
    mut visibility: ResMut<NextState<ServerVisibility>>,
) {
    visibility.set(ServerVisibility::Closing);
}

pub fn on_reset_to_menu(_: On<RequestResetToMenu>, mut scope: ResMut<NextState<AppScope>>) {
    scope.set(AppScope::Menu);
}

// --- AERONET OBSERVERS ---

// When Aeronet creates a session (Handshake start)
fn on_client_session_connecting(
    _trigger: On<Add, SessionEndpoint>,
    mut state: ResMut<NextState<ClientState>>,
) {
    state.set(ClientState::Connecting);
}

// When Handshake successful
fn on_client_session_connected(
    _trigger: On<Add, Session>,
    mut state: ResMut<NextState<ClientState>>,
) {
    info!("Aeronet: Connected!");
    state.set(ClientState::Connected);
}

// When connection drops
fn on_client_session_disconnected(
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

            // FIX: Mark for cleanup instead of immediate despawn.
            // This gives the backend one frame to process the event/disconnect state
            // before we hard-kill the entity.
            commands.entity(trigger.entity).insert(CleanupPending);
        }
    }
}

// --- SIMULATION SYSTEMS (Host Logic) ---

fn on_webtransport_session_request(mut trigger: On<SessionRequest>, clients: Query<&ChildOf>) {
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

// System to start the WebTransport Server
pub fn perform_open_server(
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

// System to stop the WebTransport Server
pub fn perform_close_server(
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
