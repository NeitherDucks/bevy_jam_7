pub mod env;
pub mod physics;
pub mod player;
pub mod target;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }),
        bevy_framepace::FramepacePlugin,
        physics::PhysicsPlugin,
        env::EnvironmentPlugin,
        player::PlayerPlugin,
        target::TargetPlugin,
    ))
    .init_state::<AppState>()
    .add_sub_state::<PlayingState>();

    #[cfg(feature = "dev")]
    app.add_plugins((
        bevy::remote::RemotePlugin::default(),
        bevy::remote::http::RemoteHttpPlugin::default(),
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
    ));
    #[cfg(feature = "dev")]
    app.add_systems(
        Update,
        (
            bevy::dev_tools::states::log_transitions::<AppState>,
            bevy::dev_tools::states::log_transitions::<PlayingState>,
        ),
    );

    app.run();
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    MainMenu,
    #[default]
    Loading,
    Playing,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppState = AppState::Playing)]
enum PlayingState {
    #[default]
    Playing,
    Paused,
}
