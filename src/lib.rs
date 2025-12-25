pub mod client;
pub mod notifications;
pub mod server;
pub mod singleplayer;
pub mod status_management;
pub use notifications::*;
pub mod local;

use {
    bevy::prelude::*,
    // bevy_replicon::prelude::*,
    client::ClientLogicPlugin,
    serde::{Deserialize, Serialize},
    server::ServerLogicPlugin,
    singleplayer::SingleplayerLogicPlugin,
    status_management::StatusManagementPlugin,
};

pub struct FOSServerPlugin;

#[derive(Event, Serialize, Message, Deserialize)]
pub struct DummyEvent;

impl Plugin for FOSServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StatusManagementPlugin,
            SingleplayerLogicPlugin,
            ServerLogicPlugin,
            ClientLogicPlugin,
        ))
        .init_resource::<NotificationQueue>()
        .add_observer(on_notify_error)
        .add_systems(Update, notification_lifecycle);
    }
}

// Component to mark local sessions for easy cleanup
