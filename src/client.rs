use {
    crate::{
        server::helpers::{DISCOVERY_PORT, MAGIC},
        states::{ClientState, ClientStateEvent},
        LocalClient, NotifyError,
    },
    aeronet_io::{connection::Disconnect, Session},
    aeronet_webtransport::client::WebTransportClient,
    bevy::prelude::*,
    helpers::client_config,
    std::net::UdpSocket,
};

pub struct ClientLogicPlugin;

impl Plugin for ClientLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiscoveredServers>()
            .add_observer(on_client_connecting)
            .add_observer(on_client_connected)
            .add_systems(
                Update,
                client_syncing.run_if(in_state(ClientState::Syncing)),
            )
            .add_observer(on_client_disconnecting)
            .add_systems(
                Update,
                client_discover_server.run_if(in_state(ClientState::Discovering)),
            );
    }
}

#[derive(Resource, Default)]
pub struct DiscoveredServers(pub Vec<String>);

pub fn client_discover_server(mut discovered: ResMut<DiscoveredServers>) {
    let socket = UdpSocket::bind(("0.0.0.0", 0)).expect("bind for discovery client");
    socket.set_broadcast(true).expect("enable broadcast");
    socket
        .set_read_timeout(Some(std::time::Duration::from_millis(200)))
        .ok();

    // Discovery-Paket senden
    let _ = socket.send_to(MAGIC, ("255.255.255.255", DISCOVERY_PORT));

    // Antworten einsammeln
    let mut buf = [0u8; 256];
    discovered.0.clear();
    while let Ok((len, src)) = socket.recv_from(&mut buf) {
        let s = String::from_utf8_lossy(&buf[..len]);
        if let Some(port_str) = s.strip_prefix("FORGE_RESP_V1;") {
            if let Ok(port) = port_str.parse::<u16>() {
                let addr = format!("https://{}:{}", src.ip(), port);
                discovered.0.push(addr);
            }
        }
    }
}

pub fn on_client_connecting(
    event: On<ClientStateEvent>,
    mut commands: Commands,
    mut _target: Local<String>,
    mut cert_hash: Local<String>,
    mut session_id: Local<usize>,
) {
    info!("{:?}", event.transition);
    match event.transition {
        ClientState::Connecting => {
            const DEFAULT_TARGET: &str = "https://127.0.0.1:25571";

            let mut target = String::new();
            if target.is_empty() {
                DEFAULT_TARGET.clone_into(&mut target);
            }
            let _cert_hash_resp = &mut *cert_hash;
            let cert_hash = cert_hash.clone();
            let config = match client_config(cert_hash) {
                Ok(config) => config,
                Err(err) => {
                    commands.trigger(NotifyError::new(format!(
                        "Failed to create client config: {err:?}"
                    )));
                    return;
                }
            };

            *session_id += 1;
            let name = format!("{}. {target}", *session_id);
            commands
                .spawn(Name::new(name))
                .queue(WebTransportClient::connect(config, target));
        }
        _ => {}
    }
}

pub fn on_client_connected(trigger: On<Add, Session>, names: Query<&Name>, mut commands: Commands) {
    let target = trigger.event_target();
    let _name = names
        .get(target)
        .expect("our session entity should have a name");

    commands.trigger(ClientStateEvent {
        transition: ClientState::Syncing,
    });
}

pub fn client_syncing(mut commands: Commands) {
    info!("TODO: Implement client sync system");
    commands.trigger(ClientStateEvent {
        transition: ClientState::Running,
    });
}

pub fn on_client_running() {}

pub fn on_client_receive_disconnect() {}

pub fn on_client_disconnecting(
    event: On<ClientStateEvent>,
    mut commands: Commands,
    client_query: Query<Entity, With<LocalClient>>,
) {
    match event.transition {
        ClientState::Disconnecting => {
            if let Ok(entity) = client_query.single() {
                commands.trigger(Disconnect::new(entity, "pressed disconnect button"));
            }
        }
        _ => {}
    }
}

mod helpers {
    use {
        aeronet_webtransport::{cert, client::ClientConfig, wtransport::tls::Sha256Digest},
        bevy::prelude::*,
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
