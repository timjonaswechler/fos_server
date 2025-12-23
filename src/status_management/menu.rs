pub(super) mod main;
pub(super) mod multiplayer;
pub(super) mod settings;
pub(super) mod singleplayer;
pub(super) mod wiki;

use {
    super::AppScope, bevy::prelude::*, main::MainMenuPlugin, multiplayer::MultiplayerMenuPlugin,
    settings::SettingsMenuPlugin, singleplayer::SingleplayerMenuPlugin, wiki::WikiMenuPlugin,
};

pub(super) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SingleplayerMenuPlugin,
            MultiplayerMenuPlugin,
            SettingsMenuPlugin,
            WikiMenuPlugin,
            MainMenuPlugin,
        ));
    }
}
