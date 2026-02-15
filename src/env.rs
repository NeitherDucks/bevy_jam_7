use std::time::Duration;

use avian_rerecast::prelude::*;
use avian3d::prelude::*;
use bevy::{
    gltf::{GltfMesh, GltfNode},
    light::CascadeShadowConfigBuilder,
    prelude::*,
    time::common_conditions::on_timer,
};
use bevy_landmass::{
    Archipelago3d, ArchipelagoOptions, ArchipelagoRef3d, FromAgentRadius, Island, Landmass3dPlugin,
};
#[cfg(feature = "dev")]
use bevy_landmass::{coords::ThreeD, debug::LandmassDebugPlugin};
use bevy_rerecast::prelude::*;
use landmass_rerecast::{Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d};

use crate::{
    game::{AppState, SetupState},
    loader::{LevelAssetHandles, LevelDef},
};

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NavmeshPlugins::default(),
            AvianBackendPlugin::default(),
            Landmass3dPlugin::default(),
            LandmassRerecastPlugin::default(),
        ))
        .add_systems(OnEnter(SetupState::Environment), setup)
        .add_systems(
            Update,
            wait_for_navmesh
                .run_if(on_timer(Duration::from_millis(50)).and(in_state(SetupState::Environment))),
        );

        #[cfg(feature = "dev")]
        app.add_plugins(LandmassDebugPlugin::<ThreeD>::default());
    }
}

#[allow(clippy::too_many_lines)]
fn setup(
    mut commands: Commands,
    gltf: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    meshes: Res<Assets<Mesh>>,
    handles: Res<LevelAssetHandles>,
    level_def: Res<LevelDef>,
) {
    info!("Spawning environment");

    commands.spawn((
        InheritedVisibility::HIDDEN,
        Transform::IDENTITY,
        DespawnOnExit(AppState::Playing),
        Name::new("Groud plane"),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    let gltf = gltf.get(handles.environment.id()).unwrap();

    let material = &gltf.materials[0];

    commands
        .spawn((
            InheritedVisibility::VISIBLE,
            Transform::IDENTITY,
            DespawnOnExit(AppState::Playing),
            Name::new("Environment"),
        ))
        .with_children(|builder| {
            let mut nodes = gltf
                .nodes
                .iter()
                .map(|node| (node, Transform::IDENTITY))
                .collect::<Vec<_>>();

            let mut new_meshes = Vec::new();

            while let Some((node, parent_transform)) = nodes.pop() {
                let Some(node) = gltf_nodes.get(node.id()) else {
                    continue;
                };

                let transform = parent_transform * node.transform;

                if let Some(handle) = &node.mesh
                    && let Some(mesh) = gltf_meshes.get(handle.id())
                {
                    for prim in &mesh.primitives {
                        let (col, hide) = if let Some(extras) = &mesh.extras {
                            (extras.value.contains("col"), extras.value.contains("hide"))
                        } else {
                            (false, false)
                        };

                        new_meshes.push((&prim.mesh, transform, &prim.name, col, hide));
                    }
                }

                for child in &node.children {
                    nodes.push((child, transform));
                }
            }

            for (handle_mesh, transform, name, col, hide) in new_meshes {
                let Some(mesh) = meshes.get(handle_mesh.id()) else {
                    continue;
                };

                let mut entity = builder.spawn((
                    if hide {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    },
                    Name::new(name.clone()),
                    transform,
                    Mesh3d(handle_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    RigidBody::Static,
                ));

                if col {
                    entity.insert(Collider::convex_decomposition_from_mesh(mesh).unwrap());
                }
            }
        });

    // Lights
    info!("Spawning lights");
    commands.insert_resource(level_def.ambient_light.clone());

    commands.spawn((
        level_def.directional_light,
        level_def.directional_light_transform,
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
        DespawnOnExit(AppState::Playing),
    ));

    info!("Spawning navmesh");
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
        Transform::from_xyz(0.0, 0.3, 0.0),
        Island3dBundle {
            island: Island,
            archipelago_ref: ArchipelagoRef3d::new(archipelago),
            nav_mesh: NavMeshHandle3d(handles.navmesh.clone()),
        },
    ));
}

fn wait_for_navmesh(
    navmesh: Single<&bevy_landmass::Archipelago3d>,
    mut next_state: ResMut<NextState<SetupState>>,
) {
    // Repeatedly attempt to sample the navmesh until it becomes available
    if navmesh
        .sample_point(
            Vec3::ZERO,
            &bevy_landmass::PointSampleDistance3d {
                horizontal_distance: 5.0,
                distance_above: 5.0,
                distance_below: 5.0,
                vertical_preference_ratio: 1.0,
                animation_link_max_vertical_distance: 5.0,
            },
        )
        .is_ok()
    {
        info!("Navmesh is available");
        next_state.set(SetupState::Entities);
    }
}
