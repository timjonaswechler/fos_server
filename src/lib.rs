pub mod client;
pub mod notifications;
pub mod protocol;
pub mod server;
pub mod singleplayer;
pub mod status_management;
pub use notifications::*;
pub mod local;

use {
    aeronet_replicon::{client::AeronetRepliconClientPlugin, server::AeronetRepliconServerPlugin},
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    client::ClientLogicPlugin,
    protocol::ProtocolPlugin,
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
            RepliconPlugins,
            AeronetRepliconServerPlugin,
            AeronetRepliconClientPlugin,
            ProtocolPlugin,
            StatusManagementPlugin,
            SingleplayerLogicPlugin,
            ServerLogicPlugin,
            ClientLogicPlugin,
        ))
        .init_resource::<NotificationQueue>()
        .add_observer(on_notify)
        .add_systems(Update, notification_lifecycle);
    }
}

// Component to mark local sessions for easy cleanup
