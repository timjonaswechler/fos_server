pub mod client;
pub mod notifications;
pub mod server;
pub mod singleplayer;
pub mod states;

pub use notifications::*;

use {
    aeronet_channel::ChannelIoPlugin,
    aeronet_webtransport::{client::WebTransportClientPlugin, server::WebTransportServerPlugin},
    bevy::prelude::*,
    client::*,
    server::ServerLogicPlugin,
    singleplayer::SingleplayerLogicPlugin,
    states::*,
};

pub struct FOSServerPlugin;

impl Plugin for FOSServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WebTransportClientPlugin,
            WebTransportServerPlugin,
            ChannelIoPlugin,
            StatesPlugin,
            SingleplayerLogicPlugin,
            ServerLogicPlugin,
        ))
        .init_resource::<ErrorMessage>()
        .init_resource::<SingleplayerServerConfig>()
        .init_resource::<SingleplayerServerConnectionConfig>()
        .init_resource::<ClientConnectionConfig>()
        .add_observer(on_notify_error)
        .add_systems(Update, error_lifecycle)
        .add_observer(on_client_connecting)
        .add_observer(on_client_connected)
        .add_observer(on_client_sync_complete)
        // .add_observer(on_client_running)
        // .add_observer(on_client_receive_disconnect)
        .add_systems(
            Update,
            client_syncing.run_if(in_state(ClientState::Syncing)),
        );
    }
}

// Component to mark local sessions for easy cleanup
#[derive(Component)]
pub struct LocalSession;

#[derive(Component)]
pub struct LocalClient;

#[derive(Component)]
pub struct LocalServer;

#[derive(Component)]
pub struct LocalBot;

#[derive(Component)]
pub struct ClientConnection;

// Resource for LAN Server Details
#[derive(Resource)]
pub struct SingleplayerServerConfig {
    pub address: String,
    pub port: String,
    pub cert_hash: String,
}

impl Default for SingleplayerServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: "25565".to_string(),
            cert_hash: "".to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct SingleplayerServerConnectionConfig {
    pub address: String,
    pub cert_hash: String,
}

#[derive(Resource)]
pub struct ClientConnectionConfig {
    pub address: String,
    pub port: String,
    pub cert_hash: String,
}

impl Default for ClientConnectionConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: "25565".to_string(),
            cert_hash: "".to_string(),
        }
    }
}
