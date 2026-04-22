//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};

use crate::screens::Screen;

/// Key to toggle dev-tools
const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

pub(super) fn plugin(app: &mut App) {
    // Init
    app.add_systems(Startup, init_dev_tools);

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_dev_tools.run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

fn init_dev_tools(mut options: ResMut<UiDebugOptions>, mut config_store: ResMut<GizmoConfigStore>) {
    options.enabled = false;

    let (gizmo_config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    gizmo_config.enabled = false;
}

fn toggle_dev_tools(
    mut options: ResMut<UiDebugOptions>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    options.toggle();

    let (gizmo_config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    gizmo_config.enabled = options.enabled;
}
