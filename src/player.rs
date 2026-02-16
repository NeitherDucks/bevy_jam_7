use avian3d::prelude::*;
use bevy::{
    platform::collections::HashSet,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_enhanced_input::prelude::*;
// use bevy_landmass::Character3dBundle;

use crate::{
    game::{AppState, GameSettings, PlayingState, SetupState},
    loader::PermanentAssetHandles,
    physics::{DAMP_FACTOR, Grounded, MaxSlopeAngle, MovementAcceleration, MovementDampingFactor},
    target::TargetBehavior,
};

pub const PLAYER_DEFAULT_SPEED: f32 = 10.0;
pub const PLAYER_BOOST_SPEED: f32 = PLAYER_DEFAULT_SPEED * 2.0;
pub const PLAYER_SPEED_FACTOR: f32 = 1.0 / PLAYER_DEFAULT_SPEED;
pub const JUMP_IMPULSE: f32 = 25.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<Player>()
            .add_systems(OnEnter(SetupState::Entities), setup)
            .add_systems(OnEnter(PlayingState::Playing), enable_controls)
            .add_systems(OnExit(PlayingState::Playing), disable_controls)
            .add_systems(
                Update,
                (update_camera_pos).run_if(in_state(AppState::Playing)),
            )
            .add_observer(apply_movement)
            .add_observer(apply_rotation)
            .add_observer(apply_jump)
            .add_observer(apply_toggle_menu)
            .add_observer(apply_toggle_cursor);
    }
}

fn setup(mut commands: Commands, handles: Res<PermanentAssetHandles>) {
    // info!("Spawning Player");

    let collider = Collider::capsule_endpoints(
        0.5,
        Vec3::new(0.0, 0.5 * 0.5, -0.2),
        Vec3::new(0.0, 0.5 * 0.5, -1.0),
    );
    let mut caster_shape = collider.clone();
    caster_shape.set_scale(Vec3::ONE * 0.99, 10);

    commands.spawn((
        DespawnOnExit(AppState::Playing),
        Transform::from_xyz(0.0, 0.5 * 0.5, 0.0),
        SceneRoot(handles.player.clone()),
        Name::new("Player"),
        Player,
        PlayerHitEntities(HashSet::new()),
        CharacterController,
        (
            RigidBody::Kinematic,
            collider,
            ShapeCaster::new(caster_shape, Vec3::ZERO, Quat::IDENTITY, Dir3::NEG_Y),
            CollisionEventsEnabled,
            CustomPositionIntegration,
            MovementAcceleration::new(PLAYER_DEFAULT_SPEED),
            MovementDampingFactor(DAMP_FACTOR),
            LockedAxes::new().lock_rotation_x().lock_rotation_z(),
            MaxSlopeAngle(35.0f32.to_radians()),
        ),
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
            // #[cfg(feature = "dev")]
            // (
            //     Action::<DevToggleMouseGrab>::new(),
            //     bindings![KeyCode::Tab]
            // )
            ]
        ),
        ContextActivity::<Player>::INACTIVE
    ));

    commands.spawn((
        Name::new("Camera anchor"),
        DespawnOnExit(AppState::Playing),
        Transform::IDENTITY,
        PlayerCameraAnchorY,
        children![(
            Transform::from_rotation(Quat::from_rotation_x(10.0f32.to_radians())),
            PlayerCameraAnchorX,
            children![(
                Transform::from_xyz(0.0, 0.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
                Name::new("Player Camera"),
                Camera3d::default(),
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
pub struct Jump;

#[derive(Event)]
pub struct PlayerJump;

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
            &mut MovementAcceleration,
            Has<Grounded>,
        ),
        With<Player>,
    >,
    anchor: Single<&Transform, With<PlayerCameraAnchorY>>,
    time: Res<Time>,
) {
    let (mut lin_vel, mut max_acceleration, is_grounded) = player.into_inner();

    // smooth speed change
    max_acceleration.current = max_acceleration
        .current
        .lerp(max_acceleration.target, time.delta_secs());

    // If not on the ground stop processing inputs
    if !is_grounded {
        return;
    }

    let mut velocity = movement.value.extend(0.0).xzy().normalize_or_zero()
        * Vec3::new(-1.0, 1.0, 1.0)
        * max_acceleration.current;

    velocity = anchor.rotation * velocity;

    lin_vel.0.x += velocity.x;
    lin_vel.0.z += velocity.z;
}

fn apply_rotation(
    rotate: On<Fire<Rotate>>,
    mut anchor_y: Single<&mut Transform, (With<PlayerCameraAnchorY>, Without<PlayerCameraAnchorX>)>,
    mut anchor_x: Single<&mut Transform, (With<PlayerCameraAnchorX>, Without<PlayerCameraAnchorY>)>,
    cursor_options: Single<&CursorOptions>,
    settings: Res<GameSettings>,
) {
    if cursor_options.visible {
        return;
    }

    let (mut yaw, _, _) = anchor_y.rotation.to_euler(EulerRot::YXZ);
    let (_, mut pitch, _) = anchor_x.rotation.to_euler(EulerRot::YXZ);

    yaw += rotate.value.x.to_radians() * settings.camera_x_sensitivity;
    pitch += rotate.value.y.to_radians() * settings.camera_y_sensitivity;
    pitch = pitch.clamp(3.0f32.to_radians(), 89.0f32.to_radians());

    anchor_y.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
    anchor_x.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, pitch, 0.0);
}

fn apply_jump(
    _: On<Start<Jump>>,
    mut commands: Commands,
    player: Single<(&mut LinearVelocity, Has<Grounded>), With<Player>>,
) {
    let (mut velocity, is_grounded) = player.into_inner();

    if is_grounded {
        // info!("Player jumped");
        velocity.y += JUMP_IMPULSE;
        commands.trigger(PlayerJump);
    }
}

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
        CursorGrabMode::Locked
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
