pub mod client;
pub mod notifications;
pub mod server;
pub mod singleplayer;
pub mod states;

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::StateInspectorPlugin;

pub use notifications::*;

use {
    aeronet_channel::ChannelIoPlugin,
    aeronet_webtransport::{client::WebTransportClientPlugin, server::WebTransportServerPlugin},
    bevy::prelude::*,
    client::*,
    server::*,
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
        ))
        .init_resource::<ErrorMessage>()
        .init_resource::<SingleplayerServerConfig>()
        .init_resource::<SingleplayerServerConnectionConfig>()
        .init_resource::<ClientConnectionConfig>()
        .add_observer(on_notify_error)
        .add_systems(Update, error_lifecycle)
        .add_observer(on_server_going_public)
        .add_observer(on_server_going_private)
        .add_observer(on_server_is_private)
        // .add_observer(on_server_session_request)
        // .add_observer(on_server_client_timeout)
        // .add_observer(on_server_client_lost)
        // .add_observer(on_server_client_graceful_disconnect)
        // .add_observer(on_server_shutdown_notify_clients)
        .add_systems(
            Update,
            server_is_public
                .run_if(in_state(SingleplayerState::Running))
                .run_if(in_state(ServerVisibilityState::GoingPublic)),
        )
        .add_systems(
            Update,
            server_running
                .run_if(in_state(ServerVisibilityState::Public))
                .run_if(in_state(SingleplayerState::Running)),
        )
        .add_systems(
            Update,
            server_going_private
                .run_if(in_state(ServerVisibilityState::GoingPrivate))
                .run_if(in_state(SingleplayerState::Running)),
        )
        .add_observer(on_client_connecting)
        .add_observer(on_client_connected)
        .add_observer(on_client_sync_complete)
        // .add_observer(on_client_running)
        // .add_observer(on_client_receive_disconnect)
        .add_observer(on_client_disconnecting)
        .add_systems(
            Update,
            client_syncing.run_if(in_state(ClientState::Syncing)),
        );

        #[cfg(debug_assertions)]
        {
            app.register_type::<AppScope>()
                .add_plugins(StateInspectorPlugin::<AppScope>::default())
                .register_type::<MenuScreen>()
                .add_plugins(StateInspectorPlugin::<MenuScreen>::default())
                .register_type::<SingleplayerState>()
                .add_plugins(StateInspectorPlugin::<SingleplayerState>::default())
                .register_type::<ServerVisibilityState>()
                .add_plugins(StateInspectorPlugin::<ServerVisibilityState>::default())
                .register_type::<ClientState>()
                .add_plugins(StateInspectorPlugin::<ClientState>::default());
        }
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
