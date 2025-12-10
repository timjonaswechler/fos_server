use {
    crate::{
        states::{AppScope, AppScopeEvent, SingleplayerState, SingleplayerStateEvent},
        LocalBot, LocalClient, LocalServer, LocalSession,
    },
    aeronet_channel::ChannelIo,
    aeronet_io::{connection::Disconnect, server::Close},
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::prelude::*,
};

pub struct SingleplayerLogicPlugin;

impl Plugin for SingleplayerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_singleplayer_starting)
            .add_observer(on_singleplayer_ready)
            .add_observer(on_singleplayer_running)
            .add_systems(
                Update,
                singleplayer_stopping.run_if(in_state(SingleplayerState::Stopping)),
            );
    }
}

pub fn on_singleplayer_starting(event: On<SingleplayerStateEvent>, mut commands: Commands) {
    match event.transition {
        SingleplayerState::Starting => {
            let server_entity = commands
                .spawn((Name::new("Local Server"), LocalSession, LocalServer))
                .id();
            let client_entity = commands
                .spawn((Name::new("Local Client"), LocalSession, LocalClient))
                .id();

            commands.queue(ChannelIo::open(server_entity, client_entity));
        }
        _ => {}
    }
}

pub fn on_singleplayer_ready(
    _: On<Add, LocalClient>,
    mut commands: Commands,
    current_state: Res<State<SingleplayerState>>,
) {
    if current_state.get() == &SingleplayerState::Starting {
        info!("Starting State detected")
    } // TODO: check in which state we are at the moment
      // TODO: add If statement to check if the client and Server is added
    commands.trigger(SingleplayerStateEvent {
        transition: SingleplayerState::Running,
    });
    info!("Singleplayer is ready");
}

pub fn on_singleplayer_running(event: On<SingleplayerStateEvent>, mut _commands: Commands) {
    match event.transition {
        SingleplayerState::Running => {
            debug!("Singleplayer is running");
        }
        _ => {}
    }
}

pub fn singleplayer_stopping(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    local_client_query: Query<Entity, With<LocalClient>>,
    local_bot_query: Query<Entity, With<LocalBot>>,
    local_server_query: Query<Entity, With<LocalServer>>,
) {
    // TODO: Save world state
    // after saving, we can disconnect clients
    // first tick clients will be disconnected
    if !client_query.is_empty() {
        for client in &client_query {
            commands.trigger(Disconnect::new(client, "Singleplayer closing"));
        }
        return;
    }
    // second tick server will be closed
    if let Ok(server_entity) = server_query.single() {
        commands.trigger(Close::new(server_entity, "Singleplayer closing"));
        return;
    }
    // third tick bots will be despawned
    if !local_bot_query.is_empty() {
        for bot in &local_bot_query {
            if let Ok(mut bot_entity) = commands.get_entity(bot) {
                bot_entity.despawn();
            }
        }
        return;
    }
    // fourth tick client will be despawned
    if let Ok(client_entity) = local_client_query.single() {
        if let Ok(mut client_entity) = commands.get_entity(client_entity) {
            client_entity.despawn();
        }
        return;
    }
    // fifth tick server will be despawned
    if let Ok(server_entity) = local_server_query.single() {
        if let Ok(mut server_entity) = commands.get_entity(server_entity) {
            server_entity.despawn();
        }
    }
    // sixth tick request Main Menu
    commands.trigger(AppScopeEvent {
        transition: AppScope::Menu,
    });
}
