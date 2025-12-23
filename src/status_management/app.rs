use {
    super::{MainMenuContext, SessionType},
    bevy::prelude::*,
};

pub(super) struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppScope>()
            .add_observer(on_change_app_scope);
    }
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppScope {
    #[default]
    Menu,
    InGame,
}

#[derive(Event, Debug)]
pub struct ChangeAppScope {
    pub transition: AppScope,
}

fn on_change_app_scope(
    event: On<ChangeAppScope>,
    mut state: ResMut<NextState<AppScope>>,
    mut menu_state: ResMut<NextState<MainMenuContext>>,
    mut session_type: ResMut<NextState<SessionType>>,
) {
    match event.transition {
        AppScope::Menu => {
            state.set(AppScope::Menu);
            menu_state.set(MainMenuContext::Main);
            session_type.set(SessionType::None);
        }
        _ => {}
    }
}
