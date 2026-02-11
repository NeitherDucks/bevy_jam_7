use avian_rerecast::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{
    Archipelago3d, ArchipelagoOptions, ArchipelagoRef3d, FromAgentRadius, Island, Landmass3dPlugin,
};
#[cfg(feature = "dev")]
use bevy_landmass::{coords::ThreeD, debug::LandmassDebugPlugin};
use bevy_rerecast::prelude::*;
use landmass_rerecast::{Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d};

use crate::{game::AppState, loader::LevelAssetHandles};

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NavmeshPlugins::default(),
            AvianBackendPlugin::default(),
            Landmass3dPlugin::default(),
            LandmassRerecastPlugin::default(),
        ))
        .add_systems(OnEnter(AppState::EnvironmentSetup), setup);

        #[cfg(feature = "dev")]
        app.add_plugins(LandmassDebugPlugin::<ThreeD>::default());
    }
}

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    handles: Res<LevelAssetHandles>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    commands.spawn(SceneRoot(handles.environment.clone()));

    let collision_mesh = meshes.get(handles.collisions.id()).unwrap();

    commands.spawn((
        Transform::IDENTITY,
        InheritedVisibility::HIDDEN,
        RigidBody::Static,
        Collider::trimesh_from_mesh(collision_mesh).unwrap(),
        DespawnOnExit(AppState::Playing),
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
        DespawnOnExit(AppState::Playing),
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

    next_state.set(AppState::Playing);
}
