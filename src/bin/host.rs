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
        .insert_resource(UI)
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run()
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource)]
pub struct UI;

// --- UI SYSTEM ---

fn ui_example_system(
    mut commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    menu_state: Res<State<MenuState>>,
    singleplayer_menu_state: Option<Res<State<SingleplayerMenuState>>>,
    multiplayer_menu_state: Option<Res<State<MultiplayerMenuState>>>,
    // wiki_menu_state: Res<State<WikiMenuState>>,
    // settings_menu_state: Res<State<SettingsMenuState>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| match *app_state.get() {
            AppScope::Menu => match *menu_state.get() {
                MenuState::Main => {
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Singleplayer").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(
                                MenuState::Singleplayer,
                            ));
                        }
                        if ui.button("Multiplayer").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(
                                MenuState::Multiplayer,
                            ));
                        }
                        if ui.button("Wiki").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(MenuState::Wiki));
                        }
                        if ui.button("Settings").clicked() {
                            commands
                                .trigger(MainMenuEvent::RequestTransitionTo(MenuState::Settings));
                        }
                        if ui.add_enabled(false, egui::Button::new("Quit")).clicked() {
                            unreachable!();
                        }
                    });
                }
                MenuState::Singleplayer => {
                    ui.vertical_centered_justified(|ui| {
                        if let Some(singleplayer_menu_state) = singleplayer_menu_state.as_ref() {
                            match singleplayer_menu_state.get() {
                                SingleplayerMenuState::Overview => {
                                    if ui.button("New Game").clicked() {
                                        commands.trigger(
                                            SingleplayerMenuEvent::RequestTransitionTo(
                                                SingleplayerMenuState::NewGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Load Game").clicked() {
                                        commands.trigger(
                                            SingleplayerMenuEvent::RequestTransitionTo(
                                                SingleplayerMenuState::LoadGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Back").clicked() {
                                        commands.trigger(MainMenuEvent::RequestTransitionTo(
                                            MenuState::Main,
                                        ));
                                    }
                                }
                                SingleplayerMenuState::NewGame => {
                                    if ui.button("Start").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            SingleplayerMenuEvent::RequestTransitionTo(
                                                SingleplayerMenuState::Overview,
                                            ),
                                        );
                                    }
                                }
                                SingleplayerMenuState::LoadGame => {
                                    if ui.button("Load").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            SingleplayerMenuEvent::RequestTransitionTo(
                                                SingleplayerMenuState::Overview,
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    });
                }
                MenuState::Multiplayer => {
                    ui.vertical_centered_justified(|ui| {
                        if let Some(multiplayer_menu_state) = multiplayer_menu_state.as_ref() {
                            match multiplayer_menu_state.get() {
                                MultiplayerMenuState::Overview => {
                                    if ui.button("Host new Game").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuState::HostNewGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Host saved Game").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuState::HostNewGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Join public Game").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuState::JoinPublicGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Join local Game").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuState::JoinLocalGame,
                                            ),
                                        );
                                    }
                                    if ui.button("Back").clicked() {
                                        commands.trigger(MainMenuEvent::RequestTransitionTo(
                                            MenuState::Main,
                                        ));
                                    }
                                }
                                MultiplayerMenuState::HostNewGame => {
                                    if ui.button("New Game").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuState::Overview,
                                            ),
                                        );
                                    }
                                }
                                MultiplayerMenuState::HostSavedGame => {
                                    if ui.button("Load Game").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuState::Overview,
                                            ),
                                        );
                                    }
                                }
                                MultiplayerMenuState::JoinPublicGame => {
                                    if ui.button("Join Public Game").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuState::Overview,
                                            ),
                                        );
                                    }
                                }
                                MultiplayerMenuState::JoinLocalGame => {
                                    if ui.button("Join Local Game").clicked() {}
                                    if ui.button("Back").clicked() {
                                        commands.trigger(
                                            MultiplayerMenuEvent::RequestTransitionTo(
                                                MultiplayerMenuState::Overview,
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    });
                }
                MenuState::Wiki => {
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Back").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(MenuState::Main));
                        }
                    });
                }
                MenuState::Settings => {
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Back").clicked() {
                            commands.trigger(MainMenuEvent::RequestTransitionTo(MenuState::Main));
                        }
                    });
                }
            },
            _ => {}
        });
    });
    Ok(())
}
