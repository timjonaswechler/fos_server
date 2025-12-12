use bevy::app::AppExit;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fos_server::client::SetClientTarget;
use fos_server::{client::DiscoveredServers, states::*, *};

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
                .run_if(in_state(GameMode::Singleplayer)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_client_system
                .run_if(in_state(AppScope::InGame))
                .run_if(in_state(GameMode::Client)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_game_menu
                .run_if(in_state(AppScope::InGame))
                .run_if(in_state(InGameMode::GameMenu)),
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
    menu_state: Res<'w, State<MenuScreen>>,
    singleplayer_menu_state: Option<Res<'w, State<SingleplayerMenuScreen>>>,
    multiplayer_menu_state: Option<Res<'w, State<MultiplayerMenuScreen>>>,
    discovered_servers: Option<Res<'w, DiscoveredServers>>,
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
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<GameMode>>,
    singleplayer_state: Res<State<SingleplayerState>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Singleplayer").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| {
            if *app_state.get() == AppScope::InGame
                && *game_mode_state.get() == GameMode::Singleplayer
            {
                ui.label(format!(
                    "States: \n AppScope: {:?}\nGameMode: {:?}\nSingleplayer: {:?}\n",
                    app_state, game_mode_state, singleplayer_state
                ));
                match *singleplayer_state.get() {
                    SingleplayerState::Running => {
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
    game_mode_state: Res<State<GameMode>>,
    client_state: Res<State<ClientState>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Client").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::InGame => match *game_mode_state.get() {
                GameMode::Client => {
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
    game_mode_state: Res<State<GameMode>>,
    in_game_mode_state: Res<State<InGameMode>>,
    server_visibility: Res<State<ServerVisibilityState>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game Menu").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::InGame => match *in_game_mode_state.get() {
                InGameMode::GameMenu => {
                    ui.label("Game Menu");
                    match *game_mode_state.get() {
                        GameMode::Singleplayer => {
                            match *server_visibility.get() {
                                ServerVisibilityState::Private => {
                                    ui.button("Open to LAN ").clicked().then(|| {
                                        commands.trigger(ServerVisibilityEvent {
                                            transition: ServerVisibilityState::GoingPublic,
                                        });
                                    });
                                }
                                ServerVisibilityState::Public => {
                                    ui.button("Close to LAN").clicked().then(|| {
                                        commands.trigger(ServerVisibilityEvent {
                                            transition: ServerVisibilityState::GoingPrivate,
                                        });
                                    });
                                }
                                _ => {}
                            };
                        }
                        GameMode::Client => {}
                    };
                    ui.button("Back")
                        .clicked()
                        .then(|| match *game_mode_state.get() {
                            GameMode::Singleplayer => {
                                commands.trigger(SingleplayerStateEvent {
                                    transition: SingleplayerState::Stopping,
                                });
                            }
                            GameMode::Client => {
                                commands.trigger(ClientStateEvent {
                                    transition: ClientState::Disconnecting,
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

    // 3. mutables „Action“-Bundle bauen für Commands + Exit
    let mut actions = MenuActions {
        commands: params.commands,
        exit: params.exit,
    };

    egui::Window::new("APP Menu").show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            if app_state.get() != &AppScope::Menu {
                return;
            }

            match menu_state.get() {
                MenuScreen::Main => render_menu_main(ui, &mut actions),
                MenuScreen::Singleplayer => render_singleplayer_menu(ui, &mut actions, single),
                MenuScreen::Multiplayer => {
                    render_multiplayer_menu(ui, &mut actions, multi, discovered)
                }
                MenuScreen::Wiki => render_menu_wiki(ui, &mut actions),
                MenuScreen::Settings => render_menu_settings(ui, &mut actions),
            }
        });
    });

    Ok(())
}

fn render_menu_main(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Singleplayer").clicked() {
        actions.commands.trigger(MainMenuEvent {
            transition: MenuScreen::Singleplayer,
        });
    }
    if ui.button("Multiplayer").clicked() {
        actions.commands.trigger(MainMenuEvent {
            transition: MenuScreen::Multiplayer,
        });
    }
    if ui.button("Wiki").clicked() {
        actions.commands.trigger(MainMenuEvent {
            transition: MenuScreen::Wiki,
        });
    }
    if ui.button("Settings").clicked() {
        actions.commands.trigger(MainMenuEvent {
            transition: MenuScreen::Settings,
        });
    }
    if ui.button("Quit").clicked() {
        actions.exit.write(AppExit::Success);
    }
}

fn render_singleplayer_menu(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<SingleplayerMenuScreen>>,
) {
    ui.vertical_centered_justified(|ui| {
        let Some(single) = state else {
            return;
        };

        match single.get() {
            SingleplayerMenuScreen::Overview => {
                render_singleplayer_overview(ui, actions);
            }
            SingleplayerMenuScreen::NewGame => {
                render_singleplayer_new_game(ui, actions);
            }
            SingleplayerMenuScreen::LoadGame => {
                render_singleplayer_load_game(ui, actions);
            }
        }
    });
}

fn render_singleplayer_overview(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("New Game").clicked() {
        actions.commands.trigger(SingleplayerMenuEvent {
            transition: SingleplayerMenuScreen::NewGame,
        });
    }
    if ui.button("Load Game").clicked() {
        actions.commands.trigger(SingleplayerMenuEvent {
            transition: SingleplayerMenuScreen::LoadGame,
        });
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(MainMenuEvent {
            transition: MenuScreen::Main,
        });
    }
}

fn render_singleplayer_new_game(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Start").clicked() {
        actions.commands.trigger(GameModeEvent {
            transition: GameMode::Singleplayer,
        });
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(SingleplayerMenuEvent {
            transition: SingleplayerMenuScreen::Overview,
        });
    }
}

fn render_singleplayer_load_game(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Load").clicked() {
        // ...
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(SingleplayerMenuEvent {
            transition: SingleplayerMenuScreen::Overview,
        });
    }
}

fn render_multiplayer_menu(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<MultiplayerMenuScreen>>,
    discovered_servers: Option<&DiscoveredServers>,
) {
    ui.vertical_centered_justified(|ui| {
        let Some(multi) = state else {
            return;
        };

        match multi.get() {
            MultiplayerMenuScreen::Overview => {
                render_multiplayer_overview(ui, actions);
            }
            MultiplayerMenuScreen::HostNewGame => {
                render_multiplayer_host_new(ui, actions);
            }
            MultiplayerMenuScreen::HostSavedGame => {
                render_multiplayer_host_saved(ui, actions);
            }
            MultiplayerMenuScreen::JoinPublicGame => {
                render_multiplayer_join_public(ui, actions);
            }
            MultiplayerMenuScreen::JoinLocalGame => {
                render_multiplayer_join_local(ui, actions, discovered_servers);
            }
        }
    });
}

fn render_multiplayer_overview(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Host new Game").clicked() {
        actions.commands.trigger(MultiplayerMenuEvent {
            transition: MultiplayerMenuScreen::HostNewGame,
        });
    }

    if ui.button("Host saved Game").clicked() {
        actions.commands.trigger(MultiplayerMenuEvent {
            transition: MultiplayerMenuScreen::HostSavedGame,
        });
    }

    if ui.button("Join public Game").clicked() {
        actions.commands.trigger(MultiplayerMenuEvent {
            transition: MultiplayerMenuScreen::JoinPublicGame,
        });
    }

    if ui.button("Join local Game").clicked() {
        actions.commands.trigger(MultiplayerMenuEvent {
            transition: MultiplayerMenuScreen::JoinLocalGame,
        });
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(MainMenuEvent {
            transition: MenuScreen::Main,
        });
    }
}

fn render_multiplayer_host_new(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("New Game").clicked() {
        // TODO: Host-Logik
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(MultiplayerMenuEvent {
            transition: MultiplayerMenuScreen::Overview,
        });
    }
}

fn render_multiplayer_host_saved(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Load Game").clicked() {
        // TODO: Host saved logic
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(MultiplayerMenuEvent {
            transition: MultiplayerMenuScreen::Overview,
        });
    }
}

fn render_multiplayer_join_public(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Join Public Game").clicked() {
        actions.commands.trigger(GameModeEvent {
            transition: GameMode::Client,
        });
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(MultiplayerMenuEvent {
            transition: MultiplayerMenuScreen::Overview,
        });
    }
}

fn render_multiplayer_join_local(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    discovered_servers: Option<&DiscoveredServers>,
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
                        actions.commands.queue(SetClientTarget(server.clone()));
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

    // todo: if nothing is selected button is disabled
    ui.add_enabled(false, egui::Button::new("Can't click this"));
    if ui.button("Join Selected Game").clicked() {
        actions.commands.trigger(GameModeEvent {
            transition: GameMode::Client,
        });
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(MultiplayerMenuEvent {
            transition: MultiplayerMenuScreen::Overview,
        });
    }
}

fn render_menu_wiki(ui: &mut egui::Ui, actions: &mut MenuActions) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Back").clicked() {
            actions.commands.trigger(MainMenuEvent {
                transition: MenuScreen::Main,
            });
        }
    });
}

fn render_menu_settings(ui: &mut egui::Ui, actions: &mut MenuActions) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Back").clicked() {
            actions.commands.trigger(MainMenuEvent {
                transition: MenuScreen::Main,
            });
        }
    });
}
