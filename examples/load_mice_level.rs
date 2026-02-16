use avian3d::prelude::*;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        bevy::remote::RemotePlugin::default(),
        bevy::remote::http::RemoteHttpPlugin::default(),
        avian3d::PhysicsPlugins::default(),
        bevy_rerecast::NavmeshPlugins::default(),
        avian_rerecast::AvianBackendPlugin::default(),
        bevy_landmass::Landmass3dPlugin::default(),
        landmass_rerecast::LandmassRerecastPlugin::default(),
    ))
    .init_resource::<Handles>()
    .add_systems(Startup, setup)
    .add_systems(Update, add_collider);

    app.run();
}

#[derive(Resource)]
struct Handles {
    env: Handle<Scene>,
}

impl FromWorld for Handles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        Handles {
            env: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("levels/mice/environment_col.glb")),
        }
    }
}

fn setup(mut commands: Commands, handles: Res<Handles>) {
    commands.spawn(SceneRoot(handles.env.clone()));
    commands.spawn(Camera3d::default());
    commands.spawn(DirectionalLight::default());
}

fn add_collider(
    mut commands: Commands,
    query: Query<(Entity, &Mesh3d), Added<Mesh3d>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, mesh3d) in &query {
        let Some(mesh) = meshes.get(mesh3d.0.id()) else {
            continue;
        };

        commands.entity(entity).insert((
            Collider::trimesh_from_mesh(mesh).unwrap(),
            RigidBody::Static,
        ));
    }
}
