use bevy::prelude::*;

use crate::{
    loader::{LevelAssetHandles, LevelDef},
    player::PlayerHitEntities,
    target::{Target, TargetBundle},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .register_type::<GameState>()
            .add_sub_state::<PlayingState>()
            .init_resource::<GameState>()
            .add_systems(OnEnter(AppState::Playing), init_game)
            .add_systems(
                Update,
                (check_for_hit, tick_timer).run_if(in_state(PlayingState::Playing)),
            );

        #[cfg(feature = "dev")]
        app.add_systems(
            Update,
            (
                bevy::dev_tools::states::log_transitions::<AppState>,
                bevy::dev_tools::states::log_transitions::<PlayingState>,
            ),
        );
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    MainMenu,
    SettingsMenu,
    Loading,
    EnvironmentSetup,
    Playing,
    ScoreMenu,
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

#[derive(Resource, Reflect)]
pub struct GameState {
    difficulty: u8,
    score: u32,
    timer: Timer,
}

impl FromWorld for GameState {
    fn from_world(_world: &mut World) -> Self {
        GameState {
            difficulty: 0,
            score: 0,
            timer: Timer::from_seconds(120.0, TimerMode::Once),
        }
    }
}

impl GameState {
    fn next_difficulty(&mut self) {
        self.difficulty += 1;

        let new_duration = (120 - (u64::from(self.difficulty) * 10)).max(30);

        self.timer
            .set_duration(std::time::Duration::from_secs(new_duration));
    }
}

#[derive(Component)]
pub struct Powerup;

fn init_game(
    mut commands: Commands,
    navmesh: Single<(Entity, &bevy_landmass::Archipelago3d)>,
    mut rng: Single<&mut bevy_prng::ChaCha20Rng, With<bevy_rand::global::GlobalRng>>,
    mut state: ResMut<GameState>,
    level_def: Res<LevelDef>,
    handles: Res<LevelAssetHandles>,
) {
    info!("Initializing game ...");

    state.next_difficulty();

    // Spawn targets
    let amount = 10 + (state.difficulty - 1) * 4;

    for _i in 0..amount {
        let mut iter = 0;
        let mut pos = None;
        while pos.is_none() && iter < 100 {
            iter += 1;
            pos = get_random_position_on_navmesh(navmesh.1, &mut rng);
        }

        let Some(pos) = pos else {
            continue;
        };

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
    mut player: Single<&mut PlayerHitEntities>,
    targets: Query<Entity, With<Target>>,
    powerups: Query<Entity, With<Powerup>>,
    mut game_state: ResMut<GameState>,
) {
    for entity in player.0.drain() {
        if targets.contains(entity) {
            game_state.score += 100;
            commands.entity(entity).despawn();
        } else if powerups.contains(entity) {
            // TODO: Power ups
            commands.entity(entity).despawn();
        }
    }
}

pub fn get_random_position_on_navmesh<'a>(
    navmesh: &'a bevy_landmass::Archipelago3d,
    rng: &mut bevy_prng::ChaCha20Rng,
) -> Option<bevy_landmass::SampledPoint<'a, bevy_landmass::coords::ThreeD>> {
    let circle = Circle { radius: 135.0 };
    let new_pos = circle.sample_interior(rng).extend(0.0).xzy();

    navmesh
        .sample_point(
            new_pos,
            &bevy_landmass::PointSampleDistance3d {
                horizontal_distance: 5.0,
                distance_above: 0.5,
                distance_below: 0.5,
                vertical_preference_ratio: 1.0,
                animation_link_max_vertical_distance: 5.0,
            },
        )
        .ok()
}
