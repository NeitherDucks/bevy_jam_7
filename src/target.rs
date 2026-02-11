use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{
    Agent, Agent3dBundle, AgentDesiredVelocity3d, AgentSettings, AgentTarget3d, ArchipelagoRef3d,
};

use crate::{
    game::{AppState, PlayingState, get_random_position_on_navmesh},
    physics::{MovementAcceleration, MovementDampingFactor},
};

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IdleTimer>()
            // .add_systems(OnEnter(AppState::Playing), setup)
            .add_systems(
                Update,
                (
                    (move_agents, tick_idle_timers).run_if(in_state(PlayingState::Playing)),
                    assign_new_target,
                ),
            );
    }
}

#[derive(Component)]
pub struct Target;

#[derive(Component, Clone, Copy)]
pub enum TargetBehavior {
    Mice,
    Skeleton,
}

#[derive(Component, Reflect)]
struct IdleTimer(Timer);

#[derive(Component)]
struct RequestNewTarget;

#[derive(Bundle)]
pub struct TargetBundle {
    mesh: SceneRoot,
    // mat: MeshMaterial3d<StandardMaterial>,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    position_intergration: CustomPositionIntegration,
    marker: Target,
    agent: Agent3dBundle,
    idle: IdleTimer,
    name: Name,
    despawn: DespawnOnExit<AppState>,
}

impl TargetBundle {
    pub fn new(
        mesh: Handle<Scene>,
        // mat: Handle<StandardMaterial>,
        position: Vec3,
        navmesh: Entity,
    ) -> Self {
        Self {
            mesh: SceneRoot(mesh),
            // mat: MeshMaterial3d(mat),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(1.0, 1.0, 1.0),
            acceleration: MovementAcceleration(10.0),
            damping: MovementDampingFactor(0.4),
            position_intergration: CustomPositionIntegration,
            marker: Target,
            agent: Agent3dBundle {
                agent: Agent::default(),
                settings: AgentSettings {
                    radius: 0.5,
                    desired_speed: 30.0,
                    max_speed: 40.0,
                },
                archipelago_ref: ArchipelagoRef3d::new(navmesh),
            },
            idle: IdleTimer(Timer::from_seconds(1.0, TimerMode::Once)),
            name: Name::new("Target"),
            despawn: DespawnOnExit(AppState::Playing),
        }
    }
}

fn move_agents(
    mut commands: Commands,
    agent: Query<(
        Entity,
        &mut LinearVelocity,
        &MovementAcceleration,
        &MovementDampingFactor,
        &AgentDesiredVelocity3d,
        Has<IdleTimer>,
    )>,
) {
    for (entity, mut lin_vel, max_acceleration, damping, desired_vel, has_timer) in agent {
        lin_vel.0 = desired_vel.velocity().normalize_or_zero() * max_acceleration.0;

        let current_speed = lin_vel.length();
        if current_speed > 1.0 {
            lin_vel.0 *= 1.0 - damping.0;
        } else {
            lin_vel.0 = Vec3::ZERO;

            if !has_timer {
                commands
                    .entity(entity)
                    .insert(IdleTimer(Timer::from_seconds(1.5, TimerMode::Once)));
            }
        }
    }
}

fn tick_idle_timers(
    mut commands: Commands,
    query: Query<(Entity, &mut IdleTimer)>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query {
        timer.0.tick(time.delta());

        if timer.0.is_finished() {
            commands
                .entity(entity)
                .remove::<IdleTimer>()
                .insert(RequestNewTarget);
        }
    }
}

fn assign_new_target(
    mut commands: Commands,
    query: Query<Entity, Added<RequestNewTarget>>,
    navmesh: Single<&bevy_landmass::Archipelago3d>,
    mut rng: Single<&mut bevy_prng::ChaCha20Rng, With<bevy_rand::global::GlobalRng>>,
) {
    for entity in query {
        let Some(target) = get_random_position_on_navmesh(&navmesh, &mut rng) else {
            return;
        };

        // assign new target
        commands
            .entity(entity)
            .remove::<RequestNewTarget>()
            .insert(AgentTarget3d::Point(target.point()));
    }
}
