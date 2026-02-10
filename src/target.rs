use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{
    Agent, Agent3dBundle, AgentDesiredVelocity3d, AgentSettings, AgentTarget3d, ArchipelagoRef3d,
};

use crate::{
    AppState, PlayingState,
    env::NavmeshId,
    physics::{MovementAcceleration, MovementDampingFactor},
};

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup)
            .add_systems(Update, move_agents.run_if(in_state(PlayingState::Playing)));
    }
}

#[derive(Component)]
struct Target;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    navmesh: Res<NavmeshId>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(20.0, 0.5, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        MovementAcceleration(10.0),
        MovementDampingFactor(0.4),
        CustomPositionIntegration,
        Name::new("Target"),
        Target,
        Agent3dBundle {
            agent: Agent::default(),
            settings: AgentSettings {
                radius: 0.5,
                desired_speed: 30.0,
                max_speed: 40.0,
            },
            archipelago_ref: ArchipelagoRef3d::new(navmesh.0),
        },
        AgentTarget3d::Point(Vec3::new(0.0, 1.0, 50.0)),
    ));
}

fn move_agents(
    agent: Single<(
        &mut LinearVelocity,
        &MovementAcceleration,
        &MovementDampingFactor,
        &AgentDesiredVelocity3d,
    )>,
) {
    let (mut lin_vel, max_acceleration, damping, desired_vel) = agent.into_inner();

    lin_vel.0 = desired_vel.velocity().normalize_or_zero() * max_acceleration.0;

    let current_speed = lin_vel.length();
    if current_speed > 1.0 {
        lin_vel.0 *= 1.0 - damping.0;
    } else {
        lin_vel.0 = Vec3::ZERO;
    }
}
