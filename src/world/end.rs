use bevy::prelude::*;

use crate::{
    ext::*,
    MESSAGE,
    Fonts,
    GameStates, EndStates, CixStates,
    WorldFade,
    Timed,
};

#[derive(Component, Copy, Clone)]
pub struct EndState;
impl EndState {
    pub const TIME: f64 = 2.;
}

#[derive(Component)]
pub struct EndText {
    pub value: Vec<String>,
    pub wait: f64,
    pub wait_page: Option<f64>,
    pub last: Option<char>,
    pub index: usize,
    pub page: usize,
}

pub fn on_end_sys(mut commands: Commands) {
    commands.spawn((
        EndState,
        Timed::new(EndState::TIME),
    ));
}

pub fn end_update_sys(
    end: Query<&Timed, With<EndState>>,
    mut fade: Query<&mut TextureAtlasSprite, With<WorldFade>>,
    mut cix_state: ResMut<NextState<CixStates>>,
    mut end_state: ResMut<NextState<EndStates>>,
    mut game_state: ResMut<NextState<GameStates>>,
) {
    let &timed = end.single();
    let mut fade = fade.single_mut();

    let mut f = timed.fin();
    f = f * f * (3. - 2. * f);
    fade.color = Color::NONE.lerp(Color::BLACK, f);

    if timed.ended() {
        cix_state.set(CixStates::Nonexistent);
        end_state.set(EndStates::Done);
        game_state.set(GameStates::Ending);
    }
}

pub fn game_end_enter_sys(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((
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
        EndText {
            value: MESSAGE.unwrap_or(". . .\nHello, little creature.\nThis is a temporary message.\nPlease, do go on...\n\nGoodbye.")
                .split("\n\n").map(|s| String::from(s))
                .collect(),
            wait: 0.,
            wait_page: None,
            last: None,
            index: 0,
            page: 0,
        },
        TextBundle {
            style: Style {
                size: Size::new(Val::Px(700.), Val::Percent(100.)),
                ..default()
            },
            ..TextBundle::from_section("", TextStyle {
                font: fonts.font.clone_weak(),
                font_size: 24.,
                color: Color::WHITE,
            })
        },
    )); });
}

pub fn game_end_update_sys(
    time: Res<Time>,
    mut text: Query<(&mut EndText, &mut Text)>,
) {
    let current = time.elapsed_seconds_f64();

    let (mut state, mut text) = text.single_mut();
    if state.index >= state.value[state.page].len() {
        if state.wait_page.is_none() {
            state.wait_page = Some(current);
        }

        if state.page < state.value.len() - 1 && current - state.wait_page.unwrap() >= 2. {
            state.wait_page = None;

            state.index = 0;
            state.page += 1;
            state.wait = current;
        }

        return;
    }

    if current - state.wait >= state.last.map(|c|
        if c == '\n' {
            1.
        } else if c.is_alphanumeric() {
            0.02
        } else if c.is_whitespace() {
            0.05
        } else {
            0.04
        }
    ).unwrap_or(0.) {
        state.wait = current;
        state.last = state.value[state.page].chars().skip(state.index).next();
        state.index += 1;

        text.sections[0].value = state.value[state.page][0..state.index].into();
    }
}
