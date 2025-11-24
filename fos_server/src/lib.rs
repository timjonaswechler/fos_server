use {
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    aeronet_io::{
        connection::{Disconnect, DisconnectReason, Disconnected},
        Session, SessionEndpoint,
    },
    aeronet_webtransport::client::{ClientConfig, WebTransportClient, WebTransportClientPlugin},
    bevy::prelude::*,
    rand::Rng,
};

pub struct FOSServerPlugin;

impl Plugin for FOSServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WebTransportClientPlugin, ChannelIoPlugin))
            .init_state::<AppScope>()
            .init_resource::<FakeLoadingTimer>()
            .init_resource::<ErrorMessage>()
            .init_resource::<ConnectionConfig>()
            .add_sub_state::<SingleplayerState>()
            .add_sub_state::<OpenToLANState>()
            .add_sub_state::<ConnectToServerState>()
            // --- Simulation Logic (LAN Only) ---
            .add_systems(
                Update,
                finish_singleplayer_closing.run_if(in_state(SingleplayerState::Closing)),
            )
            .add_systems(
                Update,
                simulate_going_public.run_if(in_state(OpenToLANState::GoingPublic)),
            )
            .add_systems(
                Update,
                simulate_going_private.run_if(in_state(OpenToLANState::GoingPrivate)),
            )
            // --- Observers (UI Events) ---
            .add_observer(on_start_singleplayer)
            .add_observer(on_stop_singleplayer)
            .add_observer(on_lan_going_public)
            .add_observer(on_lan_going_private)
            .add_observer(on_start_connection)
            .add_observer(on_disconnect_from_server)
            .add_observer(on_retry_connection)
            .add_observer(on_reset_to_menu)
            // --- Observers (Aeronet Network Events) ---
            .add_observer(on_client_connecting)
            .add_observer(on_client_connected)
            .add_observer(on_client_disconnected);
    }
}

// ... (Resources unchanged) ...

#[derive(Resource)]
pub struct ConnectionConfig {
    pub target_ip: String,
    pub lan_port: String,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            target_ip: "127.0.0.1:25565".to_string(),
            lan_port: "25565".to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct FakeLoadingTimer(Timer);

impl FakeLoadingTimer {
    fn start(&mut self, seconds: f32) {
        self.0 = Timer::from_seconds(seconds, TimerMode::Once);
        self.0.reset();
    }
}

#[derive(Resource, Default)]
pub struct ErrorMessage(pub String);

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
    GoingPublic,
    Public,
    GoingPrivate,
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

// --- OBSERVERS (Logik) ---

// Startet den Singleplayer mit ChannelIO
pub fn on_start_singleplayer(
    _: On<StartSingleplayer>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>,
    mut sp_state: ResMut<NextState<SingleplayerState>>,
) {
    scope.set(AppScope::Singleplayer);

    // Create Entities
    let server_entity = commands.spawn(Name::new("Local Server")).id();
    let client_entity = commands.spawn(Name::new("Local Client")).id();

    // Connect them via ChannelIo
    commands.queue(ChannelIo::open(server_entity, client_entity));

    // Singleplayer is instant (no handshake delay in channel io usually)
    sp_state.set(SingleplayerState::Running);
}

// Stoppt den Singleplayer und räumt auf
pub fn on_stop_singleplayer(
    _: On<StopSingleplayer>,
    mut sp_state: ResMut<NextState<SingleplayerState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    sp_state.set(SingleplayerState::Closing);
    timer.start(0.5); // Kurze Pause für "Shutting down..." UI Feedback
}

pub fn on_lan_going_public(
    _: On<SingleplayerGoingPublic>,
    mut lan_state: ResMut<NextState<OpenToLANState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    lan_state.set(OpenToLANState::GoingPublic);
    timer.start(1.0);
}

pub fn on_lan_going_private(
    _: On<SingleplayerGoingPrivate>,
    mut lan_state: ResMut<NextState<OpenToLANState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    lan_state.set(OpenToLANState::GoingPrivate);
    timer.start(0.5);
}

// Startet den ECHTEN Verbindungsaufbau
pub fn on_start_connection(
    _: On<StartConnection>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>,
    config: Res<ConnectionConfig>,
) {
    scope.set(AppScope::Client); // UI auf Client Mode

    // Aeronet Connect Call
    let client_config = ClientConfig::default();
    let target_url = format!("https://{}", config.target_ip); // WebTransport braucht URL

    info!("Connecting to {}...", target_url);
    let name = format!("Connection. {}", target_url);
    commands
        .spawn(Name::new(name))
        .queue(WebTransportClient::connect(client_config, target_url));
}

pub fn on_disconnect_from_server(
    _: On<DisconnectFromServer>,
    mut commands: Commands,
    mut conn_state: ResMut<NextState<ConnectToServerState>>,
    sessions: Query<Entity, With<Session>>,
) {
    conn_state.set(ConnectToServerState::Disconnecting);
    // Aeronet Disconnect
    for entity in &sessions {
        commands.trigger(Disconnect::new(entity, "Disconnect Button clicked"));
    }
}

pub fn on_retry_connection(
    _: On<RetryConnection>,
    mut commands: Commands,
    mut conn_state: ResMut<NextState<ConnectToServerState>>,
    config: Res<ConnectionConfig>,
    // Query für existierende (evtl. tote) Sessions oder Endpoints
    old_sessions: Query<Entity, Or<(With<Session>, With<SessionEndpoint>)>>,
) {
    // Cleanup: Alte Versuche löschen, bevor wir neu starten
    for entity in &old_sessions {
        commands.entity(entity).despawn_recursive();
    }

    conn_state.set(ConnectToServerState::Connecting);
    
    // Retry = Connect again
    let client_config = ClientConfig::default();
    let target_url = format!("https://{}", config.target_ip);
    let name = format!("Connection. {}", target_url);
    commands
        .spawn(Name::new(name))
        .queue(WebTransportClient::connect(client_config, target_url));
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
    mut app_scope: ResMut<NextState<AppScope>>, // Um bei User Disconnect ins Menu zu gehen
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

pub fn finish_singleplayer_closing(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next_scope: ResMut<NextState<AppScope>>,
    mut next_lan: ResMut<NextState<OpenToLANState>>,
    mut commands: Commands,
    // Cleanup Queries
    sessions: Query<Entity, With<Session>>,
) {
    timer.0.tick(time.delta());

    // Force LAN Private
    next_lan.set(OpenToLANState::Private);

    if timer.0.is_finished() {
        // Cleanup Real Entities
        for entity in &sessions {
            // Wir könnten filtern nach "Local Server/Client", aber im Singleplayer ist alles weg
            commands.entity(entity).despawn();
        }

        next_scope.set(AppScope::MainMenu);
    }
}

pub fn simulate_going_public(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next: ResMut<NextState<OpenToLANState>>,
    mut err: ResMut<ErrorMessage>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        // 20% Chance auf Fehler (z.B. Port schon belegt)
        if rand::thread_rng().gen_bool(0.2) {
            err.0 = "UPnP Negotiation Failed".to_string();
            next.set(OpenToLANState::Failed);
        } else {
            next.set(OpenToLANState::Public);
        }
    }
}

pub fn simulate_going_private(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next: ResMut<NextState<OpenToLANState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next.set(OpenToLANState::Private);
    }
}
