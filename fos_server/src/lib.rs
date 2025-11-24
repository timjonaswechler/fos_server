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
        .add_sub_state::<SingleplayerState>()
        .add_sub_state::<OpenToLANState>()
        .add_sub_state::<ConnectToServerState>()
        // --- LAN Server Logik Systeme  ---
        .add_systems(OnEnter(OpenToLANState::GoingPublic), on_going_public)
        .add_systems(OnEnter(OpenToLANState::GoingPrivate), on_going_private)
        // --- Observers (UI Events) ---
        .add_observer(on_start_singleplayer)
        .add_observer(on_stop_singleplayer)
        .add_observer(on_lan_going_public)
        .add_observer(on_lan_going_private)
        .add_observer(connection::to_server_start_connection)
        .add_observer(connection::to_server_disconnect)
        .add_observer(connection::to_server_retry_connection)
        .add_observer(on_reset_to_menu)
        // --- Observers (Aeronet Network Events) ---
        .add_observer(on_client_connecting)
        .add_observer(on_client_connected)
        .add_observer(on_client_disconnected)
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

//  Resource für LAN Server Details
#[derive(Resource, Default)]
pub struct LanServerInfo {
    pub address: String,
    pub cert_hash: String,
}

// --- STATES ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppScope {
    #[default]
    MainMenu,
    Singleplayer,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Singleplayer)]
pub enum SingleplayerState {
    #[default]
    Starting,
    Running,
    Closing,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Singleplayer)]
pub enum OpenToLANState {
    #[default]
    Private,
    GoingPrivate,
    Public,
    GoingPublic,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Client)]
pub enum ConnectToServerState {
    #[default]
    Connecting,
    Connected,
    Disconnecting,
    Failed,
}

// --- EVENTS ---

#[derive(Event, Debug, Clone, Copy)]
pub struct StartSingleplayer;
#[derive(Event, Debug, Clone, Copy)]
pub struct StopSingleplayer;
#[derive(Event, Debug, Clone, Copy)]
pub struct SingleplayerGoingPublic;
#[derive(Event, Debug, Clone, Copy)]
pub struct SingleplayerGoingPrivate;
#[derive(Event, Debug, Clone, Copy)]
pub struct StartConnection;
#[derive(Event, Debug, Clone, Copy)]
pub struct DisconnectFromServer;
#[derive(Event, Debug, Clone, Copy)]
pub struct RetryConnection;
#[derive(Event, Debug, Clone, Copy)]
pub struct ResetToMainMenu;

// --- OBSERVERS (UI Trigger Logik) ---

// Component zum Markieren von lokalen Sessions für einfaches Cleanup
#[derive(Component)]
pub struct LocalSession;

// Startet den Singleplayer mit ChannelIO
pub fn on_start_singleplayer(
    _: On<StartSingleplayer>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>,
    mut sp_state: ResMut<NextState<SingleplayerState>>,
) {
    scope.set(AppScope::Singleplayer);

    // Create Entities mit Tag
    let server_entity = commands
        .spawn((Name::new("Local Server"), LocalSession))
        .id();
    let client_entity = commands
        .spawn((Name::new("Local Client"), LocalSession))
        .id();

    // Connect them via ChannelIo
    commands.queue(ChannelIo::open(server_entity, client_entity));

    // Singleplayer is instant (no handshake delay in channel io usually)
    sp_state.set(SingleplayerState::Running);
}

// Stoppt den Singleplayer und räumt auf
pub fn on_stop_singleplayer(
    _: On<StopSingleplayer>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>, // Scope direkt ändern
    sessions: Query<Entity, With<Session>>,
    local_sessions: Query<Entity, With<LocalSession>>,
) {
    // State setzen ist hier eher kosmetisch, da wir sofort ins MainMenu springen,
    // aber es ist gut für die Event-Chain.

    info!("Stopping Singleplayer: Cleaning up entities...");
    // Wir sammeln erst alle Entities, um Dopplungen zu vermeiden
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

    // Cleanup fertig (synchron) -> Zurück ins Hauptmenü
    scope.set(AppScope::MainMenu);
}

pub fn on_lan_going_public(
    _: On<SingleplayerGoingPublic>,
    mut lan_state: ResMut<NextState<OpenToLANState>>,
) {
    lan_state.set(OpenToLANState::GoingPublic);
}

pub fn on_lan_going_private(
    _: On<SingleplayerGoingPrivate>,
    mut lan_state: ResMut<NextState<OpenToLANState>>,
) {
    lan_state.set(OpenToLANState::GoingPrivate);
}

pub fn on_reset_to_menu(_: On<ResetToMainMenu>, mut scope: ResMut<NextState<AppScope>>) {
    scope.set(AppScope::MainMenu);
}

// --- AERONET OBSERVERS ---

// Wenn Aeronet eine Session anlegt (Handshake start)
fn on_client_connecting(
    _trigger: On<Add, SessionEndpoint>,
    mut state: ResMut<NextState<ConnectToServerState>>,
) {
    state.set(ConnectToServerState::Connecting);
}

// Wenn Handshake erfolgreich
fn on_client_connected(
    _trigger: On<Add, Session>,
    mut state: ResMut<NextState<ConnectToServerState>>,
) {
    info!("Aeronet: Connected!");
    state.set(ConnectToServerState::Connected);
}

// Wenn Verbindung abbricht
fn on_client_disconnected(
    trigger: On<Disconnected>,
    mut state: ResMut<NextState<ConnectToServerState>>,
    mut err: ResMut<ErrorMessage>,
    mut app_scope: ResMut<NextState<AppScope>>,
) {
    let reason = &trigger.event().reason;
    info!("Aeronet: Disconnected: {:?}", reason);

    match reason {
        DisconnectReason::ByUser(_) => {
            // Gewollter Disconnect -> Zurück ins Main Menu
            app_scope.set(AppScope::MainMenu);
        }
        _ => {
            // Fehler -> Fehler Screen
            err.0 = format!("{:?}", reason);
            state.set(ConnectToServerState::Failed);
        }
    }
}

// --- SIMULATION SYSTEMS (nur noch Host Logic) ---

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

// System zum Starten des WebTransport Servers
pub fn on_going_public(
    mut commands: Commands,
    mut lan_server_info: ResMut<LanServerInfo>,
    config: Res<ConnectionConfig>,
    mut next_lan_state: ResMut<NextState<OpenToLANState>>,
    mut error_msg: ResMut<ErrorMessage>,
    existing_server: Query<Entity, With<WebTransportServer>>,
) {
    // Falls schon ein Server läuft, diesen zuerst schließen
    for entity in &existing_server {
        commands.entity(entity).despawn();
    }

    let port: u16 = config.lan_port.parse().unwrap_or_else(|_| {
        warn!("Invalid LAN port, using default 25565");
        25565
    });
    let listen_address = format!("0.0.0.0:{}", port);

    // Selbst-signiertes Zertifikat generieren
    let identity = match Identity::self_signed(["localhost"]) {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to generate identity: {}", e);
            error_msg.0 = format!("Cert Error: {}", e);
            next_lan_state.set(OpenToLANState::Failed);
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

    // Erfolg!
    next_lan_state.set(OpenToLANState::Public);
}

// System zum Stoppen des WebTransport Servers
pub fn on_going_private(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut lan_server_info: ResMut<LanServerInfo>, // Info beim Stoppen leeren
    mut next_lan_state: ResMut<NextState<OpenToLANState>>,
) {
    for entity in &server_query {
        commands.entity(entity).despawn(); // Server schließen durch Despawn
    }
    lan_server_info.address = String::default();
    lan_server_info.cert_hash = String::default();

    next_lan_state.set(OpenToLANState::Private);
}
