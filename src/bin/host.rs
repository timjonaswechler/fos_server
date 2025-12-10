use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fos_server::{states::*, *};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            FOSServerPlugin,
        ))
        .insert_resource(UI)
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

#[derive(Resource)]
pub struct UI;

// --- UI SYSTEM ---

fn ui_singleplayer_system(
    mut commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<GameMode>>,
    singleplayer_state: Res<State<SingleplayerState>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Singleplayer").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::InGame => match *game_mode_state.get() {
                GameMode::Singleplayer => match *singleplayer_state.get() {
                    SingleplayerState::Running => {
                        ui.label("Singleplayer is running");
                        ui.separator();
                    }
                    _ => {
                        ui.label("Singleplayer is not running");
                    }
                },
                _ => {} // client
            },
            _ => {} // menu
        });
    });
    Ok(())
}

fn ui_client_system(
    mut commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<GameMode>>,
    singleplayer_menu_state: Option<Res<State<SingleplayerMenuScreen>>>,
    multiplayer_menu_state: Option<Res<State<MultiplayerMenuScreen>>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Client").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::InGame => match *game_mode_state.get() {
                GameMode::Client => {
                    ui.label("Hello");
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
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game Menu").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::InGame => match *in_game_mode_state.get() {
                InGameMode::GameMenu => {
                    ui.label("Game Menu");
                    ui.button("Back")
                        .clicked()
                        .then(|| match *game_mode_state.get() {
                            GameMode::Singleplayer => {
                                commands.trigger(SingleplayerStateEvent::RequestTransitionTo(
                                    SingleplayerState::Stopping,
                                ));
                            }
                            GameMode::Client => {
                                commands.trigger(ClientStateEvent::RequestTransitionTo(
                                    ClientState::Disconnecting,
                                ));
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

fn ui_menu_system(
    mut commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    menu_state: Res<State<MenuScreen>>,
    singleplayer_menu_state: Option<Res<State<SingleplayerMenuScreen>>>,
    multiplayer_menu_state: Option<Res<State<MultiplayerMenuScreen>>>,
    // wiki_menu_state: Res<State<WikiMenuScreen>>,
    // settings_menu_state: Res<State<SettingsMenuScreen>>,
    mut exit: MessageWriter<AppExit>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Menu").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::Menu => match *menu_state.get() {
                MenuScreen::Main => {
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Singleplayer").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(
                                MenuScreen::Singleplayer,
                            ));
                        }
                        if ui.button("Multiplayer").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(
                                MenuScreen::Multiplayer,
                            ));
                        }
                        if ui.button("Wiki").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(MenuScreen::Wiki));
                        }
                        if ui.button("Settings").clicked() {
                            commands
                                .trigger(MainMenuEvent::RequestTransitionTo(MenuScreen::Settings));
                        }
                        if ui.button("Quit").clicked() {
                            exit.write(AppExit::Success);
                        }
                    });
                }
                MenuScreen::Singleplayer => {
                    ui.vertical_centered_justified(|ui| {
                        if let Some(singleplayer_menu_state) = singleplayer_menu_state.as_ref() {
                            match singleplayer_menu_state.get() {
                                SingleplayerMenuScreen::Overview => {
                                    if ui.button("New Game").clicked() {
                                        commands.trigger(
                                            SingleplayerMenuEvent::RequestTransitionTo(
                                                SingleplayerMenuScreen::NewGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Load Game").clicked() {
                                        commands.trigger(
                                            SingleplayerMenuEvent::RequestTransitionTo(
                                                SingleplayerMenuScreen::LoadGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Back").clicked() {
                                        commands.trigger(MainMenuEvent::RequestTransitionTo(
                                            MenuScreen::Main,
                                        ));
                                    }
                                }
                                SingleplayerMenuScreen::NewGame => {
                                    if ui.button("Start").clicked() {
                                        commands.trigger(GameModeEvent::RequestTransitionTo(
                                            GameMode::Singleplayer,
                                        ));
                                    }
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            SingleplayerMenuEvent::RequestTransitionTo(
                                                SingleplayerMenuScreen::Overview,
                                            ),
                                        );
                                    }
                                }
                                SingleplayerMenuScreen::LoadGame => {
                                    if ui.button("Load").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            SingleplayerMenuEvent::RequestTransitionTo(
                                                SingleplayerMenuScreen::Overview,
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    });
                }
                MenuScreen::Multiplayer => {
                    ui.vertical_centered_justified(|ui| {
                        if let Some(multiplayer_menu_state) = multiplayer_menu_state.as_ref() {
                            match multiplayer_menu_state.get() {
                                MultiplayerMenuScreen::Overview => {
                                    if ui.button("Host new Game").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuScreen::HostNewGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Host saved Game").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuScreen::HostNewGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Join public Game").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuScreen::JoinPublicGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Join local Game").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuScreen::JoinLocalGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Back").clicked() {
                                        commands.trigger(MainMenuEvent::RequestTransitionTo(
                                            MenuScreen::Main,
                                        ));
                                    }
                                }
                                MultiplayerMenuScreen::HostNewGame => {
                                    if ui.button("New Game").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuScreen::Overview,
                                            ),
                                        );
                                    }
                                }
                                MultiplayerMenuScreen::HostSavedGame => {
                                    if ui.button("Load Game").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuScreen::Overview,
                                            ),
                                        );
                                    }
                                }
                                MultiplayerMenuScreen::JoinPublicGame => {
                                    if ui.button("Join Public Game").clicked() {
                                        commands.trigger(GameModeEvent::RequestTransitionTo(
                                            GameMode::Client,
                                        ));
                                    }
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuScreen::Overview,
                                            ),
                                        );
                                    }
                                }
                                MultiplayerMenuScreen::JoinLocalGame => {
                                    if ui.button("Join Local Game").clicked() {
                                        commands.trigger(GameModeEvent::RequestTransitionTo(
                                            GameMode::Client,
                                        ));
                                    }
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuScreen::Overview,
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    });
                }
                MenuScreen::Wiki => {
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Back").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(MenuScreen::Main));
                        }
                    });
                }
                MenuScreen::Settings => {
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Back").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(MenuScreen::Main));
                        }
                    });
                }
            },
            _ => {}
        });
    });
    Ok(())
}
