pub mod env;
pub mod game;
pub mod god;
pub mod loader;
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
        bevy_rand::prelude::EntropyPlugin::<bevy_prng::ChaCha20Rng>::default(),
        loader::LoaderPlugin,
        physics::PhysicsPlugin,
        env::EnvironmentPlugin,
        player::PlayerPlugin,
        target::TargetPlugin,
        game::GamePlugin,
    ));

    #[cfg(feature = "dev")]
    app.add_plugins((
        bevy::remote::RemotePlugin::default(),
        bevy::remote::http::RemoteHttpPlugin::default(),
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        bevy_inspector_egui::quick::ResourceInspectorPlugin::<game::GameState>::default(),
    ));

    app.run();
}
