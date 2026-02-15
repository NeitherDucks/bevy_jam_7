pub mod anim;
pub mod audio;
pub mod env;
pub mod game;
pub mod god;
pub mod loader;
pub mod menus;
pub mod physics;
pub mod player;
pub mod powerup;
pub mod shuffle;
pub mod target;
pub mod transition;

use bevy::{asset::AssetMetaCheck, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }),
        bevy_framepace::FramepacePlugin,
        bevy_rand::prelude::EntropyPlugin::<bevy_prng::ChaCha20Rng>::default(),
        audio::AudioPlugin,
        loader::LoaderPlugin,
        transition::TransitionPlugin,
        physics::PhysicsPlugin,
        env::EnvironmentPlugin,
        player::PlayerPlugin,
        target::TargetPlugin,
        powerup::PowerupPlugin,
        game::GamePlugin,
        menus::MenusPlugin,
        anim::AnimPlugin,
    ));

    #[cfg(feature = "dev")]
    app.add_plugins((
        bevy::remote::RemotePlugin::default(),
        bevy::remote::http::RemoteHttpPlugin::default(),
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        bevy_inspector_egui::quick::ResourceInspectorPlugin::<game::GameState>::new(),
    ))
    .add_systems(PreStartup, setup_egui);

    app.run();
}

#[cfg(feature = "dev")]
fn setup_egui(mut egui_settings: ResMut<bevy_inspector_egui::bevy_egui::EguiGlobalSettings>) {
    egui_settings.auto_create_primary_context = false;
}
