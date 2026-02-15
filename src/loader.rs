use bevy::prelude::*;
use bevy_rerecast::Navmesh;
use bevy_seedling::sample::AudioSample;

use crate::{
    game::{AppState, LoadingState},
    god::GodBehavior,
    shuffle::Shuffle,
    target::TargetBehavior,
};

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PreLoadAssets>()
            .init_resource::<PreLoadAssets>()
            .insert_resource(Shuffle::new(&[LevelDef::MICE]))
            .add_systems(OnEnter(LoadingState::Loading), load_assets)
            .add_systems(Update, check_load.run_if(in_state(LoadingState::Loading)))
            .add_systems(OnExit(AppState::Playing), unload_assets);
    }
}

#[derive(Resource, Clone)]
pub struct LevelDef {
    pub prefix: &'static str,
    pub goal: &'static str,
    pub target_behavior: TargetBehavior,
    pub god_behavior: GodBehavior,
    pub musics: [&'static str; 3],
    pub ambient_light: GlobalAmbientLight,
    pub directional_light: DirectionalLight,
    pub directional_light_transform: Transform,
}

impl LevelDef {
    const MICE: LevelDef = LevelDef {
        prefix: "mice",
        goal: "Mice for the Cat-God",
        target_behavior: TargetBehavior::Mice,
        god_behavior: GodBehavior::Cat,
        musics: [
            "apple_cider-zane_little_music.ogg",
            "nature_sketch-remaxim.ogg",
            "the_secret_within_the_silent_woods-hitctrl.ogg",
        ],
        ambient_light: GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 1200.0,
            affects_lightmapped_meshes: true,
        },
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            shadow_depth_bias: DirectionalLight::DEFAULT_SHADOW_DEPTH_BIAS,
            shadow_normal_bias: DirectionalLight::DEFAULT_SHADOW_NORMAL_BIAS,
            affects_lightmapped_mesh_diffuse: true,
        },
        directional_light_transform: Transform::from_rotation(Quat::from_xyzw(
            -0.5257311, -0.0, -0.0, 0.85065085,
        )),
    };

    // const SKELETON: LevelDef = LevelDef {
    //     prefix: "skel",
    //     goal: "Bones for the Necromancer-God",
    //     target_behavior: TargetBehavior::Skeleton,
    //     god_behavior: GodBehavior::Necromencer,
    // };
}

#[derive(Resource)]
pub struct Fonts {
    pub blue_winter: Handle<Font>,
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        Fonts {
            blue_winter: world.load_asset("fonts/blue_winter.ttf"),
        }
    }
}

#[derive(Reflect, Resource)]
pub struct PreLoadAssets {
    pub eye: Handle<Scene>,
    pub eye_animation_graph: Handle<AnimationGraph>,
    pub eye_close: AnimationNodeIndex,
    pub eye_open: AnimationNodeIndex,
    pub day_bg: Handle<Image>,
    pub night_bg: Handle<Image>,
    pub button_sound: Handle<AudioSample>,
}

impl FromWorld for PreLoadAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let eye = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/eye.glb"));
        let day_bg = asset_server.load("textures/day.png");
        let night_bg = asset_server.load("textures/night.png");
        let button_sound = asset_server.load("sfx/button.wav");

        let (graph, indices) = AnimationGraph::from_clips([
            asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/eye.glb")),
            asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/eye.glb")),
        ]);

        let eye_animation_graph = world
            .get_resource_mut::<Assets<AnimationGraph>>()
            .unwrap()
            .add(graph);

        Self {
            eye,
            eye_animation_graph,
            eye_close: indices[0],
            eye_open: indices[1],
            day_bg,
            night_bg,
            button_sound,
        }
    }
}

#[derive(Resource)]
pub struct PermanentAssetHandles {
    pub player: Handle<Scene>,
    pub cheese: Handle<Scene>,
    pub jump_sound: Handle<AudioSample>,
    pub powerup_sound: Handle<AudioSample>,
    pub target_sound: Handle<AudioSample>,
    pub laser_sound: Handle<AudioSample>,
}

impl PermanentAssetHandles {
    fn are_loaded(&self, asset_server: &AssetServer) -> bool {
        let handles = [
            self.player.clone().untyped(),
            self.cheese.clone().untyped(),
            self.jump_sound.clone().untyped(),
            self.powerup_sound.clone().untyped(),
            self.target_sound.clone().untyped(),
            self.laser_sound.clone().untyped(),
        ];

        handles.iter().all(|h| asset_server.is_loaded(h.id()))
    }
}

#[derive(Resource)]
pub struct LevelAssetHandles {
    pub environment: Handle<Gltf>,
    pub navmesh: Handle<Navmesh>,
    pub target: Handle<Scene>,
    pub god: Handle<Scene>,
    pub musics: Shuffle<Handle<AudioSample>>,
}

impl LevelAssetHandles {
    fn are_loaded(&self, asset_server: &AssetServer) -> bool {
        let handles = [
            self.environment.clone().untyped(),
            self.navmesh.clone().untyped(),
            self.target.clone().untyped(),
        ];

        handles.iter().all(|h| asset_server.is_loaded(h.id()))
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut level_shuffle: ResMut<Shuffle<LevelDef>>,
    mut rng: Single<&mut bevy_prng::ChaCha20Rng, With<bevy_rand::global::GlobalRng>>,
    mut once: Local<bool>,
) {
    if !*once {
        // info!("Loading permanent assets");
        commands.insert_resource(PermanentAssetHandles {
            player: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/player.glb")),
            cheese: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cheese.glb")),
            jump_sound: asset_server.load("sfx/jump.wav"),
            powerup_sound: asset_server.load("sfx/powerup.wav"),
            target_sound: asset_server.load("sfx/target.wav"),
            laser_sound: asset_server.load("sfx/laser.wav"),
        });

        *once = true;
    }

    // info!("Picking level");
    let level_def = level_shuffle.next(&mut rng);

    commands.insert_resource(level_def.clone());

    // info!("Loading level");
    let env_path = format!("levels/{}/environment.glb", level_def.prefix);
    let nav_path = format!("levels/{}/navmesh.nav", level_def.prefix);
    let tar_path = format!("levels/{}/target.glb", level_def.prefix);
    let god_path = format!("levels/{}/god.glb", level_def.prefix);

    let musics = level_def
        .musics
        .iter()
        .map(|music| {
            asset_server.load::<AudioSample>(format!("music/{}/{}", level_def.prefix, music))
        })
        .collect::<Vec<_>>();

    commands.insert_resource(LevelAssetHandles {
        environment: asset_server.load(env_path),
        navmesh: asset_server.load(nav_path),
        target: asset_server.load(GltfAssetLabel::Scene(0).from_asset(tar_path)),
        god: asset_server.load(GltfAssetLabel::Scene(0).from_asset(god_path)),
        musics: Shuffle::new(&musics),
    });
}

fn check_load(
    asset_server: Res<AssetServer>,
    level_handles: Res<LevelAssetHandles>,
    perm_handles: Res<PermanentAssetHandles>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if level_handles.are_loaded(&asset_server) && perm_handles.are_loaded(&asset_server) {
        // info!("All assets loaded!");
        next_state.set(AppState::Setup);
    }
}

fn unload_assets(mut commands: Commands) {
    // info!("Unloading assets");
    commands.remove_resource::<LevelAssetHandles>();
}
