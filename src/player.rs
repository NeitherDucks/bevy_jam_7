use avian3d::{math::AdjustPrecision, prelude::*};
use bevy::{
    platform::collections::HashSet,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_enhanced_input::prelude::*;
// use bevy_landmass::Character3dBundle;

use crate::{
    game::{AppState, PlayingState, SetupState},
    loader::LevelAssetHandles,
    physics::{MovementAcceleration, MovementDampingFactor},
    target::TargetBehavior,
};

pub const PLAYER_DEFAULT_SPEED: f32 = 10.0;
pub const PLAYER_BOOST_SPEED: f32 = 20.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<Player>()
            .add_observer(apply_movement)
            .add_observer(apply_rotation)
            .add_observer(apply_toggle_menu)
            .add_observer(apply_toggle_cursor)
            .add_systems(OnEnter(SetupState::Entities), setup)
            .add_systems(OnEnter(PlayingState::Playing), enable_controls)
            .add_systems(OnExit(PlayingState::Playing), disable_controls)
            .add_systems(
                Update,
                update_camera_pos.run_if(in_state(PlayingState::Playing)),
            );
    }
}

fn setup(mut commands: Commands, handles: Res<LevelAssetHandles>) {
    info!("Spawning Player");
    let mut player = commands.spawn((
        DespawnOnExit(AppState::Playing),
        Transform::from_xyz(0.0, 0.5, 0.0),
        SceneRoot(handles.player.clone()),
        Name::new("Player"),
        Player,
        PlayerHitEntities(HashSet::new()),
        CharacterController,
        RigidBody::Kinematic,
        Collider::cuboid(1.0, 1.0, 1.0),
        CustomPositionIntegration,
        MovementAcceleration(PLAYER_DEFAULT_SPEED),
        MovementDampingFactor(0.4),
        TargetBehavior::Mice,
        // Character3dBundle {
        //     character: todo!(),
        //     settings: todo!(),
        //     archipelago_ref: todo!(),
        // },
        actions!(
            Player[(
                Action::<Movement>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                Bindings::spawn((
                    Cardinal::wasd_keys(), Axial::left_stick()))
            ),
            (
                Action::<Rotate>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_motion(), Scale::new(Vec3::new(0.1, 0.015, 0.1)), Negate::all())),
                    Axial::right_stick().with((Scale::splat(2.0), Negate::x())),
                )),
            ),
            (
                Action::<Jump>::new(),
                bindings![KeyCode::Space, GamepadButton::South]
            ),
            (
                Action::<ToggleMenu>::new(),
                bindings![KeyCode::Escape, GamepadButton::Start, GamepadButton::Select]
            ),
            #[cfg(feature = "dev")]
            (
                Action::<DevToggleMouseGrab>::new(),
                bindings![KeyCode::Tab]
            )
            ]
        ),
    ));

    player.insert(ContextActivity::<Player>::INACTIVE);

    commands.spawn((
        Name::new("Camera anchor"),
        DespawnOnExit(AppState::Playing),
        Transform::IDENTITY,
        PlayerCameraAnchorY,
        children![(
            DespawnOnExit(AppState::Playing),
            Transform::from_rotation(Quat::from_rotation_x(10.0f32.to_radians())),
            PlayerCameraAnchorX,
            children![(
                DespawnOnExit(AppState::Playing),
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
                PlayerCamera,
            )],
        )],
    ));
}

fn enable_controls(
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    commands
        .entity(player.into_inner())
        .insert(ContextActivity::<Player>::ACTIVE);

    grab_cursor(&mut cursor_options, true);
    commands.insert_resource(GrabMousePlease(true));
}

fn disable_controls(
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    commands
        .entity(player.into_inner())
        .insert(ContextActivity::<Player>::INACTIVE);

    grab_cursor(&mut cursor_options, false);
    commands.insert_resource(GrabMousePlease(false));
}

/// Tag for the Player
#[derive(Component)]
pub struct Player;

/// Tag for the Camera
#[derive(Component)]
struct PlayerCamera;

/// Tag for the Y rotation of the orbiting camera (yaw)
///
/// This is used to found out what is forward as well
#[derive(Component)]
struct PlayerCameraAnchorY;

/// Tag for the X rotation of the orbiting camera (pitch)
#[derive(Component)]
struct PlayerCameraAnchorX;

/// Tag for the Movement inputs
#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

/// Tag for the Rotate inputs
#[derive(InputAction)]
#[action_output(Vec2)]
struct Rotate;

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

#[derive(InputAction)]
#[action_output(bool)]
struct DevToggleMouseGrab;

#[derive(InputAction)]
#[action_output(bool)]
struct ToggleMenu;

#[derive(Resource)]
struct GrabMousePlease(bool);

#[derive(Component)]
struct CharacterController;

#[derive(Component)]
pub struct PlayerHitEntities(pub HashSet<Entity>);

fn apply_movement(
    movement: On<Fire<Movement>>,
    player: Single<
        (
            &mut LinearVelocity,
            &MovementAcceleration,
            &MovementDampingFactor,
        ),
        With<Player>,
    >,
    anchor: Single<&Transform, With<PlayerCameraAnchorY>>,
    time: Res<Time>,
    mut speed: Local<f32>,
) {
    let (mut lin_vel, max_acceleration, damping) = player.into_inner();

    if *speed < f32::EPSILON {
        *speed = max_acceleration.0;
    }

    // smooth speed change
    *speed = speed.lerp(max_acceleration.0, 10.0 * time.delta_secs());

    let mut velocity = movement.value.extend(0.0).xzy() * Vec3::new(-1.0, 1.0, 1.0) * *speed;

    velocity = anchor.rotation * velocity;

    lin_vel.0 += velocity.adjust_precision();

    let current_speed = lin_vel.length();
    if current_speed > 1.0 {
        lin_vel.0 *= 1.0 - damping.0;
    } else {
        lin_vel.0 = Vec3::ZERO;
    }
}

fn apply_rotation(
    rotate: On<Fire<Rotate>>,
    mut anchor_y: Single<&mut Transform, (With<PlayerCameraAnchorY>, Without<PlayerCameraAnchorX>)>,
    mut anchor_x: Single<&mut Transform, (With<PlayerCameraAnchorX>, Without<PlayerCameraAnchorY>)>,
    cursor_options: Single<&CursorOptions>,
) {
    // TODO: Camera-Ground interaction
    if cursor_options.visible {
        return;
    }

    let (mut yaw, _, _) = anchor_y.rotation.to_euler(EulerRot::YXZ);
    let (_, mut pitch, _) = anchor_x.rotation.to_euler(EulerRot::YXZ);

    yaw += rotate.value.x.to_radians();
    pitch += rotate.value.y.to_radians();
    pitch = pitch.clamp(-89.0f32.to_radians(), 89.0f32.to_radians());

    anchor_y.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
    anchor_x.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, pitch, 0.0);
}

// fn apply_jump(jump: On<Fire<Jump>>) {
//     // TODO: Jump ??
// }

fn apply_toggle_menu(
    _toggle: On<Start<ToggleMenu>>,
    mut next_state: ResMut<NextState<PlayingState>>,
) {
    next_state.set(PlayingState::Paused);
}

fn apply_toggle_cursor(
    toggle: On<Start<DevToggleMouseGrab>>,
    mut cursor_options: Single<&mut CursorOptions>,
    mut grab: ResMut<GrabMousePlease>,
) {
    if toggle.value {
        grab.0 = !grab.0;
        grab_cursor(&mut cursor_options, grab.0);
    }
}

fn grab_cursor(cursor_options: &mut CursorOptions, grab: bool) {
    cursor_options.grab_mode = if grab {
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::None
    };
    cursor_options.visible = !grab;
}

fn update_camera_pos(
    mut anchor: Single<&mut Transform, (With<PlayerCameraAnchorY>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<PlayerCameraAnchorY>)>,
) {
    // TODO: Slight lerp, so camera needs to catch up to the character
    anchor.translation = player.translation;
}
