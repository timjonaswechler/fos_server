use bevy::app::AppExit;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fos_server::{
    client::{ClientTarget, DiscoveredServers, SetClientTarget},
    states::*,
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
            ui_menu_system.run_if(in_state(GamePhase::Menu)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_singleplayer_system
                .run_if(in_state(GamePhase::InGame))
                .run_if(in_state(SessionType::Singleplayer)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_client_system
                .run_if(in_state(GamePhase::InGame))
                .run_if(in_state(SessionType::Client)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_game_menu
                .run_if(in_state(GamePhase::InGame))
                .run_if(in_state(GameplayFocus::GameMenu)),
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
    app_state: Res<'w, State<GamePhase>>,
    menu_state: Res<'w, State<MenuContext>>,
    singleplayer_menu_state: Option<Res<'w, State<SingleplayerSetup>>>,
    multiplayer_menu_state: Option<Res<'w, State<MultiplayerSetup>>>,
    discovered_servers: Option<Res<'w, DiscoveredServers>>,
    client_target: Option<ResMut<'w, ClientTarget>>,
    exit: MessageWriter<'w, AppExit>,
}

struct MenuActions<'w, 's> {
    commands: Commands<'w, 's>,
    exit: MessageWriter<'w, AppExit>,
}

// --- UI SYSTEM ---

fn ui_singleplayer_system(
    mut _commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<GamePhase>>,
    game_mode_state: Res<State<SessionType>>,
    singleplayer_state: Res<State<SingleplayerStatus>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Singleplayer").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| {
            if *app_state.get() == GamePhase::InGame
                && *game_mode_state.get() == SessionType::Singleplayer
            {
                ui.label(format!(
                    "States: \n GamePhase: {:?}\nGameMode: {:?}\nSingleplayer: {:?}\n",
                    app_state, game_mode_state, singleplayer_state
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
    app_state: Res<State<GamePhase>>,
    game_mode_state: Res<State<SessionType>>,
    client_state: Res<State<ClientStatus>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Client").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            GamePhase::InGame => match *game_mode_state.get() {
                SessionType::Client => {
                    ui.label(format!(
                        "States: \n GamePhase: {:?}\nGameMode: {:?}\nClientState: {:?}\n",
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
    app_state: Res<State<GamePhase>>,
    game_mode_state: Res<State<SessionType>>,
    in_game_mode_state: Res<State<GameplayFocus>>,
    server_visibility: Option<Res<State<ServerVisibility>>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game Menu").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            GamePhase::InGame => match *in_game_mode_state.get() {
                GameplayFocus::GameMenu => {
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
                                commands.trigger(SetClientStatus {
                                    transition: ClientStatus::Disconnecting,
                                });
                            }
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
        exit: params.exit,
    };

    egui::Window::new("APP Menu").show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            if app_state.get() != &GamePhase::Menu {
                return;
            }

            match menu_state.get() {
                MenuContext::Main => render_menu_main(ui, &mut actions),
                MenuContext::Singleplayer => render_singleplayer_menu(ui, &mut actions, single),
                MenuContext::Multiplayer => {
                    render_multiplayer_menu(ui, &mut actions, multi, discovered, client_target)
                }
                MenuContext::Wiki => render_menu_wiki(ui, &mut actions),
                MenuContext::Settings => render_menu_settings(ui, &mut actions),
            }
        });
    });

    Ok(())
}

fn render_menu_main(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Singleplayer").clicked() {
        actions.commands.trigger(NavigateMainMenu {
            transition: MenuContext::Singleplayer,
        });
    }
    if ui.button("Multiplayer").clicked() {
        actions.commands.trigger(NavigateMainMenu {
            transition: MenuContext::Multiplayer,
        });
    }
    if ui.button("Wiki").clicked() {
        actions.commands.trigger(NavigateMainMenu {
            transition: MenuContext::Wiki,
        });
    }
    if ui.button("Settings").clicked() {
        actions.commands.trigger(NavigateMainMenu {
            transition: MenuContext::Settings,
        });
    }
    if ui.button("Quit").clicked() {
        actions.exit.write(AppExit::Success);
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
        actions.commands.trigger(NavigateSingleplayerMenu {
            transition: SingleplayerSetup::NewGame,
        });
    }
    if ui.button("Load Game").clicked() {
        actions.commands.trigger(NavigateSingleplayerMenu {
            transition: SingleplayerSetup::LoadGame,
        });
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(NavigateMainMenu {
            transition: MenuContext::Main,
        });
    }
}

fn render_singleplayer_new_game(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Start").clicked() {
        actions.commands.trigger(ChangeGameMode {
            transition: SessionType::Singleplayer,
        });
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(NavigateSingleplayerMenu {
            transition: SingleplayerSetup::Overview,
        });
    }
}

fn render_singleplayer_load_game(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Load").clicked() {
        // ...
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(NavigateSingleplayerMenu {
            transition: SingleplayerSetup::Overview,
        });
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
        actions.commands.trigger(NavigateMultiplayerMenu {
            transition: MultiplayerSetup::HostNewGame,
        });
    }

    if ui.button("Host saved Game").clicked() {
        actions.commands.trigger(NavigateMultiplayerMenu {
            transition: MultiplayerSetup::HostSavedGame,
        });
    }

    if ui.button("Join Game").clicked() {
        actions.commands.trigger(NavigateMultiplayerMenu {
            transition: MultiplayerSetup::JoinGame,
        });
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(NavigateMainMenu {
            transition: MenuContext::Main,
        });
    }
}

fn render_multiplayer_host_new(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("New Game").clicked() {
        // TODO: Host-Logik
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(NavigateMultiplayerMenu {
            transition: MultiplayerSetup::Overview,
        });
    }
}

fn render_multiplayer_host_saved(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Load Game").clicked() {
        // TODO: Host saved logic
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(NavigateMultiplayerMenu {
            transition: MultiplayerSetup::Overview,
        });
    }
}

fn render_multiplayer_join_game(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    discovered_servers: Option<&DiscoveredServers>,
    client_target: Option<&mut ClientTarget>,
) {
    ui.heading("Local Servers");

    match discovered_servers {
        Some(res) => {
            let servers = &res.0;
            if servers.is_empty() {
                ui.label("No local servers discovered...");
                if ui.button("🔍 Refresh").clicked() {
                    // todo Optional: Discovery manuell triggern
                }
            } else {
                ui.separator();
                for server in servers {
                    if ui.selectable_label(false, format!("{}", server)).clicked() {
                        actions.commands.queue(SetClientTarget {
                            target: server.clone(),
                        });
                    }
                }
                ui.separator();
            }
        }
        None => {
            ui.label("No local servers discovered...");
            if ui.button("🔍 Refresh").clicked() {
                // todo: Optional: Discovery manuell triggern
            }
        }
    }

    ui.separator();
    let mut is_client_target_valid = false;
    if let Some(target) = client_target {
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut target.0).hint_text("Enter server address"));
        });
        let trimmed = target.0.trim().to_owned();
        target.0 = trimmed;
        is_client_target_valid = !target.0.is_empty();
    } else {
        ui.label("No client target available");
    }
    ui.separator();

    let join_button = ui.add_enabled(
        is_client_target_valid,
        egui::Button::new("Join Selected Game"),
    );

    if join_button.clicked() {
        actions.commands.trigger(ChangeGameMode {
            transition: SessionType::Client,
        });
    }
    ui.separator();

    if ui.button("Back").clicked() {
        actions.commands.trigger(NavigateMultiplayerMenu {
            transition: MultiplayerSetup::Overview,
        });
    }
}

fn render_menu_wiki(ui: &mut egui::Ui, actions: &mut MenuActions) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Back").clicked() {
            actions.commands.trigger(NavigateMainMenu {
                transition: MenuContext::Main,
            });
        }
    });
}

fn render_menu_settings(ui: &mut egui::Ui, actions: &mut MenuActions) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Back").clicked() {
            actions.commands.trigger(NavigateMainMenu {
                transition: MenuContext::Main,
            });
        }
    });
}
