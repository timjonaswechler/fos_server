pub mod client;
pub mod notifications;
pub mod server;
pub mod singleplayer;
pub mod states;
pub use notifications::*;
pub mod local;

use {
    aeronet_channel::ChannelIoPlugin,
    aeronet_webtransport::client::WebTransportClientPlugin,
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    client::ClientLogicPlugin,
    serde::{Deserialize, Serialize},
    server::ServerLogicPlugin,
    singleplayer::SingleplayerLogicPlugin,
    states::*,
};

pub struct FOSServerPlugin;

#[derive(Event, Serialize, Message, Deserialize)]
pub struct DummyEvent;

impl Plugin for FOSServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WebTransportClientPlugin,
            ChannelIoPlugin,
            StatesPlugin,
            RepliconPlugins,
            SingleplayerLogicPlugin,
            ServerLogicPlugin,
            ClientLogicPlugin,
        ))
        .add_client_message::<DummyEvent>(Channel::Ordered)
        .init_resource::<ErrorMessage>()
        .add_observer(on_notify_error)
        .add_systems(Update, error_lifecycle);
    }
}

// Component to mark local sessions for easy cleanup
