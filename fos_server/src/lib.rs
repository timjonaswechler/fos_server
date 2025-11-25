pub mod events;
mod observer;
pub mod states;
mod systems;

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::StateInspectorPlugin;

use {
    aeronet_channel::{ChannelDisconnected, ChannelIo, ChannelIoPlugin},
    aeronet_webtransport::{
        client::{WebTransportClient, WebTransportClientPlugin},
        server::{WebTransportServer, WebTransportServerClient, WebTransportServerPlugin},
    },
    bevy::prelude::*,
    events::*,
    observer::*,
    states::*,
    systems::*,
};

pub struct FOSServerPlugin;

impl Plugin for FOSServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WebTransportClientPlugin,
            WebTransportServerPlugin,
            ChannelIoPlugin,
        ))
        .init_state::<AppScope>()
        .add_sub_state::<HostState>()
        .add_sub_state::<ServerVisibility>()
        .add_sub_state::<ClientState>()
        .init_resource::<ErrorMessage>()
        .init_resource::<HostServerConfig>()
        .init_resource::<HostServerConnectionConfig>()
        .init_resource::<ClientConnectionConfig>()
        .add_observer(host_start)
        .add_observer(host_stop)
        .add_observer(host_go_public)
        .add_observer(host_session_request)
        .add_observer(host_go_private)
        .add_observer(client_connect)
        .add_observer(client_disconnect)
        .add_observer(client_retry)
        .add_observer(reset_to_menu)
        .add_systems(
            Update,
            on_host_starting
                .run_if(in_state(ServerVisibility::Local))
                .run_if(in_state(HostState::Starting))
                .run_if(in_state(AppScope::Host)),
        )
        .add_systems(
            Update,
            on_host_running_private
                .run_if(in_state(ServerVisibility::Local))
                .run_if(in_state(HostState::Running))
                .run_if(in_state(AppScope::Host)),
        )
        .add_systems(
            Update,
            on_host_stopping
                .run_if(in_state(HostState::Stopping))
                .run_if(in_state(AppScope::Host)),
        )
        .add_systems(
            Update,
            on_host_going_public
                .run_if(in_state(ServerVisibility::GoingPublic))
                .run_if(in_state(HostState::Running))
                .run_if(in_state(AppScope::Host)),
        )
        .add_systems(
            Update,
            (on_host_running_public)
                .run_if(in_state(ServerVisibility::Public))
                .run_if(in_state(HostState::Running))
                .run_if(in_state(AppScope::Host)),
        )
        .add_systems(
            Update,
            on_host_going_private
                .run_if(in_state(ServerVisibility::GoingPrivate))
                .run_if(in_state(HostState::Running))
                .run_if(in_state(AppScope::Host)),
        )
        .add_systems(
            Update,
            on_client_connecting
                .run_if(in_state(ClientState::Connecting))
                .run_if(in_state(AppScope::Client)),
        )
        .add_systems(
            Update,
            on_client_connected
                .run_if(in_state(ClientState::Connected))
                .run_if(in_state(AppScope::Client)),
        )
        .add_systems(
            Update,
            on_client_disconnecting
                .run_if(in_state(ClientState::Disconnecting))
                .run_if(in_state(AppScope::Client)),
        );

        #[cfg(debug_assertions)]
        {
            app.register_type::<AppScope>()
                .add_plugins(StateInspectorPlugin::<AppScope>::default())
                .register_type::<HostState>()
                .add_plugins(StateInspectorPlugin::<HostState>::default())
                .register_type::<ServerVisibility>()
                .add_plugins(StateInspectorPlugin::<ServerVisibility>::default())
                .register_type::<ClientState>()
                .add_plugins(StateInspectorPlugin::<ClientState>::default());
        }
    }
}

#[derive(Resource, Default)]
pub struct ErrorMessage(pub String);

// Component to mark local sessions for easy cleanup
#[derive(Component)]
pub struct LocalSession;

// Resource for LAN Server Details
#[derive(Resource)]
pub struct HostServerConfig {
    pub address: String,
    pub port: String,
    pub cert_hash: String,
}

impl Default for HostServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: "25565".to_string(),
            cert_hash: "".to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct HostServerConnectionConfig {
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

#[derive(Component)]
pub struct ClientConnection;
