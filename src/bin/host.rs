use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fos_server::{client::events::*, server::events::*, singleplayer::events::*, states::*, *};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            FOSServerPlugin,
        ))
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
    server_visibility: Option<Res<State<ServerVisibility>>>,
    client_state: Option<Res<State<ClientState>>>,
    error_msg: Res<ErrorMessage>,
    mut singleplayer_config: ResMut<SingleplayerServerConfig>,
    mut client_config: ResMut<ClientConnectionConfig>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("Singleplayer Client Status").show(egui.ctx_mut()?, |ui| {
        match app_scope.get() {
            AppScope::Menu => ui_main_menu(ui, &mut commands, &mut client_config),

            AppScope::Singleplayer => {
                if let (Some(h_state), Some(vis_state)) = (singleplayer_state, server_visibility) {
                    ui_singleplayer(
                        ui,
                        &mut commands,
                        h_state.get(),
                        vis_state.get(),
                        &error_msg.0,
                        &mut singleplayer_config,
                    );
                }
            }

            AppScope::Client => {
                if let Some(c_state) = client_state {
                    ui_client(ui, &mut commands, c_state.get(), &error_msg.0);
                }
            }
        }
    });
    Ok(())
}

fn ui_main_menu(ui: &mut egui::Ui, commands: &mut Commands, config: &mut ClientConnectionConfig) {
    ui.heading("Main Menu");
    if ui.button("Start Singleplayer").clicked() {
        commands.trigger(RequestSingleplayerStart);
    }

    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Target IP:");
        ui.text_edit_singleline(&mut config.address);
        ui.label("Target Port:");
        ui.text_edit_singleline(&mut config.port);
    });

    if ui.button("Connect to Server").clicked() {
        commands.trigger(RequestClientConnect);
    }
}

fn ui_singleplayer(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    h_state: &SingleplayerState,
    vis_state: &ServerVisibility,
    error_text: &str,
    config: &mut SingleplayerServerConfig,
) {
    ui.heading("Singleplayer Mode");

    // Global Singleplayer Error (e.g. Crash on start)
    if *h_state == SingleplayerState::Error {
        ui.colored_label(egui::Color32::RED, format!("Error: {}", error_text));
        if ui.button("Back to Menu").clicked() {
            commands.trigger(RequestResetToMenu);
        }
        return; // Do not show further controls
    }

    ui.label(format!("State: {:?}", h_state));

    match h_state {
        SingleplayerState::Starting => {
            ui.spinner();
            ui.label("Initializing World...");
        }
        SingleplayerState::Running => {
            ui.separator();

            // Server Visibility Status Handling
            if *vis_state == ServerVisibility::Error {
                ui.colored_label(
                    egui::Color32::RED,
                    format!("Server Visibility Error: {}", error_text),
                );
                if ui.button("Acknowledge (Reset to Local)").clicked() {
                    // We reset the visibility status, but stay in Singleplayer mode
                    commands.trigger(RequestSingleplayerGoPrivate);
                }
            } else {
                ui.label(format!("Visibility: {:?}", vis_state));
                match vis_state {
                    ServerVisibility::Local => {
                        ui.horizontal(|ui| {
                            ui.label("Port:");
                            ui.text_edit_singleline(&mut config.port);
                        });
                        if ui.button("Open to Public (LAN)").clicked() {
                            commands.trigger(RequestSingleplayerGoPublic);
                        }
                    }
                    ServerVisibility::GoingPublic => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Opening Ports...");
                        });
                    }
                    ServerVisibility::Public => {
                        ui.label("Server is visible on LAN");
                        if ui.button("Close (Go Private)").clicked() {
                            commands.trigger(RequestSingleplayerGoPrivate);
                        }
                    }
                    ServerVisibility::GoingPrivate => {
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
                commands.trigger(RequestSingleplayerStop);
            }
        }
        SingleplayerState::Stopping => {
            ui.spinner();
            ui.label("Saving & Shutting down...");
        }
        _ => {}
    }
}

fn ui_client(ui: &mut egui::Ui, commands: &mut Commands, state: &ClientState, error_text: &str) {
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
                commands.trigger(RequestClientDisconnect);
            }
        }
        ClientState::Disconnecting => {
            ui.spinner();
            ui.label("Disconnecting...");
        }
        ClientState::Error => {
            ui.colored_label(
                egui::Color32::RED,
                format!("Connection Failed: {}", error_text),
            );
            ui.horizontal(|ui| {
                if ui.button("Retry").clicked() {
                    commands.trigger(RequestClientRetry);
                }
                if ui.button("Back to Menu").clicked() {
                    commands.trigger(RequestResetToMenu);
                }
            });
        }
        _ => {}
    }
}
