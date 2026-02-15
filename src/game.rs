use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_tweening::{Tween, TweenAnim, lens::UiTransformScaleLens};

use crate::{
    loader::{LevelAssetHandles, LevelDef},
    menus::Fonts,
    physics::{MovementAcceleration, PlayerHitPowerup, PlayerHitTarget},
    player::{PLAYER_BOOST_SPEED, Player},
    powerup::{PowerupBundle, PowerupTimer},
    target::TargetBundle,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .register_type::<GameState>()
            .add_sub_state::<MenuState>()
            .add_sub_state::<LoadingState>()
            .add_sub_state::<SetupState>()
            .add_sub_state::<PlayingState>()
            .init_resource::<GameState>()
            .init_resource::<GameSettings>()
            .add_systems(
                OnTransition {
                    exited: AppState::MainMenu,
                    entered: AppState::Loading,
                },
                reset_game,
            )
            .add_systems(OnEnter(SetupState::Entities), (init_game, setup_ui))
            .add_systems(
                Update,
                (
                    tick_timer,
                    update_ui.run_if(on_timer(Duration::from_millis(100))),
                    spawn_powerup.run_if(on_timer(Duration::from_secs(15))),
                )
                    .run_if(in_state(PlayingState::Playing)),
            )
            .add_systems(OnEnter(PlayingState::GameOver), game_over)
            .add_observer(on_player_hit_powerup)
            .add_observer(on_player_hit_target)
            // .add_observer(check_collision_with_target)
        ;

        #[cfg(feature = "dev")]
        app.add_systems(
            Update,
            (
                bevy::dev_tools::states::log_transitions::<AppState>,
                bevy::dev_tools::states::log_transitions::<MenuState>,
                bevy::dev_tools::states::log_transitions::<LoadingState>,
                bevy::dev_tools::states::log_transitions::<SetupState>,
                bevy::dev_tools::states::log_transitions::<PlayingState>,
            ),
        );
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    MainMenu,
    Loading,
    Setup,
    Playing,
    ScoreMenu,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppState = AppState::MainMenu)]
pub enum MenuState {
    #[default]
    Main,
    Settings,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppState = AppState::Loading)]
pub enum LoadingState {
    #[default]
    TransitionIn,
    Waiting,
    TransitionOut,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppState = AppState::Setup)]
pub enum SetupState {
    #[default]
    Environment,
    Entities,
    Animation,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(AppState = AppState::Playing)]
pub enum PlayingState {
    #[default]
    Starting,
    Playing,
    Paused,
    SettingsMenu,
    GameOver,
}

#[derive(Resource, Clone, Copy)]
pub struct GameSettings {
    pub camera_x_sensitivity: f32,
    pub camera_y_sensitivity: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            camera_x_sensitivity: 1.0,
            camera_y_sensitivity: 0.2,
            music_volume: 50.0,
            sfx_volume: 50.0,
        }
    }
}

impl GameSettings {
    pub fn cam_x(&self, v: f32) -> Self {
        Self {
            camera_x_sensitivity: self.camera_x_sensitivity + v,
            ..*self
        }
    }

    pub fn cam_y(&self, v: f32) -> Self {
        Self {
            camera_y_sensitivity: self.camera_y_sensitivity + v,
            ..*self
        }
    }

    pub fn music(&self, v: f32) -> Self {
        Self {
            music_volume: self.music_volume + v,
            ..*self
        }
    }

    pub fn sfx(&self, v: f32) -> Self {
        Self {
            sfx_volume: self.sfx_volume + v,
            ..*self
        }
    }
}

#[derive(Debug, Resource, Reflect)]
pub struct GameState {
    pub difficulty: u8,
    pub score: u32,
    pub timer: Timer,
    pub total_targets: u8,
    pub aquired_targets: u8,
}

impl FromWorld for GameState {
    fn from_world(_world: &mut World) -> Self {
        GameState {
            difficulty: 0,
            score: 0,
            timer: Timer::from_seconds(120.0, TimerMode::Once),
            total_targets: 0,
            aquired_targets: 0,
        }
    }
}

impl GameState {
    fn next_difficulty(&mut self) {
        self.difficulty += 1;
        self.total_targets = 10 + (self.difficulty - 1) * 4;
        self.aquired_targets = 0;

        let new_duration = (120 - (u64::from(self.difficulty) * 10)).max(30);

        self.timer
            .set_duration(std::time::Duration::from_secs(new_duration));
        self.timer.reset();
    }
}

fn reset_game(mut commands: Commands) {
    info!("Resetting game");
    commands.remove_resource::<GameState>();
    commands.init_resource::<GameState>();
}

fn init_game(
    mut commands: Commands,
    navmesh: Single<(Entity, &bevy_landmass::Archipelago3d)>,
    mut rng: Single<&mut bevy_prng::ChaCha20Rng, With<bevy_rand::global::GlobalRng>>,
    mut state: ResMut<GameState>,
    level_def: Res<LevelDef>,
    handles: Res<LevelAssetHandles>,
    mut next_state: ResMut<NextState<SetupState>>,
) {
    info!("Picking difficulty");
    state.next_difficulty();

    // Spawn targets
    info!("Spawning targets:");
    for i in 0..state.total_targets {
        info!("\t Target {}", i);
        let mut iter = 0;
        let mut pos = Err(bevy_landmass::SamplePointError::OutOfRange);
        while pos.is_err() && iter < 100 {
            iter += 1;
            pos = get_random_position_on_navmesh(Vec3::ZERO, 135.0, navmesh.1, &mut rng);
        }

        let pos = match pos {
            Ok(p) => p,
            Err(err) => {
                warn!("Could not spawn target {}: {}", i, err);
                continue;
            }
        };

        info!("\t Target {} spawned!", i);
        commands.spawn((
            TargetBundle::new(
                handles.target.clone(),
                // handles.material.clone(),
                pos.point(),
                navmesh.0,
            ),
            level_def.target_behavior,
        ));
    }

    next_state.set(SetupState::Animation);
}

#[derive(Component)]
struct TimerUi;

#[derive(Component)]
struct TargetsUi;

fn setup_ui(mut commands: Commands, fonts: Res<Fonts>) {
    info!("Setting up UI");
    commands.spawn((
        GlobalZIndex(-1),
        DespawnOnExit(AppState::Playing),
        Node {
            width: percent(100),
            ..Default::default()
        },
        children![
            // Targets
            (
                Node {
                    left: px(25),
                    top: px(25),
                    ..Default::default()
                },
                children![(
                    Node {
                        padding: UiRect::all(px(10)),
                        width: px(150),
                        border_radius: BorderRadius::all(px(50)),
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.8)),
                    children![(
                        TargetsUi,
                        Text::new(""),
                        TextFont {
                            font: fonts.blue_winter.clone(),
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextShadow::default(),
                    )],
                )]
            )
        ],
    ));
    commands.spawn((
        GlobalZIndex(-1),
        DespawnOnExit(AppState::Playing),
        Node {
            width: percent(100),
            ..Default::default()
        },
        children![
            // Timer
            (
                Node {
                    margin: auto().horizontal(),
                    top: px(25),
                    ..Default::default()
                },
                children![(
                    Node {
                        padding: UiRect::all(px(10)),
                        width: px(200),
                        border_radius: BorderRadius::all(px(50)),
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.8)),
                    children![(
                        TimerUi,
                        Text::new(""),
                        TextFont {
                            font: fonts.blue_winter.clone(),
                            font_size: 52.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextShadow::default(),
                    )],
                )]
            )
        ],
    ));
}

fn update_ui(
    mut commands: Commands,
    mut timer_ui: Single<(Entity, &mut Text, &mut TextColor), (With<TimerUi>, Without<TargetsUi>)>,
    mut targets_ui: Single<(Entity, &mut Text), (With<TargetsUi>, Without<TimerUi>)>,
    game_state: Res<GameState>,
    mut once: Local<bool>,
) {
    timer_ui.1.0 = format!("{:.0}", game_state.timer.remaining_secs());
    if game_state.timer.remaining_secs() <= 10.0 {
        timer_ui.2.0 = Color::linear_rgb(0.95, 0.05, 0.05);

        if !*once {
            *once = true;

            commands
                .entity(timer_ui.0)
                .insert(TweenAnim::new(Tween::new(
                    EaseFunction::CubicIn,
                    Duration::from_secs(10),
                    UiTransformScaleLens {
                        start: Vec2::ONE,
                        end: Vec2::splat(5.0),
                    },
                )));
        }
    } else {
        *once = false;
    }
    let new = format!(
        "{} / {}",
        game_state.aquired_targets, game_state.total_targets
    );

    if targets_ui.1.0 != new {
        // add animation
        commands.entity(targets_ui.0).insert(TweenAnim::new(
            Tween::new(
                EaseFunction::BounceInOut,
                Duration::from_secs_f32(0.25),
                UiTransformScaleLens {
                    start: Vec2::splat(1.5),
                    end: Vec2::ONE,
                },
            )
            // .with_repeat_strategy(bevy_tweening::RepeatStrategy::MirroredRepeat)
            .with_repeat_count(1),
        ));
    }

    targets_ui.1.0 = new;
}

fn tick_timer(
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<PlayingState>>,
    time: Res<Time>,
) {
    game_state.timer.tick(time.delta());

    if game_state.timer.is_finished() {
        next_state.set(PlayingState::GameOver);
    }
}

fn on_player_hit_powerup(
    trigger: On<PlayerHitPowerup>,
    mut commands: Commands,
    mut player: Single<(Entity, &mut MovementAcceleration), With<Player>>,
) {
    commands.entity(trigger.0).despawn();

    commands.entity(player.0).insert(PowerupTimer::default());
    player.1.target = PLAYER_BOOST_SPEED;
}

fn on_player_hit_target(
    trigger: On<PlayerHitTarget>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    commands.entity(trigger.0).despawn();

    game_state.aquired_targets += 1;
    game_state.score += 100;

    if game_state.aquired_targets == game_state.total_targets {
        info!("Player won the round!");
        next_state.set(AppState::Loading);
    }
}

fn game_over(mut next_state: ResMut<NextState<AppState>>) {
    info!("Game over!");
    next_state.set(AppState::ScoreMenu);
}

fn spawn_powerup(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    navmesh: Single<(Entity, &bevy_landmass::Archipelago3d)>,
    mut rng: Single<&mut bevy_prng::ChaCha20Rng, With<bevy_rand::global::GlobalRng>>,
    handles: Res<LevelAssetHandles>,
) {
    let mut iter = 0;
    let mut pos = Err(bevy_landmass::SamplePointError::OutOfRange);
    while pos.is_err() && iter < 100 {
        iter += 1;
        pos = get_random_position_on_navmesh(player.translation, 50.0, navmesh.1, &mut rng);
    }

    let pos = match pos {
        Ok(p) => p,
        Err(err) => {
            warn!("Could not spawn powerup: {}", err);
            return;
        }
    };

    info!("\t Powerup spawned!");
    commands.spawn(PowerupBundle::new(
        SceneRoot(handles.target.clone()),
        pos.point(),
        Name::new("Powerup"),
    ));
}

pub fn get_random_position_on_navmesh<'a>(
    center: Vec3,
    radius: f32,
    navmesh: &'a bevy_landmass::Archipelago3d,
    rng: &mut bevy_prng::ChaCha20Rng,
) -> Result<
    bevy_landmass::SampledPoint<'a, bevy_landmass::coords::ThreeD>,
    bevy_landmass::SamplePointError,
> {
    let circle = Circle { radius };
    let new_pos = circle.sample_interior(rng).extend(0.0).xzy() + center;

    navmesh.sample_point(
        new_pos,
        &bevy_landmass::PointSampleDistance3d {
            horizontal_distance: radius * 0.5,
            distance_above: 1.0,
            distance_below: 1.0,
            vertical_preference_ratio: 0.0,
            animation_link_max_vertical_distance: 5.0,
        },
    )
}
