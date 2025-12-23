use bevy::app::AppExit;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fos_server::{
    client::{ClientTarget, DiscoveredServers, SetClientTarget},
    status_management::*,
    *,
};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            FOSServerPlugin,
        ))
        .add_systems(Startup, setup_camera_system)
        .add_systems(
            EguiPrimaryContextPass,
            ui_menu_system.run_if(in_state(AppScope::Menu)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_singleplayer_system
                .run_if(in_state(AppScope::InGame))
                .run_if(in_state(SessionType::Singleplayer)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_client_system
                .run_if(in_state(AppScope::InGame))
                .run_if(in_state(SessionType::Client)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_game_menu
                .run_if(in_state(AppScope::InGame))
                .run_if(in_state(SessionStatus::Paused)),
        )
        .run()
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(SystemParam)]
struct MenuUiParams<'w, 's> {
    commands: Commands<'w, 's>,
    egui: EguiContexts<'w, 's>,
    app_state: Res<'w, State<AppScope>>,
    menu_state: Res<'w, State<MainMenuContext>>,
    singleplayer_menu_state: Option<Res<'w, State<SingleplayerSetup>>>,
    multiplayer_menu_state: Option<Res<'w, State<MultiplayerSetup>>>,
    discovered_servers: Option<Res<'w, DiscoveredServers>>,
    client_target: Option<ResMut<'w, ClientTarget>>,
}

struct MenuActions<'w, 's> {
    commands: Commands<'w, 's>,
}

// --- UI SYSTEM ---

fn ui_singleplayer_system(
    mut _commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<SessionType>>,
    singleplayer_state: Res<State<SingleplayerStatus>>,
    session_life_cycle: Option<Res<State<SessionLifecycle>>>,
    shutdown_step: Option<Res<State<SingleplayerShutdownStep>>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Singleplayer").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| {
            if *app_state.get() == AppScope::InGame
                && *game_mode_state.get() == SessionType::Singleplayer
            {
                ui.label(format!(
                    "States: \n AppScope: {:?}\nSessionType: {:?}\nSingleplayer: {:?}\n Lifecycle: {:?}\n ShutdownStep: {:?}",
                    app_state, game_mode_state, singleplayer_state, session_life_cycle, shutdown_step
                ));
                match *singleplayer_state.get() {
                    SingleplayerStatus::Running => {
                        ui.label("Singleplayer is running");
                        ui.separator();
                    }
                    _ => {
                        ui.label("Singleplayer is not running");
                    }
                }
            }
        });
    });
    Ok(())
}

fn ui_client_system(
    mut _commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<SessionType>>,
    client_state: Res<State<ClientStatus>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Client").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::InGame => match *game_mode_state.get() {
                SessionType::Client => {
                    ui.label(format!(
                        "States: \n AppScope: {:?}\nGameMode: {:?}\nClientState: {:?}\n",
                        app_state, game_mode_state, client_state
                    ));
                }
                _ => {} // singleplayer
            },
            _ => {} // menu
        });
    });
    Ok(())
}

fn ui_game_menu(
    mut commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<SessionType>>,
    in_game_mode_state: Res<State<SessionStatus>>,
    server_visibility: Option<Res<State<ServerVisibility>>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game Menu").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::InGame => match *in_game_mode_state.get() {
                SessionStatus::Paused => {
                    ui.label("Game Menu");
                    match *game_mode_state.get() {
                        SessionType::Singleplayer => {
                            if let Some(server_visibility) = server_visibility {
                                match *server_visibility.get() {
                                    ServerVisibility::Private => {
                                        ui.button("Open to LAN ").clicked().then(|| {
                                            commands.trigger(SetServerVisibility {
                                                transition: ServerVisibility::GoingPublic,
                                            });
                                        });
                                    }
                                    ServerVisibility::Public => {
                                        ui.button("Close to LAN").clicked().then(|| {
                                            commands.trigger(SetServerVisibility {
                                                transition: ServerVisibility::GoingPrivate,
                                            });
                                        });
                                    }
                                    _ => {}
                                };
                            }
                        }
                        SessionType::Client => {}
                        SessionType::None => {}
                    };
                    ui.button("Back")
                        .clicked()
                        .then(|| match *game_mode_state.get() {
                            SessionType::Singleplayer => {
                                commands.trigger(SetSingleplayerStatus {
                                    transition: SingleplayerStatus::Stopping,
                                });
                            }
                            SessionType::Client => {
                                commands.trigger(SetClientStatus::Transition(
                                    ClientStatus::Disconnecting,
                                ));
                            }
                            SessionType::None => {}
                        });
                }
                _ => {} // playing
            },
            _ => {} // menu
        });
    });
    Ok(())
}

fn ui_menu_system(mut params: MenuUiParams) -> Result<(), BevyError> {
    // 1. ctx holen (nur &mut-Borrow auf params.egui, kein Move)
    let ctx = params.egui.ctx_mut()?;

    // 2. alles, was du brauchst, in lokale Referenzen packen
    let app_state = &params.app_state;
    let menu_state = &params.menu_state;
    let single = params.singleplayer_menu_state.as_deref();
    let multi = params.multiplayer_menu_state.as_deref();
    let discovered = params.discovered_servers.as_deref();
    let client_target = params.client_target.as_deref_mut();

    // 3. mutables „Action“-Bundle bauen für Commands + Exit
    let mut actions = MenuActions {
        commands: params.commands,
    };

    egui::Window::new("APP Menu").show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            if app_state.get() != &AppScope::Menu {
                return;
            }

            match menu_state.get() {
                MainMenuContext::Main => render_menu_main(ui, &mut actions),
                MainMenuContext::Singleplayer => render_singleplayer_menu(ui, &mut actions, single),
                MainMenuContext::Multiplayer => {
                    render_multiplayer_menu(ui, &mut actions, multi, discovered, client_target)
                }
                MainMenuContext::Wiki => render_menu_wiki(ui, &mut actions),
                MainMenuContext::Settings => render_menu_settings(ui, &mut actions),
            }
        });
    });

    Ok(())
}

fn render_menu_main(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Singleplayer").clicked() {
        actions.commands.trigger(MainMenuInteraction::SwitchContext(
            MainMenuContext::Singleplayer,
        ));
    }
    if ui.button("Multiplayer").clicked() {
        actions.commands.trigger(MainMenuInteraction::SwitchContext(
            MainMenuContext::Multiplayer,
        ));
    }
    if ui.button("Wiki").clicked() {
        actions
            .commands
            .trigger(MainMenuInteraction::SwitchContext(MainMenuContext::Wiki));
    }
    if ui.button("Settings").clicked() {
        actions.commands.trigger(MainMenuInteraction::SwitchContext(
            MainMenuContext::Settings,
        ));
    }
    if ui.button("Quit").clicked() {
        actions.commands.trigger(MainMenuInteraction::Exit);
    }
}

fn render_singleplayer_menu(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<SingleplayerSetup>>,
) {
    ui.vertical_centered_justified(|ui| {
        let Some(single) = state else {
            return;
        };

        match single.get() {
            SingleplayerSetup::Overview => {
                render_singleplayer_overview(ui, actions);
            }
            SingleplayerSetup::NewGame => {
                render_singleplayer_new_game(ui, actions);
            }
            SingleplayerSetup::LoadGame => {
                render_singleplayer_load_game(ui, actions);
            }
        }
    });
}

fn render_singleplayer_overview(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("New Game").clicked() {
        actions
            .commands
            .trigger(SetSingleplayerMenu::Navigate(SingleplayerSetup::NewGame));
    }
    if ui.button("Load Game").clicked() {
        actions
            .commands
            .trigger(SetSingleplayerMenu::Navigate(SingleplayerSetup::LoadGame));
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(SetSingleplayerMenu::Back);
    }
}

fn render_singleplayer_new_game(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Start").clicked() {
        actions.commands.trigger(SetSingleplayerNewGame::Confirm);
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(SetSingleplayerNewGame::Back);
    }
}

fn render_singleplayer_load_game(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Load").clicked() {
        actions.commands.trigger(SetSingleplayerSavedGame::Confirm);
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(SetSingleplayerSavedGame::Back);
    }
}

fn render_multiplayer_menu(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<MultiplayerSetup>>,
    discovered_servers: Option<&DiscoveredServers>,
    client_target: Option<&mut ClientTarget>,
) {
    ui.vertical_centered_justified(|ui| {
        let Some(multi) = state else {
            return;
        };

        match multi.get() {
            MultiplayerSetup::Overview => {
                render_multiplayer_overview(ui, actions);
            }
            MultiplayerSetup::HostNewGame => {
                render_multiplayer_host_new(ui, actions);
            }
            MultiplayerSetup::HostSavedGame => {
                render_multiplayer_host_saved(ui, actions);
            }
            MultiplayerSetup::JoinGame => {
                render_multiplayer_join_game(ui, actions, discovered_servers, client_target);
            }
        }
    });
}

fn render_multiplayer_overview(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Host new Game").clicked() {
        actions
            .commands
            .trigger(SetMultiplayerMenu::Navigate(MultiplayerSetup::HostNewGame));
    }

    if ui.button("Host saved Game").clicked() {
        actions.commands.trigger(SetMultiplayerMenu::Navigate(
            MultiplayerSetup::HostSavedGame,
        ));
    }

    if ui.button("Join Game").clicked() {
        actions
            .commands
            .trigger(SetMultiplayerMenu::Navigate(MultiplayerSetup::JoinGame));
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(SetMultiplayerMenu::Back);
    }
}

fn render_multiplayer_host_new(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("New Game").clicked() {
        actions.commands.trigger(SetNewHostGame::Confirm);
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(SetNewHostGame::Back);
    }
}

fn render_multiplayer_host_saved(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Load Game").clicked() {
        actions.commands.trigger(SetSavedHostGame::Confirm);
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(SetSavedHostGame::Back);
    }
}

fn render_multiplayer_join_game(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    discovered_servers: Option<&DiscoveredServers>,
    client_target: Option<&mut ClientTarget>,
) {
    ui.heading("Local Servers");

    // Local Servers Liste (SetClientTarget anpassen!)
    match discovered_servers {
        Some(res) => {
            let servers = &res.0;
            if servers.is_empty() {
                ui.label("No local servers discovered...");
            } else {
                ui.separator();
                for server in servers {
                    if ui.selectable_label(false, format!("{}", server)).clicked() {
                        // ✅ input statt target
                        actions.commands.queue(SetClientTarget {
                            input: server.clone(),
                        });
                    }
                }
                ui.separator();
            }
        }
        None => {
            ui.label("No local servers discovered...");
        }
    }

    ui.separator();

    let mut is_client_target_valid = false;
    if let Some(target) = client_target {
        ui.horizontal(|ui| {
            let response =
                ui.add(egui::TextEdit::singleline(&mut target.input).hint_text("127.0.0.1:8080"));

            if response.changed() {
                let val = target.input.clone();
                target.update_input(val);
            }

            // Status anzeigen
            ui.label(match target.is_valid {
                true => "Valid",
                false => {
                    if target.input.trim().is_empty() {
                        ""
                    } else {
                        "Invalid"
                    }
                }
            });
        });
        ui.label(format!(
            "Client Target:\nInput:{}\nIP-Address:{:?}\nPort:{}\nIs valid:{}",
            target.input, target.ip, target.port, target.is_valid
        ));

        is_client_target_valid = target.is_valid;
    }

    ui.separator();

    let join_button = ui.add_enabled(
        is_client_target_valid,
        egui::Button::new("Join Selected Game"),
    );

    if join_button.clicked() {
        actions.commands.trigger(SetJoinGame::Confirm);
    }

    ui.separator();

    if ui.button("Back").clicked() {
        actions.commands.trigger(SetJoinGame::Back);
    }
}

fn render_menu_wiki(ui: &mut egui::Ui, actions: &mut MenuActions) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Back").clicked() {
            actions
                .commands
                .trigger(MainMenuInteraction::SwitchContext(MainMenuContext::Main));
        }
    });
}

fn render_menu_settings(ui: &mut egui::Ui, actions: &mut MenuActions) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Back").clicked() {
            actions
                .commands
                .trigger(MainMenuInteraction::SwitchContext(MainMenuContext::Main));
        }
    });
}
