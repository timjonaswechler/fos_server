use bevy::prelude::*;
use rand::Rng;

pub struct FOSServerPlugin;

impl Plugin for FOSServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppScope>()
            .init_resource::<FakeLoadingTimer>()
            .init_resource::<ErrorMessage>()
            .add_sub_state::<SingleplayerState>()
            .add_sub_state::<OpenToLANState>()
            .add_sub_state::<ConnectToServerState>()
            // --- Simulation Logic ---
            .add_systems(
                Update,
                simulate_singleplayer_starting.run_if(in_state(SingleplayerState::Starting)),
            )
            .add_systems(
                Update,
                simulate_singleplayer_closing.run_if(in_state(SingleplayerState::Closing)),
            )
            .add_systems(
                Update,
                simulate_going_public.run_if(in_state(OpenToLANState::GoingPublic)),
            )
            .add_systems(
                Update,
                simulate_going_private.run_if(in_state(OpenToLANState::GoingPrivate)),
            )
            .add_systems(
                Update,
                simulate_connecting.run_if(in_state(ConnectToServerState::Connecting)),
            )
            .add_systems(
                Update,
                simulate_disconnecting.run_if(in_state(ConnectToServerState::Disconnecting)),
            )
            // --- Observers ---
            .add_observer(on_start_singleplayer)
            .add_observer(on_stop_singleplayer)
            .add_observer(on_lan_going_public)
            .add_observer(on_lan_going_private)
            .add_observer(on_start_connection)
            .add_observer(on_disconnect_from_server)
            .add_observer(on_retry_connection)
            .add_observer(on_reset_to_menu);
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

// --- OBSERVERS ---

pub fn on_start_singleplayer(
    _: On<StartSingleplayer>,
    mut scope: ResMut<NextState<AppScope>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    scope.set(AppScope::Singleplayer);
    timer.start(1.5);
}

pub fn on_stop_singleplayer(
    _: On<StopSingleplayer>,
    mut sp_state: ResMut<NextState<SingleplayerState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    sp_state.set(SingleplayerState::Closing);
    timer.start(1.0);
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

pub fn on_start_connection(
    _: On<StartConnection>,
    mut scope: ResMut<NextState<AppScope>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    scope.set(AppScope::Client);
    timer.start(2.0);
}

pub fn on_disconnect_from_server(
    _: On<DisconnectFromServer>,
    mut conn_state: ResMut<NextState<ConnectToServerState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    conn_state.set(ConnectToServerState::Disconnecting);
    timer.start(0.5);
}

pub fn on_retry_connection(
    _: On<RetryConnection>,
    mut conn_state: ResMut<NextState<ConnectToServerState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    // Reset state to Connecting -> triggers simulation again
    conn_state.set(ConnectToServerState::Connecting);
    timer.start(1.0);
}

pub fn on_reset_to_menu(_: On<ResetToMainMenu>, mut scope: ResMut<NextState<AppScope>>) {
    scope.set(AppScope::MainMenu);
}

// --- SIMULATION SYSTEMS (mit Zufallsfehlern) ---

pub fn simulate_singleplayer_starting(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next: ResMut<NextState<SingleplayerState>>,
    mut err: ResMut<ErrorMessage>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        // 10% Chance auf Fehler
        if rand::thread_rng().gen_bool(0.1) {
            err.0 = "Failed to bind port 8080".to_string();
            next.set(SingleplayerState::Failed);
        } else {
            next.set(SingleplayerState::Running);
        }
    }
}

pub fn simulate_singleplayer_closing(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next_scope: ResMut<NextState<AppScope>>,
    mut next_lan: ResMut<NextState<OpenToLANState>>,
) {
    timer.0.tick(time.delta());
    next_lan.set(OpenToLANState::Private);

    if timer.0.is_finished() {
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

pub fn simulate_connecting(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next: ResMut<NextState<ConnectToServerState>>,
    mut err: ResMut<ErrorMessage>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        // 30% Chance auf Connection Error
        if rand::thread_rng().gen_bool(0.3) {
            err.0 = "Host unreachable (Timeout)".to_string();
            next.set(ConnectToServerState::Failed);
        } else {
            next.set(ConnectToServerState::Connected);
        }
    }
}

pub fn simulate_disconnecting(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next_scope: ResMut<NextState<AppScope>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next_scope.set(AppScope::MainMenu);
    }
}
