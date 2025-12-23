use {super::main::MainMenuContext, bevy::prelude::*};

pub(super) struct WikiMenuPlugin;

impl Plugin for WikiMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<WikiMenuScreen>()
            .add_observer(handle_wiki_nav);
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiMenuEvent {
    Navigate(WikiMenuScreen),
    Back,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuContext = MainMenuContext::Wiki)]
pub enum WikiMenuScreen {
    #[default]
    Overview,
}

// --- LOGIC HANDLERS ---

fn handle_wiki_nav(trigger: On<WikiMenuEvent>, mut next_screen: ResMut<NextState<WikiMenuScreen>>) {
    match trigger.event() {
        WikiMenuEvent::Navigate(target) => {
            next_screen.set(*target);
        }
        WikiMenuEvent::Back => {
            // Placeholder for back navigation logic if Wiki gets more screens
        }
    }
}
