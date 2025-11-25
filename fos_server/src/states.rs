use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppScope {
    #[default]
    Menu,
    Host,
    Client,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Host)]
pub enum HostState {
    #[default]
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Host)]
pub enum ServerVisibility {
    #[default]
    Local,
    Opening,
    Public,
    Closing,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppScope = AppScope::Client)]
pub enum ClientState {
    #[default]
    Connecting,
    Connected,
    Disconnecting,
    Failed,
}
