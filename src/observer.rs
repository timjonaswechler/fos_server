use crate::{
    events::*,
    states::{AppScope, HostState, ServerVisibility},
};
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

pub fn host_go_private(
    _: On<RequestHostGoPrivate>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    next_state.set(ServerVisibility::GoingPrivate);
}
