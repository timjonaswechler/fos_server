use {
    crate::{
        AppScope, ConnectToServerState, ConnectionConfig, DisconnectFromServer, RetryConnection,
        StartConnection,
    },
    aeronet_io::{connection::Disconnect, Session, SessionEndpoint},
    aeronet_webtransport::client::{ClientConfig, WebTransportClient},
    bevy::prelude::*,
};

// Startet den Verbindungsaufbau
pub fn to_server_start_connection(
    _: On<StartConnection>,
    mut commands: Commands,
    mut scope: ResMut<NextState<AppScope>>,
    mut conn_state: ResMut<NextState<ConnectToServerState>>,
    config: Res<ConnectionConfig>,
) {
    scope.set(AppScope::Client);
    conn_state.set(ConnectToServerState::Connecting);

    connect_impl(&mut commands, &config.target_ip);
}

pub fn to_server_disconnect(
    _: On<DisconnectFromServer>,
    mut commands: Commands,
    mut conn_state: ResMut<NextState<ConnectToServerState>>,
    // Wir disconnecten alle Sessions (sollte nur eine sein)
    sessions: Query<Entity, With<Session>>,
    endpoints: Query<Entity, With<SessionEndpoint>>, // Auch Endpoints die noch verbinden
) {
    conn_state.set(ConnectToServerState::Disconnecting);

    let mut found = false;
    // Laufende Sessions trennen
    for entity in &sessions {
        commands.trigger(Disconnect::new(entity, "Disconnect Button clicked"));
        found = true;
    }
    // Noch verbindende Endpoints direkt löschen
    for entity in &endpoints {
        if !found {
            // Nur wenn noch keine Session da ist
            commands.entity(entity).despawn();
        }
    }
}

pub fn to_server_retry_connection(
    _: On<RetryConnection>,
    mut commands: Commands,
    mut conn_state: ResMut<NextState<ConnectToServerState>>,
    config: Res<ConnectionConfig>,
    // Cleanup alter Versuche
    old_sessions: Query<Entity, Or<(With<Session>, With<SessionEndpoint>)>>,
) {
    // Alles alte wegräumen
    for entity in &old_sessions {
        commands.entity(entity).despawn();
    }

    conn_state.set(ConnectToServerState::Connecting);
    connect_impl(&mut commands, &config.target_ip);
}

// Helper funktion um Code-Duplikation zu vermeiden
fn connect_impl(commands: &mut Commands, ip_str: &str) {
    // Port Handling: Wenn kein Port da ist, default 25565
    let target_url = if ip_str.contains(':') {
        format!("https://{}", ip_str)
    } else {
        format!("https://{}:25565", ip_str)
    };

    info!("Connecting to {}...", target_url);

    // WICHTIG: Für LAN Development müssen wir Zertifikats-Validierung abschalten,
    // da der Server selbst-signierte Certs nutzt.
    let client_config = ClientConfig::builder()
        .with_bind_default()
        .with_no_cert_validation()
        .build();

    let name = format!("Connection to {}", target_url);

    // Wir spawnen eine Entity mit dem WebTransportClient Component.
    // Aeronet übernimmt den Rest.
    commands
        .spawn(Name::new(name))
        .queue(WebTransportClient::connect(client_config, target_url));
}
