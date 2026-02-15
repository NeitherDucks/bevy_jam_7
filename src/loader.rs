use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::TextureFormat,
    ui::FocusPolicy,
};
use bevy_prng::ChaCha20Rng;
use bevy_rerecast::Navmesh;
use rand::seq::SliceRandom;

use crate::{
    anim::{IgnorePlayingState, PlayAnimation},
    game::{AppState, LoadingState, PlayingState},
    god::GodBehavior,
    target::TargetBehavior,
};

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PreLoadAssets>()
            .init_resource::<PreLoadAssets>()
            // .insert_resource(LevelShuffle::new(&[LevelDef::MICE, LevelDef::SKELETON]))
            .insert_resource(LevelShuffle::new(&[LevelDef::MICE]))
            .add_systems(Startup, setup_eye)
            .add_systems(OnEnter(LoadingState::TransitionIn), animation_in)
            .add_systems(OnEnter(LoadingState::Waiting), load_assets)
            .add_systems(Update, check_animation_finished)
            .add_systems(Update, check_load.run_if(in_state(LoadingState::Waiting)))
            .add_systems(OnEnter(AppState::Playing), animation_out)
            .add_systems(OnExit(AppState::Playing), unload_assets)
            .add_observer(transition_in_finished)
            .add_observer(transition_out_finished)
            .add_observer(update_eye_children);
    }
}

#[derive(Resource)]
struct LevelShuffle {
    default: Vec<LevelDef>,
    remaining: Vec<LevelDef>,
}

#[derive(Resource, Clone, Copy)]
pub struct LevelDef {
    pub prefix: &'static str,
    pub goal: &'static str,
    pub target_behavior: TargetBehavior,
    pub god_behavior: GodBehavior,
}

impl LevelDef {
    const MICE: LevelDef = LevelDef {
        prefix: "mice",
        goal: "Mice for the Cat-God",
        target_behavior: TargetBehavior::Mice,
        god_behavior: GodBehavior::Cat,
    };

    // const SKELETON: LevelDef = LevelDef {
    //     prefix: "skel",
    //     goal: "Bones for the Necromancer-God",
    //     target_behavior: TargetBehavior::Skeleton,
    //     god_behavior: GodBehavior::Necromencer,
    // };
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

#[derive(Reflect, Resource)]
pub struct PreLoadAssets {
    pub eye: Handle<Scene>,
    pub eye_animation_graph: Handle<AnimationGraph>,
    pub eye_close: AnimationNodeIndex,
    pub eye_open: AnimationNodeIndex,
}

impl FromWorld for PreLoadAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let eye = asset_server.load(GltfAssetLabel::Scene(0).from_asset("eye.glb"));

        let (graph, indices) = AnimationGraph::from_clips([
            asset_server.load(GltfAssetLabel::Animation(0).from_asset("eye.glb")),
            asset_server.load(GltfAssetLabel::Animation(1).from_asset("eye.glb")),
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
        }
    }
}

#[derive(Resource)]
pub struct PermanentAssetHandles {
    pub player: Handle<Scene>,
    pub cheese: Handle<Scene>,
}

impl PermanentAssetHandles {
    fn are_loaded(&self, asset_server: &AssetServer) -> bool {
        let handles = [self.player.clone().untyped()];

        handles.iter().all(|h| asset_server.is_loaded(h.id()))
    }
}

#[derive(Resource)]
pub struct LevelAssetHandles {
    pub environment: Handle<Gltf>,
    pub navmesh: Handle<Navmesh>,
    pub target: Handle<Scene>,
    pub god: Handle<Scene>,
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
    mut level_shuffle: ResMut<LevelShuffle>,
    mut rng: Single<&mut bevy_prng::ChaCha20Rng, With<bevy_rand::global::GlobalRng>>,
    mut once: Local<bool>,
) {
    if !*once {
        info!("Loading permanent assets");
        commands.insert_resource(PermanentAssetHandles {
            player: asset_server.load(GltfAssetLabel::Scene(0).from_asset("player.glb")),
            cheese: asset_server.load(GltfAssetLabel::Scene(0).from_asset("cheese.glb")),
        });

        *once = true;
    }

    info!("Picking level");
    let level_def = level_shuffle.next(&mut rng);

    commands.insert_resource(level_def);

    info!("Loading level");
    let env_path = format!("{}_env.glb", level_def.prefix);
    let nav_path = format!("{}_nav.nav", level_def.prefix);
    let tar_path = format!("{}_tar.glb", level_def.prefix);
    let god_path = format!("{}_god.glb", level_def.prefix);

    commands.insert_resource(LevelAssetHandles {
        environment: asset_server.load(env_path),
        navmesh: asset_server.load(nav_path),
        target: asset_server.load(GltfAssetLabel::Scene(0).from_asset(tar_path)),
        god: asset_server.load(GltfAssetLabel::Scene(0).from_asset(god_path)),
    });
}

fn check_load(
    asset_server: Res<AssetServer>,
    level_handles: Res<LevelAssetHandles>,
    perm_handles: Res<PermanentAssetHandles>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if level_handles.are_loaded(&asset_server) && perm_handles.are_loaded(&asset_server) {
        info!("All assets loaded!");
        next_state.set(AppState::Setup);
    }
}

fn unload_assets(mut commands: Commands) {
    info!("Unloading assets");
    commands.remove_resource::<LevelAssetHandles>();
}

// ----------------------------------------------------------------------------

#[derive(Component)]
pub struct EyeCamera;

#[derive(Component)]
struct Eye;

#[derive(Component, Event)]
struct EyeOpen;

#[derive(Component, Event)]
struct EyeClosed;

const EYE_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

fn setup_eye(mut commands: Commands, window: Single<&Window>, mut images: ResMut<Assets<Image>>) {
    let image = Image::new_target_texture(
        window.resolution.physical_width(),
        window.resolution.physical_height(),
        TextureFormat::Rgba8UnormSrgb,
        Some(TextureFormat::Rgba8UnormSrgb),
    );

    let image_handle = images.add(image);

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..Default::default()
        },
        Transform::IDENTITY,
        EyeCamera,
        EYE_RENDER_LAYER,
        RenderTarget::Image(image_handle.clone().into()),
        Name::new("Eye Camera"),
    ));

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                ..Default::default()
            },
            ZIndex(9999),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ))
        .with_children(|command| {
            command.spawn((
                ImageNode {
                    image: image_handle.clone(),
                    ..Default::default()
                },
                FocusPolicy::Pass,
                Pickable::IGNORE,
            ));
        });
}

fn animation_in(mut commands: Commands, assets: Res<PreLoadAssets>) {
    info!("Spawning Eye");
    // Spawn Eye
    commands.spawn((
        Eye,
        EyeClosed,
        SceneRoot(assets.eye.clone()),
        PlayAnimation {
            graph: assets.eye_animation_graph.clone(),
            index: assets.eye_close,
        },
        IgnorePlayingState,
        EYE_RENDER_LAYER,
        Name::new("Eye"),
    ));
}

fn update_eye_children(trigger: On<Add, Mesh3d>, mut commands: Commands, names: Query<&Name>) {
    if let Ok(name) = names.get(trigger.entity)
        && name.as_str() == "Cylinder.014.EyeMaterial"
    {
        commands.entity(trigger.entity).insert(EYE_RENDER_LAYER);
    }
}

fn animation_out(
    mut commands: Commands,
    eye: Single<Entity, With<Eye>>,
    assets: Res<PreLoadAssets>,
) {
    commands
        .entity(*eye)
        .insert((
            EyeOpen,
            PlayAnimation {
                graph: assets.eye_animation_graph.clone(),
                index: assets.eye_open,
            },
            IgnorePlayingState,
        ))
        .remove::<EyeClosed>();
}

fn check_animation_finished(
    mut commands: Commands,
    players: Query<(Entity, &AnimationPlayer)>,
    child_of: Query<&ChildOf>,
    open: Query<Entity, With<EyeOpen>>,
    closed: Query<Entity, With<EyeClosed>>,
) {
    for (entity, player) in &players {
        if player.all_finished() {
            if child_of
                .iter_ancestors(entity)
                .any(|parent| open.contains(parent))
            {
                commands.trigger(EyeOpen);
            }

            if child_of
                .iter_ancestors(entity)
                .any(|parent| closed.contains(parent))
            {
                commands.trigger(EyeClosed);
            }
        }
    }
}

fn transition_in_finished(_: On<EyeClosed>, mut next_state: ResMut<NextState<LoadingState>>) {
    next_state.set(LoadingState::Waiting);
}

fn transition_out_finished(
    _: On<EyeOpen>,
    mut commands: Commands,
    eye: Single<Entity, With<Eye>>,
    mut next_state: ResMut<NextState<PlayingState>>,
) {
    info!("Transition out finished");

    commands.entity(*eye).despawn();
    next_state.set(PlayingState::Playing);
}
