use {super::super::AppScope, super::super::MainMenuContext, super::SessionType, bevy::prelude::*};

pub(super) struct ClientStatusPlugin;

impl Plugin for ClientStatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ClientStatus>()
            .add_sub_state::<ClientShutdownStep>()
            .add_observer(on_client_state_event)
            .add_observer(on_set_client_shutdown_step);
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub enum SetClientStatus {
    Transition(ClientStatus),
    Failed,
}

#[derive(Event, Debug, Clone, Copy)]
pub enum SetClientShutdownStep {
    Start,
    Next,
    Done,
}

fn on_client_state_event(
    event: On<SetClientStatus>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_state: ResMut<NextState<ClientStatus>>,
) {
    match *event {
        SetClientStatus::Transition(ClientStatus::Connecting) => {
            next_state.set(ClientStatus::Connecting);
            next_session_type.set(SessionType::Client);
        }
        SetClientStatus::Transition(ClientStatus::Connected) => {
            next_state.set(ClientStatus::Connected);
            next_app_scope.set(AppScope::InGame);
        }
        SetClientStatus::Transition(ClientStatus::Disconnecting) => {
            next_state.set(ClientStatus::Disconnecting);
        }
        SetClientStatus::Transition(state) => {
            next_state.set(state);
        }
        SetClientStatus::Failed => {
            next_session_type.set(SessionType::None);
        }
    }
}

fn on_set_client_shutdown_step(
    event: On<SetClientShutdownStep>,
    shutdown_state: Res<State<ClientShutdownStep>>,
    mut next_main_menu: ResMut<NextState<MainMenuContext>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_state: ResMut<NextState<ClientShutdownStep>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
) {
    match *event {
        SetClientShutdownStep::Start => {
            next_state.set(ClientShutdownStep::DisconnectFromServer);
        }
        SetClientShutdownStep::Next => match **shutdown_state {
            ClientShutdownStep::DisconnectFromServer => {
                next_state.set(ClientShutdownStep::DespawnLocalClient);
            }
            ClientShutdownStep::DespawnLocalClient => {}
        },
        SetClientShutdownStep::Done => {
            next_app_scope.set(AppScope::Menu);
            next_main_menu.set(MainMenuContext::Main);
            next_session_type.set(SessionType::None);
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SessionType = SessionType::Client)]
pub enum ClientStatus {
    #[default]
    Connecting,
    Connected,
    Syncing,
    Running,
    Disconnecting,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(ClientStatus = ClientStatus::Disconnecting)]
pub enum ClientShutdownStep {
    #[default]
    DisconnectFromServer,
    DespawnLocalClient,
}
