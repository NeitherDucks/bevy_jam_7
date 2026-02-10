use avian_rerecast::prelude::*;
use avian3d::prelude::*;
use bevy::{asset::LoadState, prelude::*};
use bevy_landmass::{
    Archipelago3d, ArchipelagoOptions, ArchipelagoRef3d, FromAgentRadius, Island, Landmass3dPlugin,
};
use bevy_rerecast::{debug::DetailNavmeshGizmo, prelude::*};
use landmass_rerecast::{Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d};

use crate::AppState;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NavmeshPlugins::default(),
            AvianBackendPlugin::default(),
            Landmass3dPlugin::default(),
            LandmassRerecastPlugin::default(),
        ))
        .add_systems(OnEnter(AppState::Loading), load_assets)
        .add_systems(Update, check_load.run_if(in_state(AppState::Loading)))
        .add_systems(OnExit(AppState::Loading), setup);
    }
}

#[derive(Resource)]
struct AssetHandles {
    env: Handle<Scene>,
    collisions: Handle<Mesh>,
    navmesh: Handle<Navmesh>,
}

#[derive(Resource)]
pub struct NavmeshId(pub Entity);

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AssetHandles {
        env: asset_server.load(GltfAssetLabel::Scene(0).from_asset("env.glb")),
        collisions: asset_server.load(
            GltfAssetLabel::Primitive {
                mesh: 0,
                primitive: 0,
            }
            .from_asset("env_coll.glb"),
        ),
        navmesh: asset_server.load("navmesh.nav"),
    });
}

fn check_load(
    asset_server: Res<AssetServer>,
    handles: Res<AssetHandles>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(LoadState::Loaded) = asset_server.get_load_state(handles.env.id())
        && let Some(LoadState::Loaded) = asset_server.get_load_state(handles.collisions.id())
        && let Some(LoadState::Loaded) = asset_server.get_load_state(handles.navmesh.id())
    {
        next_state.set(AppState::Playing);
    }
}

fn setup(mut commands: Commands, meshes: ResMut<Assets<Mesh>>, handles: Res<AssetHandles>) {
    commands.spawn(SceneRoot(handles.env.clone()));

    let collision_mesh = meshes.get(handles.collisions.id()).unwrap();

    commands.spawn((
        Transform::IDENTITY,
        InheritedVisibility::HIDDEN,
        RigidBody::Static,
        Collider::trimesh_from_mesh(collision_mesh).unwrap(),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 800.0,
        ..Default::default()
    });

    // Lights
    commands.spawn((
        DirectionalLight {
            illuminance: 35000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let archipelago = commands
        .spawn((
            Name::new("Navmesh archipelago"),
            DespawnOnExit(AppState::Playing),
            Archipelago3d::new(ArchipelagoOptions::from_agent_radius(1.0)),
        ))
        .id();

    commands.spawn((
        Name::new("NavMesh island"),
        DespawnOnExit(AppState::Playing),
        Island3dBundle {
            island: Island,
            archipelago_ref: ArchipelagoRef3d::new(archipelago),
            nav_mesh: NavMeshHandle3d(handles.navmesh.clone()),
        },
    ));

    commands.spawn(DetailNavmeshGizmo::new(&handles.navmesh));

    commands.insert_resource(NavmeshId(archipelago));
}
