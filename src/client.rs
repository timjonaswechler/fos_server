use {
    crate::{
        local::LocalClient,
        notifications::NotifyError,
        server::helpers::{DISCOVERY_PORT, MAGIC},
        status_management::{
            ClientShutdownStep, ClientStatus, MultiplayerSetup, SetClientShutdownStep,
            SetClientStatus,
        },
    },
    aeronet::io::{
        connection::{Disconnect, Disconnected},
        Session,
    },
    aeronet_io::connection::DisconnectReason,
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
            .add_systems(
                OnEnter(ClientStatus::Disconnecting),
                on_client_start_disconnecting,
            )
            .add_observer(on_client_connected)
            .add_systems(
                Update,
                client_syncing.run_if(in_state(ClientStatus::Syncing)),
            )
            .add_observer(on_client_connection_failed)
            .add_systems(
                Update,
                client_disconnecting.run_if(in_state(ClientStatus::Disconnecting)),
            )
            .add_systems(
                Update,
                (client_discover_server, client_discover_server_collect)
                    .run_if(in_state(MultiplayerSetup::JoinGame)),
            );
    }
}

#[derive(Resource, Default)]
pub struct ClientTarget {
    pub input: String, // "127.0.0.1:8080"
    pub real_address: String,
    pub ip: String,
    pub port: u16,
    pub is_valid: bool,
}

impl ClientTarget {
    pub fn update_input(&mut self, input: String) {
        self.input = input;
        let trimmed = self.input.trim();

        if let Some((ip, port)) = helpers::parse_target_live(trimmed) {
            self.ip = ip;
            self.port = port;

            // Check if "https://" is already set in the input
            if trimmed.starts_with("https://") {
                // It is already there, so we use the trimmed input as the real address
                self.real_address = trimmed.to_string();
            } else {
                // It's not there (or it was "http://"), so we add "https://"
                // parse_target_live already stripped "http://" if present
                let host = trimmed.strip_prefix("http://").unwrap_or(trimmed);
                self.real_address = format!("https://{}", host);
            }
            self.is_valid = true;
        } else {
            self.ip.clear();
            self.port = 0;
            self.real_address.clear();
            self.is_valid = false;
        }
    }
}

pub struct SetClientTarget {
    pub input: String,
}

impl Command for SetClientTarget {
    fn apply(self, world: &mut World) {
        let mut target = ClientTarget::default();
        target.update_input(self.input);
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
            for server in result {
                if !discovered.0.contains(&server) {
                    discovered.0.push(server);
                }
            }
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
            client_target.real_address.clone(),
        ));
}

fn on_client_connection_failed(
    event: On<Disconnected>,
    current_state: Option<Res<State<ClientStatus>>>,
    mut commands: Commands,
    mut client_target: ResMut<ClientTarget>,
) {
    if let Some(current_state) = current_state {
        if *current_state.get() == ClientStatus::Connecting {
            match &event.reason {
                DisconnectReason::ByError(err) => {
                    error!("Connection Error: {}", err);
                    commands.trigger(NotifyError::new(format!("Connection Error: {}", err)));
                    client_target.is_valid = false;
                    commands.trigger(SetClientStatus::Failed);
                }
                DisconnectReason::ByUser(err) => {
                    error!("Connection Error: {}", err);
                    commands.trigger(NotifyError::new(format!("Connection Error: {}", err)));
                    client_target.is_valid = false;
                    commands.trigger(SetClientStatus::Failed);
                }
                DisconnectReason::ByPeer(err) => {
                    error!("Connection Error: {}", err);
                    commands.trigger(NotifyError::new(format!("Connection Error: {}", err)));
                    client_target.is_valid = false;
                    commands.trigger(SetClientStatus::Failed);
                }
            }
        }
    }
}

pub fn on_client_connected(trigger: On<Add, Session>, names: Query<&Name>, mut commands: Commands) {
    let target = trigger.event_target();

    let name = names.get(target).ok();
    if let Some(name) = name {
        info!("Connected as {}", name.as_str());
    } else {
        warn!("Session {} missing Name component", target);
    }
    commands.trigger(SetClientStatus::Transition(ClientStatus::Syncing));
}

pub fn client_syncing(mut commands: Commands) {
    info!("TODO: Implement client sync system");
    commands.trigger(SetClientStatus::Transition(ClientStatus::Running));
}

pub fn on_client_running() {}

pub fn on_client_receive_disconnect() {}

pub fn on_client_start_disconnecting(mut commands: Commands) {
    info!("Starting Client Disconnect sequence");
    commands.trigger(SetClientShutdownStep::Start);
}

pub fn client_disconnecting(
    mut commands: Commands,
    step: Res<State<ClientShutdownStep>>,
    client_query: Query<Entity, With<LocalClient>>,
) {
    match step.get() {
        ClientShutdownStep::DisconnectFromServer => {
            // 1. Tick: Disconnect from Server
            if let Ok(entity) = client_query.single() {
                commands.trigger(Disconnect::new(entity, "client disconnecting"));
            }
            commands.trigger(SetClientShutdownStep::Next);
        }
        ClientShutdownStep::DespawnLocalClient => {
            // 2. Tick: Despawn Local Client
            if let Ok(entity) = client_query.single() {
                if let Ok(mut entity) = commands.get_entity(entity) {
                    entity.despawn();
                }
            } else if client_query.is_empty() {
                commands.trigger(SetClientShutdownStep::Done);
            }
        }
    }
}

pub mod helpers {
    use {
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

    pub fn parse_target_live(input: &str) -> Option<(String, u16)> {
        let input = input
            .trim()
            .strip_prefix("https://")
            .unwrap_or(input)
            .strip_prefix("http://")
            .unwrap_or(input)
            .trim();

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