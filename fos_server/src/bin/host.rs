use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use fos_server::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin::default(), FOSServerPlugin))
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run()
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// --- UI SYSTEM ---

fn ui_example_system(
    mut commands: Commands,
    mut egui: EguiContexts,
    app_scope: Res<State<AppScope>>,
    singleplayer_state: Option<Res<State<SingleplayerState>>>,
    open_to_lan_state: Option<Res<State<OpenToLANState>>>,
    connect_state: Option<Res<State<ConnectToServerState>>>,
    error_msg: Res<ErrorMessage>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("Host Client Status").show(egui.ctx_mut()?, |ui| match app_scope.get() {
        AppScope::MainMenu => ui_main_menu(ui, &mut commands),

        AppScope::Singleplayer => {
            if let (Some(sp_state), Some(lan_state)) = (singleplayer_state, open_to_lan_state) {
                ui_singleplayer(
                    ui,
                    &mut commands,
                    sp_state.get(),
                    lan_state.get(),
                    &error_msg.0,
                );
            }
        }

        AppScope::Client => {
            if let Some(conn_state) = connect_state {
                ui_client(ui, &mut commands, conn_state.get(), &error_msg.0);
            }
        }
    });
    Ok(())
}

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
    error_text: &str,
) {
    ui.heading("Singleplayer Mode");

    // Globaler Singleplayer Fehler (z.B. Crash beim Start)
    if *sp_state == SingleplayerState::Failed {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", error_text));
        if ui.button("Back to Menu").clicked() {
            commands.trigger(ResetToMainMenu);
        }
        return; // Keine weiteren Controls anzeigen
    }

    ui.label(format!("State: {:?}", sp_state));

    match sp_state {
        SingleplayerState::Starting => {
            ui.spinner();
            ui.label("Initializing World...");
        }
        SingleplayerState::Running => {
            ui.separator();

            // LAN Status Behandlung
            if *lan_state == OpenToLANState::Failed {
                ui.colored_label(egui::Color32::RED, format!("LAN Error: {}", error_text));
                if ui.button("Acknowledge (Reset to Private)").clicked() {
                    // Wir resetten den LAN Status, aber bleiben im Singleplayer
                    commands.trigger(SingleplayerGoingPrivate);
                }
            } else {
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

fn ui_client(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    state: &ConnectToServerState,
    error_text: &str,
) {
    ui.heading("Client Mode");

    match state {
        ConnectToServerState::Connecting => {
            ui.spinner();
            ui.label("Connecting to server...");
        }
        ConnectToServerState::Connected => {
            ui.label("Status: Connected");
            ui.label("Ping: 24ms");
            if ui.button("Disconnect").clicked() {
                commands.trigger(DisconnectFromServer);
            }
        }
        ConnectToServerState::Disconnecting => {
            ui.spinner();
            ui.label("Disconnecting...");
        }
        ConnectToServerState::Failed => {
            ui.colored_label(
                egui::Color32::RED,
                format!("Connection Failed: {}", error_text),
            );
            ui.horizontal(|ui| {
                if ui.button("Retry").clicked() {
                    commands.trigger(RetryConnection);
                }
                if ui.button("Back to Menu").clicked() {
                    commands.trigger(ResetToMainMenu);
                }
            });
        }
    }
}
