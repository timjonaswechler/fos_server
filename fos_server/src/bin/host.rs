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
    host_state: Option<Res<State<HostState>>>,
    server_visibility: Option<Res<State<ServerVisibility>>>,
    client_state: Option<Res<State<ClientState>>>,
    error_msg: Res<ErrorMessage>,
    mut config: ResMut<ConnectionConfig>, // Neu: Zugriff auf Config
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("Host Client Status").show(egui.ctx_mut()?, |ui| match app_scope.get() {
        AppScope::Menu => ui_main_menu(ui, &mut commands, &mut config),

        AppScope::Host => {
            if let (Some(h_state), Some(vis_state)) = (host_state, server_visibility) {
                ui_host(
                    ui,
                    &mut commands,
                    h_state.get(),
                    vis_state.get(),
                    &error_msg.0,
                    &mut config, // Neu
                );
            }
        }

        AppScope::Client => {
            if let Some(c_state) = client_state {
                ui_client(ui, &mut commands, c_state.get(), &error_msg.0);
            }
        }
    });
    Ok(())
}

fn ui_main_menu(ui: &mut egui::Ui, commands: &mut Commands, config: &mut ConnectionConfig) {
    ui.heading("Main Menu");
    if ui.button("Start Host").clicked() {
        commands.trigger(RequestStartHost);
    }

    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Target IP:");
        ui.text_edit_singleline(&mut config.target_ip);
    });

    if ui.button("Connect to Server").clicked() {
        commands.trigger(RequestConnect);
    }
}

fn ui_host(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    h_state: &HostState,
    vis_state: &ServerVisibility,
    error_text: &str,
    config: &mut ConnectionConfig, // Neu
) {
    ui.heading("Host Mode");

    // Global Host Error (e.g. Crash on start)
    if *h_state == HostState::Failed {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", error_text));
        if ui.button("Back to Menu").clicked() {
            commands.trigger(RequestResetToMenu);
        }
        return; // Do not show further controls
    }

    ui.label(format!("State: {:?}", h_state));

    match h_state {
        HostState::Starting => {
            ui.spinner();
            ui.label("Initializing World...");
        }
        HostState::Running => {
            ui.separator();

            // Server Visibility Status Handling
            if *vis_state == ServerVisibility::Failed {
                ui.colored_label(egui::Color32::RED, format!("Server Visibility Error: {}", error_text));
                if ui.button("Acknowledge (Reset to Local)").clicked() {
                    // We reset the visibility status, but stay in Host mode
                    commands.trigger(RequestCloseServer);
                }
            } else {
                ui.label(format!("Visibility: {:?}", vis_state));
                match vis_state {
                    ServerVisibility::Local => {
                        ui.horizontal(|ui| {
                            ui.label("Port:");
                            ui.text_edit_singleline(&mut config.lan_port);
                        });
                        if ui.button("Open to Public (LAN)").clicked() {
                            commands.trigger(RequestOpenServer);
                        }
                    }
                    ServerVisibility::Opening => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Opening Ports...");
                        });
                    }
                    ServerVisibility::Public => {
                        ui.label("Server is visible on LAN");
                        if ui.button("Close (Go Private)").clicked() {
                            commands.trigger(RequestCloseServer);
                        }
                    }
                    ServerVisibility::Closing => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Closing Ports...");
                        });
                    }
                    _ => {}
                }
            }

            ui.separator();
            if ui.button("Stop Host").clicked() {
                commands.trigger(RequestStopHost);
            }
        }
        HostState::Stopping => {
            ui.spinner();
            ui.label("Saving & Shutting down...");
        }
        _ => {}
    }
}

fn ui_client(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    state: &ClientState,
    error_text: &str,
) {
    ui.heading("Client Mode");

    match state {
        ClientState::Connecting => {
            ui.spinner();
            ui.label("Connecting to server...");
        }
        ClientState::Connected => {
            ui.label("Status: Connected");
            ui.label("Ping: 24ms");
            if ui.button("Disconnect").clicked() {
                commands.trigger(RequestDisconnect);
            }
        }
        ClientState::Disconnecting => {
            ui.spinner();
            ui.label("Disconnecting...");
        }
        ClientState::Failed => {
            ui.colored_label(
                egui::Color32::RED,
                format!("Connection Failed: {}", error_text),
            );
            ui.horizontal(|ui| {
                if ui.button("Retry").clicked() {
                    commands.trigger(RequestRetryConnect);
                }
                if ui.button("Back to Menu").clicked() {
                    commands.trigger(RequestResetToMenu);
                }
            });
        }
    }
}