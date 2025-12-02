use {
    crate::{
        singleplayer::events::*,
        states::{AppScope, ServerVisibility, SingleplayerState},
        LocalBot, LocalClient, LocalServer, LocalSession,
    },
    aeronet_channel::ChannelIo,
    aeronet_io::{connection::Disconnect, server::Close},
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::prelude::*,
};

pub fn on_singleplayer_starting(
    _: On<RequestSingleplayerStart>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppScope>>,
) {
    next_state.set(AppScope::Singleplayer);

    let server_entity = commands
        .spawn((Name::new("Local Server"), LocalSession, LocalServer))
        .id();
    let client_entity = commands
        .spawn((Name::new("Local Client"), LocalSession, LocalClient))
        .id();

    commands.queue(ChannelIo::open(server_entity, client_entity));
}

pub fn on_singleplayer_ready(
    _: On<Add, LocalClient>,
    mut next_state: ResMut<NextState<SingleplayerState>>,
) {
    // TODO: check in which state we are at the moment
    // TODO: add If statement to check if the client and Server is added
    next_state.set(SingleplayerState::Running);
    info!("Singleplayer is ready");
}

pub fn singleplayer_running(mut _commands: Commands) {
    debug!("Singleplayer is running");
}

pub fn on_singleplayer_pauseing(
    _: On<RequestSingleplayerPause>,
    mut next_state: ResMut<NextState<SingleplayerState>>,
    server_visibility: Res<NextState<ServerVisibility>>,
) {
    match *server_visibility {
        NextState::Pending(ServerVisibility::Local) => {
            next_state.set(SingleplayerState::Paused);
        }
        _ => (),
    }
}

pub fn on_singleplayer_unpauseing(
    _: On<RequestSingleplayerResume>,
    mut next_state: ResMut<NextState<SingleplayerState>>,
    server_visibility: Res<NextState<ServerVisibility>>,
) {
    match *server_visibility {
        NextState::Pending(ServerVisibility::Local) => {
            next_state.set(SingleplayerState::Running);
        }
        _ => (),
    }
}

pub fn singleplayer_paused(mut _commands: Commands) {
    debug!("Singleplayer is paused");
}

pub fn on_singleplayer_stopping(
    _: On<RequestSingleplayerStop>,
    mut next_state: ResMut<NextState<SingleplayerState>>,
) {
    next_state.set(SingleplayerState::Stopping);
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
}

pub fn on_singleplayer_stoped(
    _: On<Remove, LocalClient>,
    mut next_state: ResMut<NextState<AppScope>>,
) {
    next_state.set(AppScope::Menu);
}

pub mod events {
    use bevy::prelude::*;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSingleplayerStart;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSingleplayerStop;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSingleplayerPause;

    #[derive(Event, Debug, Clone, Copy)]
    pub struct RequestSingleplayerResume;
}
