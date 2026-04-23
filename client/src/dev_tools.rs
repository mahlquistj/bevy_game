//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};
use bevy_fog_of_war::prelude::FogMapSettings;
use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin},
    quick::WorldInspectorPlugin,
};

use crate::screens::Screen;

/// Key to toggle dev-tools
const TOGGLE_KEY: KeyCode = KeyCode::Backquote;
const TOGGLE_FOW_KEY: KeyCode = KeyCode::Backspace;

pub(super) fn plugin(app: &mut App) {
    // Init
    app.add_systems(Startup, init_dev_tools);

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        (
            toggle_dev_tools.run_if(input_just_pressed(TOGGLE_KEY)),
            toggle_fow.run_if(input_just_pressed(TOGGLE_FOW_KEY)),
        ),
    );

    app.add_plugins(EguiPlugin::default());
    app.world_mut()
        .resource_mut::<EguiGlobalSettings>()
        .auto_create_primary_context = false;
    app.add_plugins(WorldInspectorPlugin::new());
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

fn toggle_fow(mut settings: ResMut<FogMapSettings>) {
    settings.enabled = !settings.enabled;
}
