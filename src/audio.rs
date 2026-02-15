use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_seedling::prelude::*;

use crate::{
    game::{GameSettings, PlayingState},
    loader::{LevelAssetHandles, PermanentAssetHandles, PreLoadAssets},
    menus::ButtonClicked,
    physics::{PlayerHitPowerup, PlayerHitTarget},
    player::PlayerJump,
};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SeedlingPlugin::default())
            .add_systems(OnEnter(PlayingState::Playing), unpause_music)
            .add_systems(OnExit(PlayingState::Playing), pause_music)
            .add_systems(Update, queue_music.run_if(on_timer(Duration::from_secs(5)).and(in_state(PlayingState::Playing))))
            .add_observer(on_button_clicked)
            .add_observer(on_jump)
            .add_observer(on_powerup)
            .add_observer(on_target)
            // .add_observer(on_laser_start)
            ;
    }
}

#[derive(Component)]
struct MusicPlayer;

fn queue_music(
    mut commands: Commands,
    mut level_handles: ResMut<LevelAssetHandles>,
    music_player: Query<&MusicPlayer>,
    mut rng: Single<&mut bevy_prng::ChaCha20Rng, With<bevy_rand::global::GlobalRng>>,
    settings: Res<GameSettings>,
) {
    if music_player.is_empty() {
        info!("Starting new music");
        commands.spawn((
            MusicPlayer,
            SamplePlayer::new(level_handles.musics.next(&mut rng))
                .with_volume(Volume::Linear(settings.music_volume * 0.006)),
        ));
    }
}

fn pause_music(mut music_player: Single<&mut PlaybackSettings, With<MusicPlayer>>) {
    music_player.pause();
}

fn unpause_music(mut music_player: Single<&mut PlaybackSettings, With<MusicPlayer>>) {
    music_player.play();
}

// ----------------------------------------------------------------------------------------

fn on_button_clicked(
    _: On<ButtonClicked>,
    mut commands: Commands,
    handles: Res<PreLoadAssets>,
    settings: Res<GameSettings>,
) {
    commands.spawn(
        SamplePlayer::new(handles.button_sound.clone())
            .with_volume(Volume::Linear(settings.sfx_volume * 0.01)),
    );
}

fn on_jump(
    _: On<PlayerJump>,
    mut commands: Commands,
    handles: Res<PermanentAssetHandles>,
    settings: Res<GameSettings>,
) {
    commands.spawn(
        SamplePlayer::new(handles.jump_sound.clone())
            .with_volume(Volume::Linear(settings.sfx_volume * 0.01)),
    );
}

fn on_powerup(
    _: On<PlayerHitPowerup>,
    mut commands: Commands,
    handles: Res<PermanentAssetHandles>,
    settings: Res<GameSettings>,
) {
    commands.spawn(
        SamplePlayer::new(handles.powerup_sound.clone())
            .with_volume(Volume::Linear(settings.sfx_volume * 0.01)),
    );
}

fn on_target(
    _: On<PlayerHitTarget>,
    mut commands: Commands,
    handles: Res<PermanentAssetHandles>,
    settings: Res<GameSettings>,
) {
    commands.spawn(
        SamplePlayer::new(handles.target_sound.clone())
            .with_volume(Volume::Linear(settings.sfx_volume * 0.01)),
    );
}

// fn on_laser_start(
//     // _: On<LaserStart>,
//     mut commands: Commands,
//     handles: Res<PermanentAssetHandles>,
//     settings: Res<GameSettings>,
// ) {
//     commands.spawn(
//         SamplePlayer::new(handles.laser_sound.clone())
//             .looping()
//             .with_volume(Volume::Linear(settings.sfx_volume * 0.01)),
//     );
// }
