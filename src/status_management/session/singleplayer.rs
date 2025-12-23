use {
    super::super::{MainMenuContext, SessionType},
    crate::status_management::AppScope,
    bevy::prelude::*,
};

pub(super) struct SingleplayerStatusPlugin;

impl Plugin for SingleplayerStatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SingleplayerStatus>()
            .add_sub_state::<SingleplayerShutdownStep>()
            .add_observer(on_set_singleplayer_status)
            .add_observer(on_set_singleplayer_shutdown_step);
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SetSingleplayerStatus {
    pub transition: SingleplayerStatus,
}

#[derive(Event, Debug, Clone, Copy)]
pub enum SetSingleplayerShutdownStep {
    Start,
    Next,
    Done,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SessionType = SessionType::Singleplayer)]
pub enum SingleplayerStatus {
    #[default]
    Starting,
    Running,
    Stopping,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SingleplayerStatus = SingleplayerStatus::Stopping)]
pub enum SingleplayerShutdownStep {
    #[default]
    DisconnectRemoteClients,
    CloseRemoteServer,
    DespawnBots,
    DespawnLocalClient,
    DespawnLocalServer,
}

fn on_set_singleplayer_status(
    event: On<SetSingleplayerStatus>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_state: ResMut<NextState<SingleplayerStatus>>,
) {
    match event.transition {
        SingleplayerStatus::Running => {
            next_state.set(SingleplayerStatus::Running);
            next_app_scope.set(AppScope::InGame);
        }
        SingleplayerStatus::Stopping => {
            next_state.set(SingleplayerStatus::Stopping);
        }
        SingleplayerStatus::Starting => {
            next_state.set(SingleplayerStatus::Starting);
        }
    }
}

fn on_set_singleplayer_shutdown_step(
    event: On<SetSingleplayerShutdownStep>,
    shutdown_state: Res<State<SingleplayerShutdownStep>>,
    mut next_main_menu: ResMut<NextState<MainMenuContext>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_state: ResMut<NextState<SingleplayerShutdownStep>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
) {
    match *event {
        SetSingleplayerShutdownStep::Start => {
            next_state.set(SingleplayerShutdownStep::DisconnectRemoteClients);
        }
        SetSingleplayerShutdownStep::Next => match **shutdown_state {
            SingleplayerShutdownStep::DisconnectRemoteClients => {
                next_state.set(SingleplayerShutdownStep::CloseRemoteServer);
            }
            SingleplayerShutdownStep::CloseRemoteServer => {
                next_state.set(SingleplayerShutdownStep::DespawnBots);
            }
            SingleplayerShutdownStep::DespawnBots => {
                next_state.set(SingleplayerShutdownStep::DespawnLocalClient);
            }
            SingleplayerShutdownStep::DespawnLocalClient => {
                next_state.set(SingleplayerShutdownStep::DespawnLocalServer);
            }
            SingleplayerShutdownStep::DespawnLocalServer => {}
        },
        SetSingleplayerShutdownStep::Done => {
            next_app_scope.set(AppScope::Menu);
            next_main_menu.set(MainMenuContext::Main);
            next_session_type.set(SessionType::None);
        }
    }
}
