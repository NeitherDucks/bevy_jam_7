use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
    loader::{LevelAssetHandles, LevelDef},
    menus::Fonts,
    physics::MovementAcceleration,
    player::{PLAYER_BOOST_SPEED, Player, PlayerHitEntities},
    powerup::{Powerup, PowerupBundle, PowerupTimer},
    target::{Target, TargetBundle},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .register_type::<GameState>()
            .add_sub_state::<PlayingState>()
            .add_sub_state::<SetupState>()
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
                    check_for_hit,
                    tick_timer,
                    update_ui.run_if(on_timer(Duration::from_millis(100))),
                    spawn_powerup.run_if(on_timer(Duration::from_secs(15))),
                )
                    .run_if(in_state(PlayingState::Playing)),
            )
            .add_systems(OnEnter(PlayingState::GameOver), game_over);

        #[cfg(feature = "dev")]
        app.add_systems(
            Update,
            (
                bevy::dev_tools::states::log_transitions::<AppState>,
                bevy::dev_tools::states::log_transitions::<PlayingState>,
                bevy::dev_tools::states::log_transitions::<SetupState>,
            ),
        );
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    MainMenu,
    SettingsMenu,
    #[default]
    Loading,
    Setup,
    Playing,
    ScoreMenu,
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
            height: percent(100),
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
                        margin: UiRect::bottom(px(100)),
                        padding: UiRect::all(px(10)),
                        ..Default::default()
                    },
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
            height: percent(100),
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
                        margin: UiRect::bottom(px(100)),
                        padding: UiRect::all(px(10)),
                        ..Default::default()
                    },
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
    mut timer_ui: Single<&mut Text, (With<TimerUi>, Without<TargetsUi>)>,
    mut targets_ui: Single<&mut Text, (With<TargetsUi>, Without<TimerUi>)>,
    game_state: Res<GameState>,
) {
    timer_ui.0 = format!("{:.0}", game_state.timer.remaining_secs());
    targets_ui.0 = format!(
        "{} / {}",
        game_state.aquired_targets, game_state.total_targets
    );
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

fn check_for_hit(
    mut commands: Commands,
    mut player: Single<(Entity, &mut PlayerHitEntities)>,
    targets: Query<Entity, With<Target>>,
    powerups: Query<Entity, With<Powerup>>,
    mut acceleration: Query<&mut MovementAcceleration>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let player_entity = player.0;
    for entity in player.1.0.drain() {
        if targets.contains(entity) {
            game_state.aquired_targets += 1;
            game_state.score += 100;
            commands.entity(entity).despawn();

            if game_state.aquired_targets == game_state.total_targets {
                info!("Player won the round!");
                next_state.set(AppState::Loading);
            }
        } else if powerups.contains(entity) {
            commands
                .entity(player_entity)
                .insert(PowerupTimer::default());

            if let Ok(mut acceleration) = acceleration.get_mut(player_entity) {
                acceleration.target = PLAYER_BOOST_SPEED;
            }

            commands.entity(entity).despawn();
        }
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
