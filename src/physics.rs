use avian3d::{
    math::{AdjustPrecision, AsF32, Scalar},
    prelude::*,
};
use bevy::prelude::*;

use crate::PlayingState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(avian3d::PhysicsPlugins::default());

        #[cfg(feature = "dev")]
        app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin);

        app.add_systems(Startup, disable_physics)
            .add_systems(OnEnter(PlayingState::Playing), enable_physics)
            .add_systems(OnExit(PlayingState::Playing), disable_physics)
            .add_systems(
                FixedUpdate,
                run_move_and_slide.run_if(in_state(PlayingState::Playing)),
            );
    }
}

fn enable_physics(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

fn disable_physics(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

fn run_move_and_slide(
    query: Query<
        (Entity, &mut Transform, &mut LinearVelocity, &Collider),
        With<CustomPositionIntegration>,
    >,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
) {
    for (entity, mut transform, mut lin_vel, collider) in query {
        let MoveAndSlideOutput {
            position,
            projected_velocity,
        } = move_and_slide.move_and_slide(
            collider,
            transform.translation.adjust_precision(),
            transform.rotation.adjust_precision(),
            lin_vel.0,
            time.delta(),
            &MoveAndSlideConfig::default(),
            &SpatialQueryFilter::from_excluded_entities([entity]),
            |_| MoveAndSlideHitResponse::Accept,
        );

        transform.translation = position.f32();
        lin_vel.0 = projected_velocity;
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

#[derive(Component)]
pub struct MovementAcceleration(pub Scalar);

#[derive(Component)]
pub struct MovementDampingFactor(pub Scalar);

#[derive(Component)]
struct JumpImpulse(pub Scalar);

#[derive(Component)]
struct MaxSlopeAngle(pub Scalar);
