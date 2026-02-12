use bevy::{input_focus::InputFocus, prelude::*};

use crate::game::{AppState, GameSettings, GameState, PlayingState};

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fonts>()
            .init_resource::<InputFocus>()
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(OnEnter(AppState::SettingsMenu), setup_settings_menu)
            .add_systems(OnExit(AppState::SettingsMenu), cleanup::<SettingsMenuTag>)
            .add_systems(
                OnEnter(PlayingState::SettingsMenu),
                setup_playing_settings_menu,
            )
            .add_systems(
                OnExit(PlayingState::SettingsMenu),
                cleanup::<SettingsMenuTag>,
            )
            .add_systems(OnEnter(PlayingState::Paused), setup_pause_menu)
            .add_systems(OnEnter(AppState::ScoreMenu), setup_score_menu)
            .add_systems(Update, button_system)
            .add_observer(on_quit_click)
            .add_observer(on_settings_changed);
    }
}

fn cleanup<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

#[derive(Resource)]
pub struct Fonts {
    pub blue_winter: Handle<Font>,
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        Fonts {
            blue_winter: world.load_asset("blue_winter.ttf"),
        }
    }
}

#[derive(Component)]
struct MainMenuTag;

fn setup_main_menu(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((MainMenuTag, Camera2d, DespawnOnExit(AppState::MainMenu)));
    commands.spawn((
        MainMenuTag,
        DespawnOnExit(AppState::MainMenu),
        (
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            children![(
                Node {
                    width: percent(50),
                    height: percent(40),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                children![
                    title("Dreams for the Fever Gods", fonts.blue_winter.clone()),
                    button("Sleep", fonts.blue_winter.clone(), 300, 75, UiEvents::Play),
                    button(
                        "Settings",
                        fonts.blue_winter.clone(),
                        250,
                        63,
                        UiEvents::Settings
                    ),
                    button(
                        "Leave bed",
                        fonts.blue_winter.clone(),
                        200,
                        50,
                        UiEvents::Quit
                    ),
                ]
            )],
        ),
    ));
}

#[derive(Component)]
struct SettingsMenuTag;

fn setup_settings_menu(mut commands: Commands, fonts: Res<Fonts>, settings: Res<GameSettings>) {
    commands.spawn((SettingsMenuTag, Camera2d));
    commands.spawn((
        SettingsMenuTag,
        (
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            children![settings_ui(&fonts, &settings, UiEvents::MainMenu)],
        ),
    ));
}

fn setup_playing_settings_menu(
    mut commands: Commands,
    fonts: Res<Fonts>,
    settings: Res<GameSettings>,
) {
    commands.spawn((
        SettingsMenuTag,
        (
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            children![settings_ui(&fonts, &settings, UiEvents::Pause)],
        ),
    ));
}

#[allow(clippy::too_many_lines)]
fn settings_ui(fonts: &Fonts, settings: &GameSettings, back: UiEvents) -> impl Bundle {
    (
        Node {
            width: percent(60),
            height: percent(70),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..Default::default()
        },
        children![
            title("Settings", fonts.blue_winter.clone()),
            (
                Node {
                    width: percent(100),
                    height: percent(80),
                    display: Display::Grid,
                    grid_template_columns: vec![
                        RepeatedGridTrack::percent(1, 50.0),
                        RepeatedGridTrack::percent(3, 16.66),
                    ],
                    grid_template_rows: RepeatedGridTrack::px(4, 50.0),
                    row_gap: px(36),
                    column_gap: px(24),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                children![
                    text("Vertical sensitivity", fonts.blue_winter.clone(), 32.0),
                    button(
                        "<",
                        fonts.blue_winter.clone(),
                        50,
                        50,
                        UiEvents::SettingsChange(settings.cam_x(-0.1)),
                    ),
                    text(
                        format!("{:^5.1}", settings.camera_x_sensitivity),
                        fonts.blue_winter.clone(),
                        32.0,
                    ),
                    button(
                        ">",
                        fonts.blue_winter.clone(),
                        50,
                        50,
                        UiEvents::SettingsChange(settings.cam_x(0.1)),
                    ),
                    // -----------------------------------------------------------------
                    text("Horizontal sensitivity", fonts.blue_winter.clone(), 32.0),
                    button(
                        "<",
                        fonts.blue_winter.clone(),
                        50,
                        50,
                        UiEvents::SettingsChange(settings.cam_y(-0.1)),
                    ),
                    text(
                        format!("{:^5.1}", settings.camera_y_sensitivity),
                        fonts.blue_winter.clone(),
                        32.0,
                    ),
                    button(
                        ">",
                        fonts.blue_winter.clone(),
                        50,
                        50,
                        UiEvents::SettingsChange(settings.cam_y(0.1)),
                    ),
                    // -----------------------------------------------------------------
                    text("Music volume", fonts.blue_winter.clone(), 32.0),
                    button(
                        "<",
                        fonts.blue_winter.clone(),
                        50,
                        50,
                        UiEvents::SettingsChange(settings.music(-5.0)),
                    ),
                    text(
                        format!("{:^5.1}", settings.music_volume),
                        fonts.blue_winter.clone(),
                        32.0,
                    ),
                    button(
                        ">",
                        fonts.blue_winter.clone(),
                        50,
                        50,
                        UiEvents::SettingsChange(settings.music(5.0)),
                    ),
                    // -----------------------------------------------------------------
                    text("SFX volume", fonts.blue_winter.clone(), 32.0),
                    button(
                        "<",
                        fonts.blue_winter.clone(),
                        50,
                        50,
                        UiEvents::SettingsChange(settings.sfx(-5.0)),
                    ),
                    text(
                        format!("{:^5.1}", settings.sfx_volume),
                        fonts.blue_winter.clone(),
                        32.0,
                    ),
                    button(
                        ">",
                        fonts.blue_winter.clone(),
                        50,
                        50,
                        UiEvents::SettingsChange(settings.sfx(5.0)),
                    ),
                ],
            ),
            padding(UiRect::bottom(px(50))),
            button("Back", fonts.blue_winter.clone(), 200, 50, back),
        ],
    )
}

#[derive(Component)]
struct PauseMenuTag;

fn setup_pause_menu(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((
        PauseMenuTag,
        DespawnOnExit(PlayingState::Paused),
        (
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            children![(
                Node {
                    width: percent(50),
                    height: percent(40),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                children![
                    title("Pause", fonts.blue_winter.clone()),
                    button(
                        "Back to the dream",
                        fonts.blue_winter.clone(),
                        350,
                        50,
                        UiEvents::Resume
                    ),
                    padding(UiRect::bottom(px(50))),
                    button(
                        "Settings",
                        fonts.blue_winter.clone(),
                        250,
                        50,
                        UiEvents::PlayingSettings
                    ),
                    button(
                        "Wake up",
                        fonts.blue_winter.clone(),
                        250,
                        50,
                        UiEvents::MainMenu
                    ),
                ]
            )],
        ),
    ));
}

#[derive(Component)]
struct ScoreMenuTag;

fn setup_score_menu(mut commands: Commands, fonts: Res<Fonts>, game_state: Res<GameState>) {
    commands.spawn((ScoreMenuTag, Camera2d, DespawnOnExit(AppState::ScoreMenu)));
    commands.spawn((
        ScoreMenuTag,
        DespawnOnExit(AppState::ScoreMenu),
        (
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            children![(
                Node {
                    width: percent(50),
                    height: percent(40),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                children![
                    title("You lost and woke up!", fonts.blue_winter.clone()),
                    text(
                        format!("You got: {} points, well done!", game_state.score),
                        fonts.blue_winter.clone(),
                        36.0
                    ),
                    text("(probably)", fonts.blue_winter.clone(), 24.0),
                    button(
                        "Skip day",
                        fonts.blue_winter.clone(),
                        200,
                        50,
                        UiEvents::MainMenu
                    ),
                ]
            )],
        ),
    ));
}

// ---------------------------------------------------------------------------------

#[derive(Component)]
enum UiEvents {
    Play,
    Settings,
    MainMenu,
    Quit,
    SettingsChange(GameSettings),
    Pause,
    PlayingSettings,
    Resume,
}

#[derive(Event)]
struct OnQuitClicked;

fn on_quit_click(_event: On<OnQuitClicked>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

#[derive(Event)]
struct OnSettingsChanged(GameSettings);

fn on_settings_changed(
    event: On<OnSettingsChanged>,
    mut settings: ResMut<GameSettings>,
    app_state: Option<Res<State<AppState>>>,
    playing_state: Option<Res<State<PlayingState>>>,
    next_app_state: Option<ResMut<NextState<AppState>>>,
    next_playing_state: Option<ResMut<NextState<PlayingState>>>,
) {
    *settings = GameSettings {
        camera_x_sensitivity: event.0.camera_x_sensitivity.clamp(0.0, 2.0),
        camera_y_sensitivity: event.0.camera_y_sensitivity.clamp(0.0, 2.0),
        music_volume: event.0.music_volume.clamp(0.0, 100.0),
        sfx_volume: event.0.sfx_volume.clamp(0.0, 100.0),
    };

    // Force UI to refresh
    if let Some(app_state) = app_state
        && app_state.get() == &AppState::SettingsMenu
        && let Some(mut next_app_state) = next_app_state
    {
        next_app_state.set(AppState::SettingsMenu);
    } else if let Some(playing_state) = playing_state
        && playing_state.get() == &PlayingState::SettingsMenu
        && let Some(mut next_playing_state) = next_playing_state
    {
        next_playing_state.set(PlayingState::SettingsMenu);
    }
}

// ---------------------------------------------------------------------------------

fn button(
    text: impl Into<String>,
    font: Handle<Font>,
    width: u32,
    height: u32,
    event: UiEvents,
) -> impl Bundle {
    (
        Button,
        event,
        Node {
            width: px(width),
            height: px(height),
            border: UiRect::all(px(5)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BorderColor::all(Color::WHITE),
        BackgroundColor(Color::BLACK),
        children![(
            Text::new(text),
            TextFont {
                font,
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
    )
}

fn title(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    (
        Node {
            margin: UiRect::bottom(px(100)),
            padding: UiRect::all(px(10)),
            ..Default::default()
        },
        children![(
            Text::new(text),
            TextFont {
                font,
                font_size: 52.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
    )
}

pub fn text(text: impl Into<String>, font: Handle<Font>, font_size: f32) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(10)),
            ..Default::default()
        },
        children![(
            Text::new(text),
            TextFont {
                font,
                font_size,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
    )
}

// pub fn text_tagged(
//     text: impl Into<String>,
//     font: Handle<Font>,
//     font_size: f32,
//     tag: impl Component,
// ) -> impl Bundle {
//     (
//         Node {
//             padding: UiRect::all(px(10)),
//             ..Default::default()
//         },
//         children![(
//             tag,
//             Text::new(text),
//             TextFont {
//                 font,
//                 font_size,
//                 ..default()
//             },
//             TextColor(Color::srgb(0.9, 0.9, 0.9)),
//             TextShadow::default(),
//         )],
//     )
// }

fn padding(padding: UiRect) -> impl Bundle {
    Node {
        padding,
        ..Default::default()
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.35, 0.35);

fn button_system(
    mut commands: Commands,
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &UiEvents,
        ),
        Changed<Interaction>,
    >,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_playing_state: ResMut<NextState<PlayingState>>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, event) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(Color::linear_rgb(1.0, 0.2, 0.2));

                // The accessibility system's only update the button's state when the `Button` component is marked as changed.
                button.set_changed();

                match event {
                    UiEvents::Play => next_app_state.set(AppState::Loading),
                    UiEvents::Settings => next_app_state.set(AppState::SettingsMenu),
                    UiEvents::MainMenu => next_app_state.set(AppState::MainMenu),
                    UiEvents::Quit => commands.trigger(OnQuitClicked),
                    UiEvents::SettingsChange(game_settings) => {
                        commands.trigger(OnSettingsChanged(*game_settings));
                    }
                    UiEvents::Resume => next_playing_state.set(PlayingState::Playing),
                    UiEvents::PlayingSettings => next_playing_state.set(PlayingState::SettingsMenu),
                    UiEvents::Pause => next_playing_state.set(PlayingState::Paused),
                }
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}
