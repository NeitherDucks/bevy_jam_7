use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;

use crate::{
    game::{AppState, PlayingState, SetupState},
    physics::MovementAcceleration,
    player::PLAYER_SPEED_FACTOR,
    target::TargetBehavior,
};

pub struct AnimPlugin;

impl Plugin for AnimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TweeningPlugin)
            .register_type::<BoneChain>()
            .add_systems(OnEnter(SetupState::Animation), setup_bone_chain)
            .add_systems(
                Update,
                (
                    on_play_animation,
                    orient_to_vel.run_if(in_state(PlayingState::Playing)),
                ),
            )
            .add_systems(
                PostUpdate,
                update_tail.run_if(in_state(PlayingState::Playing)),
            );
    }
}

// ------------------------------------------------------------------------------------------------------

#[derive(Reflect, Component)]
pub struct PlayAnimation {
    pub graph: Handle<AnimationGraph>,
    pub index: AnimationNodeIndex,
}

#[derive(Component)]
pub struct IgnorePlayingState;

fn on_play_animation(
    mut commands: Commands,
    query: Query<(Entity, &PlayAnimation, Has<IgnorePlayingState>)>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    playing_state: Option<Res<State<PlayingState>>>,
) {
    let playing_state = playing_state
        .filter(|state| **state == PlayingState::Playing)
        .is_some();

    for (entity, animation, force) in &query {
        if !force && !playing_state {
            continue;
        }

        for child in children.iter_descendants(entity) {
            if let Ok(mut player) = players.get_mut(child) {
                player.play(animation.index);

                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation.graph.clone()));
                commands.entity(entity).remove::<PlayAnimation>();
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------------

#[derive(Reflect, Component)]
struct MainBone(Entity, Quat);

#[derive(Reflect, Component)]
struct BoneChain([Entity; 9]);

fn setup_bone_chain(
    mut commands: Commands,
    names: Query<(Entity, &Name)>,
    child_of: Query<&ChildOf>,
    children: Query<&Children>,
    global_transforms: Query<&GlobalTransform>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (entity, name) in names {
        if name.as_str() != "Spine.001.rat" {
            continue;
        }

        info!("\tFound spine bone");

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

        // Init verlet tail state
        let mut points = Vec::with_capacity(bones.len());

        let bone_transforms = bones
            .iter()
            .map(|bone| {
                global_transforms
                    .get(*bone)
                    .expect("Bones should already exists")
                    .compute_transform()
            })
            .collect::<Vec<_>>();

        for (i, transform) in bone_transforms.iter().enumerate() {
            let pos = transform.translation;
            let rotation = transform.rotation;
            points.push(VerletPoint {
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

        let main_rotation = global_transforms
            .get(entity)
            .unwrap()
            .compute_transform()
            .rotation;

        // Store the bone chain
        commands
            .entity(parent)
            .insert(MainBone(entity, main_rotation))
            .insert(BoneChain(bones))
            .insert(VerletTailState { points });
    }

    next_state.set(AppState::Playing);
}

fn orient_to_vel(
    mut query: Query<(
        &mut Transform,
        &LinearVelocity,
        &MovementAcceleration,
        &TargetBehavior,
        &MainBone,
    )>,
    mut bones: Query<&mut Transform, Without<MainBone>>,
    time: Res<Time>,
) {
    for (mut transform, lin_vel, speed, behavior, main_bone) in &mut query {
        if lin_vel.0.length_squared() < 1.0 || behavior != &TargetBehavior::Mice {
            continue;
        }

        let vel = lin_vel.0.with_y(0.0).normalize_or_zero();

        let aim = transform.rotation.rotate_towards(
            Quat::from_rotation_arc(Vec3::Z, vel),
            520.0f32.to_radians() * time.delta_secs() * speed.current * PLAYER_SPEED_FACTOR,
        );

        transform.rotation = aim;

        if let Ok(mut head_bone) = bones.get_mut(main_bone.0) {
            let rot = head_bone.rotation.rotate_towards(
                // in world space
                Quat::from_rotation_arc(Vec3::Z, lin_vel.0.normalize_or_zero())
                // to body space
                    * transform.rotation.inverse()
                    // to bone space
                    * main_bone.1,
                360f32.to_radians() * time.delta_secs() * speed.current * PLAYER_SPEED_FACTOR,
            );
            // only keep X rotation
            let (x, _, _) = rot.to_euler(EulerRot::XYZ);
            head_bone.rotation = Quat::from_euler(EulerRot::XYZ, x, 0.0, 0.0);
        }
    }
}

// FIX: Switch to Avian physics chain
//      - Spawn entity per joint
//      - Add SphericalJoint constraint
//      - On update, apply joint entity transform to join (in local space)
fn update_tail(
    mut query: Query<(
        &BoneChain,
        &MovementAcceleration,
        &TargetBehavior,
        &mut VerletTailState,
    )>,
    time: Res<Time>,
    mut transforms: Query<(&mut Transform, &GlobalTransform)>,
) {
    for (bone_chain, speed, behavior, mut tail_state) in &mut query {
        if behavior != &TargetBehavior::Mice {
            continue;
        }

        let bones = bone_chain.0;

        let Ok((_, &anchor)) = transforms.get_mut(bones[0]) else {
            return;
        };

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
            // FIXME: Since the velocity is only difference between the previous frame and the current
            //        if the player stops, after a frame the velocity drops to 0
            //        we could keep a portion of it, this'll add overshoot
            //        but it'll also means the tail might fall into the ground
            //        since there is no collision check
            let velocity = p.current - p.previous;
            p.previous = p.current;
            p.current += velocity * time.delta_secs() * 5.0;
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

                tail_state.points[i].current = parent
                    + delta.normalize()
                        * tail_state.points[i].segment_length
                        * speed.current
                        * PLAYER_SPEED_FACTOR;
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

#[derive(Default, Component)]
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
