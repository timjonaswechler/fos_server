use {super::AppScope, bevy::prelude::*};

pub(super) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MainMenuContext>()
            .add_observer(handle_main_menu_interaction);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Menu)]
pub enum MainMenuContext {
    #[default]
    Main,
    Singleplayer,
    Multiplayer,
    Wiki,
    Settings,
}

#[derive(Event, Debug, Clone, Copy)]
pub enum MainMenuInteraction {
    SwitchContext(MainMenuContext),
    Exit,
}

fn handle_main_menu_interaction(
    event: On<MainMenuInteraction>,
    mut menu_context: ResMut<NextState<MainMenuContext>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    match *event {
        MainMenuInteraction::SwitchContext(context) => {
            menu_context.set(context);
        }
        MainMenuInteraction::Exit => {
            exit_writer.write(AppExit::Success);
        }
    }
}
