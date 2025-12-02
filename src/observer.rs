use {
    crate::{
        events::*,
        states::{AppScope, ClientState, ServerVisibility, SingleplayerState},
    },
    aeronet_channel::ChannelIo,
    aeronet_webtransport::server::{SessionRequest, SessionResponse},
    bevy::prelude::*,
};

pub fn singleplayer_go_private(
    _: On<RequestSingleplayerGoPrivate>,
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
