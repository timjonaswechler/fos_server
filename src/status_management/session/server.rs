use {super::SessionType, bevy::prelude::*};

pub(super) struct ServerStatusPlugin;

impl Plugin for ServerStatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ServerVisibility>()
            .add_observer(on_server_visibility_event);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SessionType = SessionType::Singleplayer)]
pub enum ServerVisibility {
    #[default]
    Private,
    PendingPublic,
    GoingPublic,
    Public,
    GoingPrivate,
    Failed,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SetServerVisibility {
    pub transition: ServerVisibility,
}

fn on_server_visibility_event(
    event: On<SetServerVisibility>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    //TODO: going public only when in Game Menu or PendingPublic State before
    match event.transition {
        state => {
            next_state.set(state);
        }
    }
}
