use bevy::prelude::*;

use crate::{
    PRELUDE,
    Fonts, GameStates,
};

#[derive(Component)]
pub struct WorldPrelude;

#[derive(Component)]
pub struct PreludeText {
    pub value: String,
    pub wait: f64,
    pub last: Option<char>,
    pub index: usize,
}

pub fn prelude_enter_sys(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((
        WorldPrelude,
        NodeBundle {
            style: Style {
                size: Size::all(Val::Percent(100.)),
                padding: UiRect::all(Val::Px(32.)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..default()
        },
    )).with_children(|builder| { builder.spawn((
        PreludeText {
            value: PRELUDE.into(),
            wait: 0.,
            last: None,
            index: 0,
        },
        TextBundle {
            style: Style {
                size: Size::new(Val::Px(700.), Val::Percent(100.)),
                ..default()
            },
            ..TextBundle::from_section("", TextStyle {
                font: fonts.font.clone_weak(),
                font_size: 32.,
                color: Color::WHITE,
            })
        },
    )); });
}

pub fn prelude_update_sys(
    time: Res<Time>, input: Res<Input<KeyCode>>,
    mut game_state: ResMut<NextState<GameStates>>,
    mut text: Query<(&mut PreludeText, &mut Text)>,
) {
    let (mut state, mut text) = text.single_mut();
    if state.index >= state.value.len() {
        if input.just_pressed(KeyCode::Return) {
            game_state.set(GameStates::Gameplay);
        }

        return;
    }

    let current = time.elapsed_seconds_f64();
    if current - state.wait >= state.last.map(|c|
        if c == '\n' {
            1.
        } else if c == '.' {
            0.6
        } else if c.is_alphanumeric() {
            0.03
        } else if c.is_whitespace() {
            0.075
        } else {
            0.05
        }
    ).unwrap_or(0.) {
        state.wait = current;
        state.last = state.value.chars().skip(state.index).next();
        state.index += 1;

        text.sections[0].value = state.value[0..state.index].into();
    }
}

pub fn prelude_exit_sys(mut commands: Commands, preludes: Query<Entity, With<WorldPrelude>>) {
    for prelude in &preludes {
        commands.entity(prelude).despawn_recursive();
    }
}
