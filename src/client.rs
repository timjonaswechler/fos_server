use {
    crate::{client::events::InternSyncCompleted, states::ClientState, ErrorMessage, LocalClient},
    aeronet_io::{
        connection::{Disconnect, DisconnectReason, Disconnected},
        Session,
    },
    aeronet_webtransport::client::WebTransportClient,
    bevy::prelude::*,
    events::{RequestClientConnect, RequestClientDisconnect},
    helpers::*,
};

pub fn client_discover_server(_commands: Commands) {
    todo!("Implement client discover server system")
}

pub fn on_client_connecting(
    _: On<RequestClientConnect>,
    mut commands: Commands,
    mut target: Local<String>,
    mut cert_hash: Local<String>,
    mut session_id: Local<usize>,
) {
    const DEFAULT_TARGET: &str = "https://127.0.0.1:25571";

    let mut target = String::new();
    if target.is_empty() {
        DEFAULT_TARGET.clone_into(&mut target);
    }
    let cert_hash_resp = &mut *cert_hash;
    let cert_hash = cert_hash.clone();
    let config = match client_config(cert_hash) {
        Ok(config) => config,
        Err(err) => {
            ErrorMessage::new("Failed to create client config: {err:?}");
            return;
        }
    };

    *session_id += 1;
    let name = format!("{}. {target}", *session_id);
    commands
        .spawn(Name::new(name))
        .queue(WebTransportClient::connect(config, target));
}

pub fn on_client_connected(
    trigger: On<Add, Session>,
    names: Query<&Name>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    let target = trigger.event_target();
    let _name = names
        .get(target)
        .expect("our session entity should have a name");

    next_state.set(ClientState::Syncing);
}

pub fn client_syncing(_commands: Commands) {
    todo!("Implement client sync system");
}

pub fn on_client_sync_complete(
    _: On<InternSyncCompleted>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    next_state.set(ClientState::Running);
}

pub fn on_client_running() {}

pub fn on_client_receive_disconnect() {}

pub fn on_client_disconnecting(
    _: On<RequestClientDisconnect>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ClientState>>,
    client_query: Query<Entity, With<LocalClient>>,
) {
    if let Ok(entity) = client_query.single() {
        commands.trigger(Disconnect::new(entity, "pressed disconnect button"));
        next_state.set(ClientState::Disconnecting);
    }
}

mod helpers {
    use {
        aeronet_webtransport::{cert, client::ClientConfig, wtransport::tls::Sha256Digest},
        core::time::Duration,
    };

    // TODO: Remove anyhow here
    pub(super) fn client_config(cert_hash: String) -> Result<ClientConfig, anyhow::Error> {
        let config = ClientConfig::builder().with_bind_default();

        let config = if cert_hash.is_empty() {
            #[cfg(feature = "dangerous-configuration")]
            {
                warn!("Connecting with no certificate validation");
                config.with_no_cert_validation()
            }
            #[cfg(not(feature = "dangerous-configuration"))]
            {
                config.with_server_certificate_hashes([])
            }
        } else {
            let hash = cert::hash_from_b64(&cert_hash)?;
            config.with_server_certificate_hashes([Sha256Digest::new(hash)])
        };

        Ok(config
            .keep_alive_interval(Some(Duration::from_secs(1)))
            .max_idle_timeout(Some(Duration::from_secs(5)))
            .expect("should be a valid idle timeout")
            .build())
    }
}

pub mod events {
    use bevy::prelude::*;
    // Request is a Prefix for User Events or system requests
    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestClientConnect;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestClientDisconnect;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestClientRetry;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestResetToMenu;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct InternSyncCompleted;
}
