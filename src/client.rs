use {
    crate::{
        local::LocalClient,
        notifications::NotifyError,
        server::helpers::{DISCOVERY_PORT, MAGIC},
        states::{ClientStatus, MenuContext, SetClientStatus},
    },
    aeronet::io::{connection::Disconnect, Session},
    aeronet_webtransport::client::{WebTransportClient, WebTransportClientPlugin},
    bevy::{
        prelude::*,
        tasks::{futures::check_ready, AsyncComputeTaskPool, Task},
    },
    helpers::client_config,
    std::{net::UdpSocket, time::Duration},
};

pub struct ClientLogicPlugin;

impl Plugin for ClientLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WebTransportClientPlugin)
            .init_resource::<DiscoveredServers>()
            .init_resource::<ClientTarget>()
            .insert_resource(DiscoveryTimer(Timer::from_seconds(
                2.0,
                TimerMode::Repeating,
            )))
            .add_systems(OnEnter(ClientStatus::Connecting), on_client_connecting)
            // .add_observer(on_client_connected)
            .add_systems(
                Update,
                client_syncing.run_if(in_state(ClientStatus::Syncing)),
            )
            .add_observer(on_client_disconnecting)
            .add_systems(
                Update,
                (client_discover_server, client_discover_server_collect)
                    .run_if(in_state(MenuContext::Multiplayer)),
            );
    }
}

#[derive(Resource, Default)]
pub struct ClientTarget {
    pub input: String, // "127.0.0.1:8080"
    pub ip: String,
    pub port: u16,
    pub is_valid: bool,
}

pub struct SetClientTarget {
    pub input: String,
}

impl Command for SetClientTarget {
    fn apply(self, world: &mut World) {
        let mut target = ClientTarget::default();

        match helpers::validate_server_address(&self.input, &mut world.commands()) {
            Ok(addr) => {
                target.input = self.input;
                target.ip = addr.ip().to_string();
                target.port = addr.port();
                target.is_valid = true;
            }
            Err(()) => {
                target.input = self.input;
                target.is_valid = false;
            }
        }
        world.insert_resource(target);
    }
}

#[derive(Resource, Default)]
pub struct DiscoveredServers(pub Vec<String>);

#[derive(Component)]
pub struct DiscoveryTask(Task<Vec<String>>);

#[derive(Resource)]
pub struct DiscoveryTimer(pub Timer);

pub fn client_discover_server(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<DiscoveryTimer>,
    query: Query<Entity, With<DiscoveryTask>>,
) {
    if !query.is_empty() {
        return;
    }

    if !timer.0.tick(time.delta()).is_finished() {
        return;
    }

    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let socket = UdpSocket::bind(("0.0.0.0", 0)).expect("bind for discovery client");
        socket.set_broadcast(true).expect("enable broadcast");
        socket
            .set_read_timeout(Some(Duration::from_millis(200)))
            .ok();

        let _ = socket.send_to(MAGIC, ("255.255.255.255", DISCOVERY_PORT));

        let mut buf = [0u8; 256];
        let mut result = Vec::new();

        while let Ok((len, src)) = socket.recv_from(&mut buf) {
            let s = String::from_utf8_lossy(&buf[..len]);
            if let Some(port_str) = s.strip_prefix("FORGE_RESP_V1;") {
                if let Ok(port) = port_str.parse::<u16>() {
                    let addr = format!("https://{}:{}", src.ip(), port);
                    result.push(addr);
                }
            }
        }

        result
    });

    commands.spawn((Name::new("DiscoveryTask"), DiscoveryTask(task)));
}

pub fn client_discover_server_collect(
    mut commands: Commands,
    mut discovered: ResMut<DiscoveredServers>,
    mut query: Query<(Entity, &mut DiscoveryTask)>,
) {
    for (entity, mut task) in &mut query {
        if let Some(result) = check_ready(&mut task.0) {
            discovered.0 = result;
            commands.entity(entity).despawn();
        }
    }
}

pub fn on_client_connecting(
    mut commands: Commands,
    client_target: Res<ClientTarget>,
    mut cert_hash: Local<String>,
    mut session_id: Local<usize>,
) {
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
    let name = format!("{:#?}. {:?}", *session_id, client_target.input);
    info!("Connecting to server at {:?}", client_target.input);
    commands
        .spawn((Name::new(name), LocalClient))
        .queue(WebTransportClient::connect(
            config,
            client_target.input.clone(),
        ));
}

pub fn on_client_connected(trigger: On<Add, Session>, names: Query<&Name>, mut commands: Commands) {
    let target = trigger.event_target();

    let name = names.get(target).ok(); // Use .ok() instead of .expect()
    if let Some(name) = name {
        info!("Connected as {}", name.as_str());
    } else {
        warn!("Session {} missing Name component", target);
    }
    commands.trigger(SetClientStatus {
        transition: ClientStatus::Syncing,
    });
}

pub fn client_syncing(mut commands: Commands) {
    info!("TODO: Implement client sync system");
    commands.trigger(SetClientStatus {
        transition: ClientStatus::Running,
    });
}

pub fn on_client_running() {}

pub fn on_client_receive_disconnect() {}

pub fn on_client_disconnecting(
    event: On<SetClientStatus>,
    mut commands: Commands,
    client_query: Query<Entity, With<LocalClient>>,
) {
    match event.transition {
        ClientStatus::Disconnecting => {
            if let Ok(entity) = client_query.single() {
                commands.trigger(Disconnect::new(entity, "pressed disconnect button"));
            }
        }
        _ => {}
    }
}

pub mod helpers {
    use {
        crate::notifications::NotifyError,
        aeronet_webtransport::{cert, client::ClientConfig, wtransport::tls::Sha256Digest},
        bevy::prelude::*,
        core::time::Duration,
        std::net::SocketAddr,
    };

    // TODO: Remove anyhow here
    pub(super) fn client_config(cert_hash: String) -> Result<ClientConfig, anyhow::Error> {
        let config = ClientConfig::builder().with_bind_default();

        let config = if cert_hash.is_empty() {
            #[cfg(debug_assertions)]
            {
                warn!("Connecting with no certificate validation");
                config.with_no_cert_validation()
            }
            #[cfg(not(debug_assertions))]
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

    pub(super) fn validate_server_address(
        target: &str,
        commands: &mut Commands,
    ) -> Result<SocketAddr, ()> {
        let target = target.trim();
        if target.is_empty() {
            commands.trigger(NotifyError::new("Server address is empty".to_string()));
            return Err(());
        }

        let addr: SocketAddr = match target.parse() {
            Ok(addr) => addr,
            Err(_) => {
                commands.trigger(NotifyError::new(format!(
                    "Invalid server address format: '{}'. Expected 'IP:PORT'",
                    target
                )));
                return Err(());
            }
        };

        if addr.port() == 0 {
            commands.trigger(NotifyError::new("Server port cannot be 0".to_string()));
            return Err(());
        }

        Ok(addr)
    }

    pub fn parse_target_live(input: &str) -> Option<(String, u16)> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        let addr: SocketAddr = match input.parse() {
            Ok(addr) => addr,
            Err(_) => return None,
        };

        if addr.port() == 0 {
            return None;
        }

        Some((addr.ip().to_string(), addr.port()))
    }
}
