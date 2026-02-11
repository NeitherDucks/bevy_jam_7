use bevy::{input_focus::InputFocus, prelude::*};

use crate::game::{AppState, PlayingState};

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fonts>()
            .init_resource::<InputFocus>()
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(OnExit(AppState::MainMenu), cleanup::<MainMenuTag>)
            .add_systems(OnEnter(AppState::SettingsMenu), setup_settings_menu)
            .add_systems(OnExit(AppState::SettingsMenu), cleanup::<SettingsMenuTag>)
            .add_systems(OnEnter(PlayingState::SettingsMenu), setup_settings_menu)
            .add_systems(
                OnExit(PlayingState::SettingsMenu),
                cleanup::<SettingsMenuTag>,
            )
            .add_systems(OnEnter(PlayingState::Paused), setup_pause_menu)
            .add_systems(OnExit(PlayingState::Paused), cleanup::<PauseMenuTag>)
            .add_systems(OnEnter(AppState::ScoreMenu), setup_score_menu)
            .add_systems(OnExit(AppState::ScoreMenu), cleanup::<ScoreMenuTag>)
            .add_systems(Update, button_system)
            .add_observer(on_play_click)
            .add_observer(on_settings_click)
            .add_observer(on_main_menu_click)
            .add_observer(on_quit_click);
    }
}

#[derive(Resource)]
struct Fonts {
    blue_winter: Handle<Font>,
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        Fonts {
            blue_winter: world.load_asset("blue_winter.ttf"),
        }
    }
}

fn cleanup<C: Component>(mut commands: Commands, menu: Query<Entity, With<C>>) {
    for entity in menu {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
struct MainMenuTag;

fn setup_main_menu(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((MainMenuTag, Camera2d));
    commands.spawn((
        MainMenuTag,
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

fn setup_settings_menu(mut commands: Commands, fonts: Res<Fonts>) {
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
            children![(
                Node {
                    width: percent(50),
                    height: percent(40),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                children![
                    title("Settings", fonts.blue_winter.clone()),
                    button(
                        "Back",
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

#[derive(Component)]
struct PauseMenuTag;

fn setup_pause_menu(mut commands: Commands) {
    commands.spawn((PauseMenuTag, Camera2d));
    // commands.spawn((PauseMenuTag, basic_layout()));
}

#[derive(Component)]
struct ScoreMenuTag;

fn setup_score_menu(mut commands: Commands) {
    commands.spawn((ScoreMenuTag, Camera2d));
    // commands.spawn((ScoreMenuTag, basic_layout()));
}

// ---------------------------------------------------------------------------------

#[derive(Component)]
enum UiEvents {
    Play,
    Settings,
    MainMenu,
    Quit,
}

#[derive(Event)]
struct OnPlayClicked;

fn on_play_click(_event: On<OnPlayClicked>, mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Loading);
}

#[derive(Event)]
struct OnSettingsClicked;

fn on_settings_click(_event: On<OnSettingsClicked>, mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::SettingsMenu);
}

#[derive(Event)]
struct OnMainMenuClicked;

fn on_main_menu_click(_event: On<OnMainMenuClicked>, mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::MainMenu);
}

#[derive(Event)]
struct OnQuitClicked;

fn on_quit_click(_event: On<OnQuitClicked>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

// ---------------------------------------------------------------------------------

fn button(
    text: &'static str,
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

fn title(text: &'static str, font: Handle<Font>) -> impl Bundle {
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
                    UiEvents::Play => commands.trigger(OnPlayClicked),
                    UiEvents::Settings => commands.trigger(OnSettingsClicked),
                    UiEvents::MainMenu => commands.trigger(OnMainMenuClicked),
                    UiEvents::Quit => commands.trigger(OnQuitClicked),
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
