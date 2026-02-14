use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    game::{AppState, PlayingState},
    physics::MovementAcceleration,
    player::{PLAYER_BOOST_SPEED, PLAYER_DEFAULT_SPEED},
};

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (tick_timers, animate, check_pickup).run_if(in_state(PlayingState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Powerup;

#[derive(Component)]
struct DespawnTimer(Timer);

#[derive(Component)]
pub struct PowerupTimer(pub Timer);

impl Default for PowerupTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Once))
    }
}

#[derive(Bundle)]
pub struct PowerupBundle {
    marker: Powerup,
    collider: Collider,
    collision_event_enabled: CollisionEventsEnabled,
    mesh: SceneRoot,
    transform: Transform,
    name: Name,
    despawn: DespawnOnExit<AppState>,
    despawn_timer: DespawnTimer,
}

impl PowerupBundle {
    pub fn new(mesh: SceneRoot, position: Vec3, name: Name) -> Self {
        Self {
            marker: Powerup,
            collider: Collider::sphere(1.0),
            collision_event_enabled: CollisionEventsEnabled,
            mesh,
            transform: Transform::from_translation(position),
            name,
            despawn: DespawnOnExit(AppState::Playing),
            despawn_timer: DespawnTimer(Timer::from_seconds(10.0, TimerMode::Once)),
        }
    }
}

fn check_pickup(
    mut message: MessageReader<CollisionStart>,
    mut commands: Commands,
    player: Query<&crate::player::Player>,
    power_ups: Query<&Powerup>,
    mut acceleration: Query<&mut MovementAcceleration>,
) {
    for (contact, _) in message.par_read() {
        let (player, powerup) =
            if player.contains(contact.collider1) && power_ups.contains(contact.collider2) {
                (contact.collider1, contact.collider2)
            } else if player.contains(contact.collider2) && power_ups.contains(contact.collider1) {
                (contact.collider2, contact.collider1)
            } else {
                continue;
            };

        info!("Hit between player and powerup!");

        commands.entity(powerup).despawn();
        commands.entity(player).insert(PowerupTimer::default());

        if let Ok(mut acceleration) = acceleration.get_mut(player) {
            acceleration.target = PLAYER_BOOST_SPEED;
        }
    }
}

fn tick_timers(
    mut commands: Commands,
    mut despawn_timer: Query<(Entity, &mut DespawnTimer)>,
    mut powerup_timer: Query<(Entity, &mut PowerupTimer)>,
    mut acceleration: Query<&mut MovementAcceleration>,
    time: Res<Time>,
) {
    for (entity, mut timer) in &mut despawn_timer {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut timer) in &mut powerup_timer {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            commands.entity(entity).remove::<PowerupTimer>();

            if let Ok(mut acceleration) = acceleration.get_mut(entity) {
                acceleration.target = PLAYER_DEFAULT_SPEED;
            }
        }
    }
}

fn animate(mut powerup: Query<(&mut Transform, &DespawnTimer), With<Powerup>>, time: Res<Time>) {
    for (mut transform, timer) in &mut powerup {
        transform.translation.y = timer.0.elapsed_secs().sin() + 1.5;
        transform.rotate_local_y(5.0 * time.delta_secs());
    }
}
