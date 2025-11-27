use bevy::prelude::*;

#[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum AppScope {
    #[default]
    Menu,
    Host,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Host)]
pub enum HostState {
    #[default]
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Host)]
pub enum ServerVisibility {
    #[default]
    Local,
    GoingPublic,
    Public,
    GoingPrivate,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Client)]
pub enum ClientState {
    #[default]
    Connecting,
    Connected,
    Disconnecting,
    Failed,
}
