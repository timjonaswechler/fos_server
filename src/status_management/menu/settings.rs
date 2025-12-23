use {super::main::MainMenuContext, bevy::prelude::*};

pub(super) struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SettingsMenuScreen>()
            .add_observer(handle_settings_nav);
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsMenuEvent {
    Navigate(SettingsMenuScreen),
    Back,
    Apply,
    Cancel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuContext = MainMenuContext::Settings)]
pub enum SettingsMenuScreen {
    #[default]
    Overview,
    Audio,
    Video,
    Controls,
}

// --- LOGIC HANDLERS ---

fn handle_settings_nav(
    trigger: On<SettingsMenuEvent>,
    mut next_screen: ResMut<NextState<SettingsMenuScreen>>,
    current_screen: Res<State<SettingsMenuScreen>>,
) {
    match trigger.event() {
        SettingsMenuEvent::Navigate(target) => {
            next_screen.set(*target);
        }
        SettingsMenuEvent::Back => {
            if *current_screen.get() != SettingsMenuScreen::Overview {
                next_screen.set(SettingsMenuScreen::Overview);
            } else {
                // If already at overview, maybe go back to Main Menu?
                // This logic typically belongs to the parent menu handler,
                // but we can signal or handle it here if we had access to MenuContext.
                // For now, let's assume 'Back' from Overview is handled by the UI layer or a parent observer.
            }
        }
        SettingsMenuEvent::Apply => {
            // TODO: Apply settings logic
        }
        SettingsMenuEvent::Cancel => {
            next_screen.set(SettingsMenuScreen::Overview);
        }
    }
}
