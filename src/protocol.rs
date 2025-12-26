use {
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    serde::{Deserialize, Serialize},
};

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Wir versuchen es ohne expliziten Pfad, falls es im Prelude ist,
        // oder nutzen u8 falls es eine ID ist (eher unwahrscheinlich)
        app.add_client_message::<ClientChat>(Channel::Ordered)
            .add_server_message::<ServerChat>(Channel::Ordered);
    }
}

/// Nachricht, die ein Client an den Server sendet
#[derive(Event, Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChat {
    pub text: String,
}

/// Nachricht, die der Server an alle (oder bestimmte) Clients sendet
#[derive(Event, Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChat {
    pub sender: String,
    pub text: String,
}
