use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            // core
            DefaultPlugins,
            EguiPlugin::default(),
        ))
        .init_resource::<FakeLoadingTimer>()
        // Hierarchie statt flacher States:
        .init_state::<AppScope>()
        .add_sub_state::<SingleplayerState>()
        .add_sub_state::<OpenToLANState>()
        .add_sub_state::<ConnectToServerState>()
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        // --- Simulations-Systeme (Laufen während Timer aktiv ist) ---
        // Singleplayer Lifecycle
        .add_systems(
            Update,
            simulate_singleplayer_starting.run_if(in_state(SingleplayerState::Starting)),
        )
        .add_systems(
            Update,
            simulate_singleplayer_closing.run_if(in_state(SingleplayerState::Closing)),
        )
        // LAN Lifecycle
        .add_systems(
            Update,
            simulate_going_public.run_if(in_state(OpenToLANState::GoingPublic)),
        )
        .add_systems(
            Update,
            simulate_going_private.run_if(in_state(OpenToLANState::GoingPrivate)),
        )
        // Client Lifecycle
        .add_systems(
            Update,
            simulate_connecting.run_if(in_state(ConnectToServerState::Connecting)),
        )
        .add_systems(
            Update,
            simulate_disconnecting.run_if(in_state(ConnectToServerState::Disconnecting)),
        )
        // --- Event Observer (Trigger für Transitionen) ---
        .add_observer(on_start_singleplayer)
        .add_observer(on_stop_singleplayer)
        .add_observer(on_lan_going_public)
        .add_observer(on_lan_going_private)
        .add_observer(on_start_connection)
        .add_observer(on_disconnect_from_server)
        .run()
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource, Default)]
struct FakeLoadingTimer(Timer);

impl FakeLoadingTimer {
    fn start(&mut self, seconds: f32) {
        self.0 = Timer::from_seconds(seconds, TimerMode::Once);
        self.0.reset();
    }
}

// --- STATES ---

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppScope {
    #[default]
    MainMenu,
    Singleplayer,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Singleplayer)]
enum SingleplayerState {
    #[default]
    Starting,
    Running,
    Closing,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Singleplayer)]
enum OpenToLANState {
    #[default]
    Private,
    GoingPublic,
    Public,
    GoingPrivate,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Client)]
enum ConnectToServerState {
    #[default]
    Connecting,
    Connected,
    Disconnecting,
    Failed,
}

// --- UI SYSTEM ---

fn ui_example_system(
    mut commands: Commands,
    mut egui: EguiContexts,
    app_scope: Res<State<AppScope>>,
    // Optionals, da SubStates nicht immer existieren
    singleplayer_state: Option<Res<State<SingleplayerState>>>,
    open_to_lan_state: Option<Res<State<OpenToLANState>>>,
    connect_state: Option<Res<State<ConnectToServerState>>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("Host Client Status").show(egui.ctx_mut()?, |ui| match app_scope.get() {
        AppScope::MainMenu => ui_main_menu(ui, &mut commands),

        AppScope::Singleplayer => {
            if let (Some(sp_state), Some(lan_state)) = (singleplayer_state, open_to_lan_state) {
                ui_singleplayer(ui, &mut commands, sp_state.get(), lan_state.get());
            }
        }

        AppScope::Client => {
            if let Some(conn_state) = connect_state {
                ui_client(ui, &mut commands, conn_state.get());
            }
        }
    });
    Ok(())
}

// --- UI Sub-Functions ---

fn ui_main_menu(ui: &mut egui::Ui, commands: &mut Commands) {
    ui.heading("Main Menu");
    if ui.button("Start Singleplayer").clicked() {
        commands.trigger(StartSingleplayer);
    }
    if ui.button("Connect to Server").clicked() {
        commands.trigger(StartConnection);
    }
}

fn ui_singleplayer(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    sp_state: &SingleplayerState,
    lan_state: &OpenToLANState,
) {
    ui.heading("Singleplayer Mode");
    ui.label(format!("State: {:?}", sp_state));

    match sp_state {
        SingleplayerState::Starting => {
            ui.spinner();
            ui.label("Initializing World...");
        }
        SingleplayerState::Running => {
            ui.separator();
            ui.label(format!("LAN: {:?}", lan_state));

            match lan_state {
                OpenToLANState::Private => {
                    if ui.button("Go Public (LAN)").clicked() {
                        commands.trigger(SingleplayerGoingPublic);
                    }
                }
                OpenToLANState::GoingPublic => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Opening Ports...");
                    });
                }
                OpenToLANState::Public => {
                    ui.label("Server is visible on LAN");
                    if ui.button("Go Private").clicked() {
                        commands.trigger(SingleplayerGoingPrivate);
                    }
                }
                OpenToLANState::GoingPrivate => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Closing Ports...");
                    });
                }
                _ => {}
            }

            ui.separator();
            if ui.button("Stop Singleplayer").clicked() {
                commands.trigger(StopSingleplayer);
            }
        }
        SingleplayerState::Closing => {
            ui.spinner();
            ui.label("Saving & Shutting down...");
        }
        _ => {}
    }
}

fn ui_client(ui: &mut egui::Ui, commands: &mut Commands, state: &ConnectToServerState) {
    ui.heading("Client Mode");

    match state {
        ConnectToServerState::Connecting => {
            ui.spinner();
            ui.label("Connecting to server...");
        }
        ConnectToServerState::Connected => {
            ui.label("Status: Connected");
            ui.label("Ping: 24ms"); // Dummy
            if ui.button("Disconnect").clicked() {
                commands.trigger(DisconnectFromServer);
            }
        }
        ConnectToServerState::Disconnecting => {
            ui.spinner();
            ui.label("Disconnecting...");
        }
        ConnectToServerState::Failed => {
            ui.colored_label(egui::Color32::RED, "Connection Failed");
            if ui.button("Back to Menu").clicked() {
                commands.set_state(AppScope::MainMenu);
            }
        }
    }
}

// --- EVENTS ---

#[derive(Event, Debug, Clone, Copy)]
struct StartSingleplayer;
#[derive(Event, Debug, Clone, Copy)]
struct StopSingleplayer;
#[derive(Event, Debug, Clone, Copy)]
struct SingleplayerGoingPublic;
#[derive(Event, Debug, Clone, Copy)]
struct SingleplayerGoingPrivate;
#[derive(Event, Debug, Clone, Copy)]
struct StartConnection;
#[derive(Event, Debug, Clone, Copy)]
struct DisconnectFromServer;

// --- OBSERVERS (Trigger Logic) ---

fn on_start_singleplayer(
    _: On<StartSingleplayer>,
    mut scope: ResMut<NextState<AppScope>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    scope.set(AppScope::Singleplayer); // Default -> Starting
    timer.start(1.5); // Simuliere Boot-Zeit
}

fn on_stop_singleplayer(
    _: On<StopSingleplayer>,
    mut sp_state: ResMut<NextState<SingleplayerState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    sp_state.set(SingleplayerState::Closing);
    timer.start(1.0); // Simuliere Shutdown-Zeit (Speichern etc.)
}

fn on_lan_going_public(
    _: On<SingleplayerGoingPublic>,
    mut lan_state: ResMut<NextState<OpenToLANState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    lan_state.set(OpenToLANState::GoingPublic);
    timer.start(1.0);
}

fn on_lan_going_private(
    _: On<SingleplayerGoingPrivate>,
    mut lan_state: ResMut<NextState<OpenToLANState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    lan_state.set(OpenToLANState::GoingPrivate);
    timer.start(0.5);
}

fn on_start_connection(
    _: On<StartConnection>,
    mut scope: ResMut<NextState<AppScope>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    scope.set(AppScope::Client); // Default -> Connecting
    timer.start(2.0); // Simuliere Handshake
}

fn on_disconnect_from_server(
    _: On<DisconnectFromServer>,
    mut conn_state: ResMut<NextState<ConnectToServerState>>,
    mut timer: ResMut<FakeLoadingTimer>,
) {
    conn_state.set(ConnectToServerState::Disconnecting);
    timer.start(0.5);
}

// --- SIMULATION SYSTEMS (Worker Logic) ---

fn simulate_singleplayer_starting(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next: ResMut<NextState<SingleplayerState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        info!("Singleplayer Ready!");
        next.set(SingleplayerState::Running);
    }
}

fn simulate_singleplayer_closing(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next_scope: ResMut<NextState<AppScope>>,
    mut next_lan: ResMut<NextState<OpenToLANState>>, // Zugriff auf LAN State
) {
    timer.0.tick(time.delta());

    // WICHTIG: Wir erzwingen, dass LAN auf Private geht, während wir schließen.
    // Da wir das jeden Frame machen, ist es absolut sichergestellt.
    next_lan.set(OpenToLANState::Private);

    if timer.0.is_finished() {
        info!("Singleplayer Shutdown Complete.");
        next_scope.set(AppScope::MainMenu);
    }
}

fn simulate_going_public(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next: ResMut<NextState<OpenToLANState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next.set(OpenToLANState::Public);
    }
}

fn simulate_going_private(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next: ResMut<NextState<OpenToLANState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next.set(OpenToLANState::Private);
    }
}

fn simulate_connecting(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next: ResMut<NextState<ConnectToServerState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        info!("Connected to Server!");
        next.set(ConnectToServerState::Connected);
    }
}

fn simulate_disconnecting(
    time: Res<Time>,
    mut timer: ResMut<FakeLoadingTimer>,
    mut next_scope: ResMut<NextState<AppScope>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        info!("Disconnected.");
        next_scope.set(AppScope::MainMenu);
    }
}
