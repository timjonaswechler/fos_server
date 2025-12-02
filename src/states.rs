use bevy::prelude::*;

#[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum AppScope {
    #[default]
    Menu,
    Singleplayer,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Singleplayer)]
pub enum SingleplayerState {
    #[default]
    Starting,
    Running,
    Paused,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Singleplayer)]
pub enum ServerVisibility {
    #[default]
    Local,
    GoingPublic,
    Public,
    GoingPrivate,
    Error,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Client)]
pub enum ClientState {
    #[default]
    Discovering,
    Connecting,
    Connected,
    Syncing,
    Running,
    Disconnecting,
    Error,
}
