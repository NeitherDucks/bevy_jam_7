use avian3d::{
    math::{AsF32, Scalar},
    prelude::*,
};
use bevy::prelude::*;

use crate::{game::PlayingState, player::Player, powerup::Powerup, target::Target};

pub const DAMP_FACTOR: f32 = 0.3;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(PhysicsPlugins::default());

        #[cfg(feature = "dev")]
        app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin);

        app.add_systems(Startup, disable_physics)
            .add_systems(OnEnter(PlayingState::Playing), enable_physics)
            .add_systems(OnExit(PlayingState::Playing), disable_physics)
            .add_systems(
                Update,
                (update_grounded, apply_movement_damping)
                    .chain()
                    .run_if(in_state(PlayingState::Playing)),
            )
            .add_systems(FixedUpdate, run_move_and_slide.run_if(is_physics_enabled))
            .add_observer(check_hit_powerup);
    }
}

fn is_physics_enabled(time: Res<Time<Physics>>) -> bool {
    !time.is_paused()
}

fn enable_physics(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

fn disable_physics(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

#[derive(Event)]
pub struct PlayerHitPowerup(pub Entity);

#[derive(Event)]
pub struct PlayerHitTarget(pub Entity);

fn check_hit_powerup(
    trigger: On<CollisionStart>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    powerups: Query<Entity, With<Powerup>>,
) {
    // Only the player and power ups have they CollisionEventsEnabled

    // Make sure we don't delete the wrong entity
    if trigger.collider1 == *player && powerups.contains(trigger.collider2) {
        commands.trigger(PlayerHitPowerup(trigger.collider2));
    } else if trigger.collider2 == *player && powerups.contains(trigger.collider1) {
        commands.trigger(PlayerHitPowerup(trigger.collider1));
    }
}

fn run_move_and_slide(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &Collider,
            Has<Player>,
        ),
        With<CustomPositionIntegration>,
    >,
    targets: Query<Entity, With<Target>>,
    move_and_slide: MoveAndSlide,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut transform, mut lin_vel, collider, is_player) in query {
        let mut velocity = lin_vel.0;

        velocity.y += -9.8 * 5.0 * time.delta_secs();

        let MoveAndSlideOutput {
            position,
            projected_velocity,
        } = move_and_slide.move_and_slide(
            collider,
            transform.translation,
            transform.rotation,
            velocity,
            time.delta(),
            &MoveAndSlideConfig {
                move_and_slide_iterations: 2,
                planes: vec![Dir3::Y],
                ..Default::default()
            },
            &SpatialQueryFilter::from_excluded_entities([entity]),
            |hit| {
                if is_player && targets.contains(hit.entity) {
                    commands.trigger(PlayerHitTarget(hit.entity));
                }

                MoveAndSlideHitResponse::Accept
            },
        );

        transform.translation = position.f32();
        lin_vel.0 = projected_velocity;

        // In case the player or target drops out of the map somehow
        if transform.translation.y < -100.0 {
            transform.translation = Vec3::ZERO;
            lin_vel.0 = Vec3::ZERO;
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

#[derive(Reflect, Component)]
pub struct MovementAcceleration {
    pub current: f32,
    pub target: f32,
}

impl MovementAcceleration {
    pub fn new(target: f32) -> Self {
        Self {
            current: target,
            target,
        }
    }
}

#[derive(Component)]
pub struct MovementDampingFactor(pub Scalar);

#[derive(Component)]
pub struct MaxSlopeAngle(pub Scalar);

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        Or<(With<Player>, With<Target>)>,
    >,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        let is_grounded = hits.iter().any(|hit| {
            if hit.distance > 0.5 {
                return false;
            }

            if let Some(angle) = max_slope_angle {
                (rotation * -hit.normal2).angle_between(Vec3::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(
    mut query: Query<(&MovementDampingFactor, &mut LinearVelocity, &Grounded)>,
) {
    for (damping_factor, mut linear_velocity, _) in &mut query {
        linear_velocity.x *= damping_factor.0;
        linear_velocity.z *= damping_factor.0;
    }
}
