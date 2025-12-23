mod app;
mod menu;
mod session;

use bevy::prelude::*;

pub use {
    app::{AppScope, ChangeAppScope},
    menu::{
        main::{MainMenuContext, MainMenuInteraction},
        multiplayer::{
            HostNewGameMenuScreen, JoinGameMenuScreen, MultiplayerSetup, SetJoinGame,
            SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame,
        },
        settings::{SettingsMenuEvent, SettingsMenuScreen},
        singleplayer::{
            NewGameMenuScreen, SavedGameMenuScreen, SetSingleplayerMenu, SetSingleplayerNewGame,
            SetSingleplayerSavedGame, SingleplayerSetup,
        },
        wiki::{WikiMenuEvent, WikiMenuScreen},
    },
    session::{
        client::{ClientStatus, SetClientStatus},
        server::{ServerVisibility, SetServerVisibility},
        singleplayer::{
            SetSingleplayerShutdownStep, SetSingleplayerStatus, SingleplayerShutdownStep,
            SingleplayerStatus,
        },
        PauseMenu, PauseMenuEvent, PhysicsSimulation, SessionLifecycle, SessionStatus, SessionType,
    },
};

pub struct StatusManagementPlugin;

impl Plugin for StatusManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((app::AppPlugin, menu::MenuPlugin, session::SessionPlugin));
    }
}
