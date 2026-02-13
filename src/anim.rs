use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{
    game::{AppState, PlayingState, SetupState},
    physics::MovementAcceleration,
    player::Player,
    target::{Target, TargetBehavior},
};

pub struct AnimPlugin;

impl Plugin for AnimPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BoneChain>()
            .add_systems(OnEnter(SetupState::Animation), setup_bone_chain)
            .add_systems(
                Update,
                orient_to_vel.run_if(in_state(PlayingState::Playing)),
            )
            .add_systems(
                PostUpdate,
                update_tail.run_if(in_state(PlayingState::Playing)),
            );
    }
}

#[derive(Reflect, Component)]
struct BoneChain([Entity; 9]);

#[derive(Reflect, Component)]
struct PreviousTransform(Transform);

fn setup_bone_chain(
    mut commands: Commands,
    names: Query<(Entity, &Name)>,
    child_of: Query<&ChildOf>,
    children: Query<&Children>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (entity, name) in names {
        if name.as_str() != "Spine.001.rat" {
            continue;
        }

        info!("\tFound tail bone");

        // Dig up the bone chain and store them
        let mut bones = [entity; 9];

        let Ok(children) = children.get(entity) else {
            warn!("Can't access Spine.001 children");
            continue;
        };

        for child in children {
            let Ok((_, name)) = names.get(*child) else {
                continue;
            };

            match name.as_str() {
                "Spine.002" => bones[1] = *child,
                "Spine.003" => bones[2] = *child,
                "Spine.004" => bones[3] = *child,
                "Tail.001" => bones[4] = *child,
                "Tail.002" => bones[5] = *child,
                "Tail.003" => bones[6] = *child,
                "Tail.004" => bones[7] = *child,
                "Tail.005" => bones[8] = *child,
                _ => {}
            }
        }

        let mut parent = entity;

        // Dig up to the parent
        while let Ok(p) = child_of.get(parent) {
            parent = p.0;
        }

        // Store the bone chain
        commands.entity(parent).insert(BoneChain(bones));
    }

    next_state.set(AppState::Playing);
}

fn orient_to_vel(
    mut query: Query<(
        &mut Transform,
        &LinearVelocity,
        &MovementAcceleration,
        &TargetBehavior,
    )>,
    time: Res<Time>,
) {
    for (mut transform, lin_vel, speed, behavior) in &mut query {
        if lin_vel.0.length_squared() < 1.0 || behavior != &TargetBehavior::Mice {
            continue;
        }

        let vel = lin_vel.0.with_y(0.0).normalize_or_zero();

        let aim = transform.rotation.rotate_towards(
            Quat::from_rotation_arc(Vec3::Z, vel),
            720.0f32.to_radians() * time.delta_secs() * speed.0 * 0.1,
        );

        transform.rotation = aim;
    }
}

// FIX: Compute tail in 2d then convert back to 3d to avoid bone twist, if any
fn update_tail(
    query: Query<(&BoneChain, &MovementAcceleration, &TargetBehavior)>,
    time: Res<Time>,
    mut transforms: Query<(&mut Transform, &GlobalTransform), (Without<Player>, Without<Target>)>,
    mut tail_state: Local<VerletTailState>,
    mut speed: Local<f32>,
) {
    for (bone_chain, max_acceleration, behavior) in &query {
        if behavior != &TargetBehavior::Mice {
            continue;
        }

        let bones = bone_chain.0;

        let Ok((_, &anchor)) = transforms.get_mut(bones[0]) else {
            return;
        };

        if *speed < f32::EPSILON {
            // Not actual speed, but don't want springy rat all the time, so max_accel it is
            *speed = max_acceleration.0;
        }

        // smooth speed change
        *speed = speed.lerp(max_acceleration.0, 10.0 * time.delta_secs());

        // Init state, if needed
        if tail_state.points.len() != bones.len() {
            tail_state.points.clear();
            tail_state.points.reserve(bones.len());

            let bone_transforms = bones
                .iter()
                .filter_map(|bone| {
                    transforms
                        .get_mut(*bone)
                        .ok()
                        .map(|(_, g)| g.compute_transform())
                })
                .collect::<Vec<_>>();

            for (i, transform) in bone_transforms.iter().enumerate() {
                let pos = transform.translation;
                let rotation = transform.rotation;
                tail_state.points.push(VerletPoint {
                    current: pos,
                    previous: pos,
                    rest_rotation: rotation,
                    segment_length: if i == 0 {
                        0.0
                    } else {
                        pos.distance(bone_transforms[i - 1].translation)
                    },
                });
            }
        }

        // Integrate motion
        for i in 1..tail_state.points.len() {
            let p = &mut tail_state.points[i];
            let velocity = p.current - p.previous;
            p.previous = p.current;
            p.current += velocity * time.delta_secs() * 3.0;
        }

        let anchor_translation = anchor.compute_transform().translation;

        // Enforce constraints
        let iterations = 3;

        for _ in 0..iterations {
            tail_state.points[0].current = anchor_translation;

            #[allow(clippy::needless_range_loop)]
            for i in 1..tail_state.points.len() {
                let parent = tail_state.points[i - 1].current;
                let current = tail_state.points[i].current;
                let delta = current - parent;

                tail_state.points[i].current =
                    parent + delta.normalize() * tail_state.points[i].segment_length * *speed * 0.1;
            }
        }

        let first_bone_inverse =
            Transform::from_matrix(anchor.compute_transform().to_matrix().inverse());

        // Write local transforms back to bones
        // We skip the first as it's parented to the body bone, to preserve the head shape
        for (i, bone) in bones[1..].iter().enumerate() {
            let Ok((mut transform, _)) = transforms.get_mut(*bone) else {
                continue;
            };

            let world = Transform::from_translation(tail_state.points[i + 1].current);

            // Set the bones to look down the chain, except last (-2 since we skip the first and last)
            let world = if i < bones.len() - 2 {
                let next = Transform::from_translation(tail_state.points[i + 2].current);
                world.looking_at(next.translation, Vec3::Y)
            } else {
                world
            };

            *transform = first_bone_inverse
                * world
                * Transform::from_rotation(tail_state.points[i].rest_rotation);
        }
    }
}

#[derive(Default)]
struct VerletTailState {
    points: Vec<VerletPoint>,
}

#[derive(Default, Clone)]
struct VerletPoint {
    current: Vec3,
    previous: Vec3,
    rest_rotation: Quat,
    segment_length: f32,
}
