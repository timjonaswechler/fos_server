use crate::{
    events::*,
    states::{AppScope, ClientState, HostState, ServerVisibility},
};
use aeronet_webtransport::server::{SessionRequest, SessionResponse};
use bevy::prelude::*;

pub fn host_start(_: On<RequestHostStart>, mut next_state: ResMut<NextState<AppScope>>) {
    next_state.set(AppScope::Host);
}

pub fn host_stop(_: On<RequestHostStop>, mut next_state: ResMut<NextState<HostState>>) {
    next_state.set(HostState::Stopping);
}

pub fn host_go_public(
    _: On<RequestHostGoPublic>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    next_state.set(ServerVisibility::GoingPublic);
}

pub fn host_session_request(mut trigger: On<SessionRequest>, clients: Query<&ChildOf>) {
    let client = trigger.event_target();
    let Ok(&ChildOf(server)) = clients.get(client) else {
        return;
    };

    info!("{client} connecting to {server} with headers:");
    for (header_key, header_value) in &trigger.headers {
        info!("  {header_key}: {header_value}");
    }

    trigger.respond(SessionResponse::Accepted);
}

pub fn host_go_private(
    _: On<RequestHostGoPrivate>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    next_state.set(ServerVisibility::GoingPrivate);
}

pub fn client_connect(_: On<RequestClientConnect>, mut next_state: ResMut<NextState<AppScope>>) {
    next_state.set(AppScope::Client);
}

pub fn client_disconnect(
    _: On<RequestClientDisconnect>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    next_state.set(ClientState::Disconnecting);
}

pub fn client_retry(_: On<RequestClientRetry>, mut next_state: ResMut<NextState<ClientState>>) {
    next_state.set(ClientState::Connecting);
}

pub fn reset_to_menu(_: On<RequestResetToMenu>, mut next_state: ResMut<NextState<AppScope>>) {
    next_state.set(AppScope::Menu);
}
