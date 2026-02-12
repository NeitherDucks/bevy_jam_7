use bevy::{gltf::GltfMesh, prelude::*};
use bevy_prng::ChaCha20Rng;
use bevy_rerecast::Navmesh;
use rand::seq::SliceRandom;

use crate::{game::AppState, god::GodBehavior, target::TargetBehavior};

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app //
            // .insert_resource(LevelShuffle::new(&[LevelDef::MICE, LevelDef::SKELETON]))
            .insert_resource(LevelShuffle::new(&[LevelDef::MICE]))
            .add_systems(OnEnter(AppState::Loading), load_assets)
            .add_systems(Update, check_load.run_if(in_state(AppState::Loading)))
            .add_systems(OnExit(AppState::Playing), unload_assets);
    }
}

#[derive(Resource)]
struct LevelShuffle {
    default: Vec<LevelDef>,
    remaining: Vec<LevelDef>,
}

#[derive(Resource, Clone, Copy)]
pub struct LevelDef {
    prefix: &'static str,
    pub target_behavior: TargetBehavior,
    pub god_behavior: GodBehavior,
}

impl LevelDef {
    const MICE: LevelDef = LevelDef {
        prefix: "mice",
        target_behavior: TargetBehavior::Mice,
        god_behavior: GodBehavior::Cat,
    };

    const SKELETON: LevelDef = LevelDef {
        prefix: "skel",
        target_behavior: TargetBehavior::Skeleton,
        god_behavior: GodBehavior::Necromencer,
    };
}

impl LevelShuffle {
    fn new(levels: &[LevelDef]) -> Self {
        debug_assert!(!levels.is_empty(), "Levels must not be empty");

        LevelShuffle {
            default: Vec::from(levels),
            remaining: Vec::from(levels),
        }
    }

    fn next(&mut self, rng: &mut ChaCha20Rng) -> LevelDef {
        if self.remaining.is_empty() {
            self.remaining = self.default.clone();
        }

        self.remaining.shuffle(rng);
        self.remaining.pop().unwrap()
    }
}

#[derive(Resource)]
pub struct LevelAssetHandles {
    pub environment: Handle<Scene>,
    pub collisions: Handle<Mesh>,
    pub navmesh: Handle<Navmesh>,
    pub target: Handle<Scene>,
    pub god: Handle<GltfMesh>,
    pub material: Handle<StandardMaterial>,
}

impl LevelAssetHandles {
    fn is_loaded(&self, asset_server: &AssetServer) -> bool {
        let handles = [
            self.environment.clone().untyped(),
            self.collisions.clone().untyped(),
            self.navmesh.clone().untyped(),
        ];

        handles.iter().all(|h| asset_server.is_loaded(h.id()))
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut level_shuffle: ResMut<LevelShuffle>,
    mut rng: Single<&mut bevy_prng::ChaCha20Rng, With<bevy_rand::global::GlobalRng>>,
) {
    info!("Picking level");
    let level_def = level_shuffle.next(&mut rng);

    commands.insert_resource(level_def);

    info!("Loading level");
    let env_path = format!("{}_env.glb", level_def.prefix);
    let col_path = format!("{}_col.glb", level_def.prefix);
    let nav_path = format!("{}_nav.nav", level_def.prefix);
    let tar_path = format!("{}_tar.glb", level_def.prefix);
    let god_path = format!("{}_god.glb", level_def.prefix);

    commands.insert_resource(LevelAssetHandles {
        environment: asset_server.load(GltfAssetLabel::Scene(0).from_asset(env_path)),
        collisions: asset_server.load(
            GltfAssetLabel::Primitive {
                mesh: 0,
                primitive: 0,
            }
            .from_asset(col_path),
        ),
        navmesh: asset_server.load(nav_path),
        target: asset_server.load(GltfAssetLabel::Scene(0).from_asset(tar_path)),
        god: asset_server.load(GltfAssetLabel::Mesh(0).from_asset(god_path)),
        material: asset_server.load(
            GltfAssetLabel::Material {
                index: 0,
                is_scale_inverted: false,
            }
            .from_asset("env.glb"),
        ),
    });
}

fn check_load(
    asset_server: Res<AssetServer>,
    handles: Res<LevelAssetHandles>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if handles.is_loaded(&asset_server) {
        info!("All assets loaded!");
        next_state.set(AppState::EnvironmentSetup);
    }
}

fn unload_assets(mut commands: Commands) {
    info!("Unloading assets");
    commands.remove_resource::<LevelAssetHandles>();
}
