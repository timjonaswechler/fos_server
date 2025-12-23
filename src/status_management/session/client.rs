use {super::super::app::AppScope, super::SessionType, bevy::prelude::*};

pub(super) struct ClientStatusPlugin;

impl Plugin for ClientStatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ClientStatus>()
            .add_observer(on_client_state_event);
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub enum SetClientStatus {
    Transition(ClientStatus),
    Failed,
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
            next_session_type.set(SessionType::None);
            next_app_scope.set(AppScope::Menu);
        }
        SetClientStatus::Transition(state) => {
            next_state.set(state);
        }
        SetClientStatus::Failed => {
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
