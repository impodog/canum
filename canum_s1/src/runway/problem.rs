use std::collections::HashSet;

use super::*;

pub(super) struct ProblemPlugin;

impl Plugin for ProblemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RUNWAY_STATE.clone()), init_problem);
        app.add_systems(OnExit(RUNWAY_STATE.clone()), quit_problem);
        app.add_systems(
            PreUpdate,
            spawn_problem.run_if(in_state(RUNWAY_STATE.clone())),
        );
        app.add_systems(
            Update,
            update_problem.run_if(in_state(RUNWAY_STATE.clone())),
        );
    }
}

#[derive(Component)]
#[require(Text2d)]
struct Problem {
    answer: i32,
}

#[derive(Component)]
#[require(TextSpan)]
struct ProblemInputBox;

#[derive(Component)]
#[require(Sprite)]
struct ProblemTimer {
    timer: Timer,
}
impl ProblemTimer {
    const TIMER_LENGTH: f32 = 300.0;
    const TIMER_WIDTH: f32 = 10.0;
}

#[derive(Debug, Resource)]
struct ProblemGenerator {
    used_numbers: HashSet<i32>,
    timer: Timer,
    timeout: Duration,
}
impl Default for ProblemGenerator {
    fn default() -> Self {
        Self {
            used_numbers: Default::default(),
            timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            timeout: Duration::from_secs_f32(15.0),
        }
    }
}
impl ProblemGenerator {
    fn generate_digit_even() -> i32 {
        use rand::seq::IndexedRandom;
        [
            (1, 3.0),
            (2, 3.0),
            (3, 2.5),
            (4, 2.5),
            (5, 2.0),
            (6, 2.0),
            (7, 1.5),
            (8, 1.0),
            (9, 1.0),
        ]
        .choose_weighted(&mut rand::rng(), |(_, w)| *w)
        .unwrap()
        .0
    }
    fn generate_digit_mild() -> i32 {
        use rand::seq::IndexedRandom;
        [
            (1, 3.0),
            (2, 3.0),
            (3, 2.0),
            (4, 2.0),
            (5, 1.0),
            (6, 0.5),
            (7, 0.5),
            (8, 0.2),
            (9, 0.1),
        ]
        .choose_weighted(&mut rand::rng(), |(_, w)| *w)
        .unwrap()
        .0
    }
    fn generate_two_digits(&mut self) -> i32 {
        loop {
            let value = Self::generate_digit_mild() * 10 + Self::generate_digit_mild();
            if self.used_numbers.insert(value) || rand::random_bool(0.5) {
                return value;
            }
        }
    }
    fn generate_four_digits(&mut self) -> i32 {
        Self::generate_digit_mild() * 1000
            + Self::generate_digit_even() * 100
            + Self::generate_digit_even() * 10
            + Self::generate_digit_even()
    }
}

fn init_problem(mut commands: Commands) {
    commands.insert_resource(ProblemGenerator::default());
    commands.spawn((SessionOnly, Observer::new(handle_problem_result)));
}

fn quit_problem(mut commands: Commands) {
    commands.remove_resource::<ProblemGenerator>();
}

fn spawn_problem(
    mut commands: Commands,
    mut generator: ResMut<ProblemGenerator>,
    time: Res<Time>,
    fonts: Res<canum_ui::Fonts>,
    q_problem: Query<(), With<Problem>>,
) {
    if q_problem.iter().next().is_some() {
        return;
    }
    if generator.timer.tick(time.delta()).just_finished() {
        let (problem, text) = if rand::random_bool(0.5) {
            // Generate multiply problem.
            let first = generator.generate_two_digits();
            let second = generator.generate_two_digits();
            let answer = first * second;
            (Problem { answer }, format!("{} × {} = ", first, second))
        } else {
            // Generate addition problem.
            let first = generator.generate_four_digits();
            let second = generator.generate_four_digits();
            let answer = first + second;
            (Problem { answer }, format!("{} + {} = ", first, second))
        };
        let font = TextFont {
            font: fonts.game.clone(),
            font_size: 50.0,
            font_smoothing: bevy::text::FontSmoothing::None,
            ..default()
        };
        commands.spawn((
            problem,
            Transform::from_translation(vec3(
                0.0,
                CONFIG.display.half_virtual_size.1 - 30.0,
                26.37,
            )),
            Text2d::new(text),
            font.clone(),
            children![
                (ProblemInputBox, font),
                (
                    Transform::from_translation(vec3(0.0, -50.0, -0.1)),
                    ProblemTimer {
                        timer: Timer::new(generator.timeout, TimerMode::Once)
                    },
                    Sprite {
                        custom_size: Some(Vec2::new(
                            ProblemTimer::TIMER_LENGTH,
                            ProblemTimer::TIMER_WIDTH
                        )),
                        color: Color::srgb_u8(20, 230, 120),
                        ..default()
                    }
                )
            ],
        ));
        commands.spawn(Sound::new("Runway_Notify"));
    }
}

fn update_problem(
    input: Res<ButtonInput<KeyCode>>,
    mut q_input: Query<&mut TextSpan, With<ProblemInputBox>>,
    q_problem: Query<&Problem>,
    mut q_timer: Query<(&mut ProblemTimer, &mut Sprite)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let Ok(mut text_span) = q_input.single_mut() else {
        return;
    };
    let Ok(problem) = q_problem.single() else {
        return;
    };
    let Ok((mut timer, mut sprite)) = q_timer.single_mut() else {
        return;
    };
    if timer.timer.tick(time.delta()).just_finished() {
        commands.trigger(ProblemResult(false));
        return;
    }
    if let Some(ref mut size) = sprite.custom_size {
        size.x = timer.timer.remaining_secs() / timer.timer.duration().as_secs_f32()
            * ProblemTimer::TIMER_LENGTH;
    }
    for key in input.get_just_pressed() {
        let ch = match key {
            KeyCode::Backspace => {
                text_span.0.pop();
                continue;
            }
            KeyCode::Enter => {
                let success = text_span
                    .0
                    .trim()
                    .parse::<i32>()
                    .is_ok_and(|value| value == problem.answer);
                commands.trigger(ProblemResult(success));
                continue;
            }
            KeyCode::Digit0 | KeyCode::Numpad0 => '0',
            KeyCode::Digit1 | KeyCode::Numpad1 => '1',
            KeyCode::Digit2 | KeyCode::Numpad2 => '2',
            KeyCode::Digit3 | KeyCode::Numpad3 => '3',
            KeyCode::Digit4 | KeyCode::Numpad4 => '4',
            KeyCode::Digit5 | KeyCode::Numpad5 => '5',
            KeyCode::Digit6 | KeyCode::Numpad6 => '6',
            KeyCode::Digit7 | KeyCode::Numpad7 => '7',
            KeyCode::Digit8 | KeyCode::Numpad8 => '8',
            KeyCode::Digit9 | KeyCode::Numpad9 => '9',
            _ => continue,
        };
        text_span.0.push(ch);
    }
}

#[derive(Event, Debug, Default)]
struct ProblemResult(bool);

#[derive(Component, Default)]
struct LionGetHit;

fn handle_problem_result(
    event: On<ProblemResult>,
    mut commands: Commands,
    primary_player: Option<Res<player::PrimaryPlayer>>,
    q_problem: Query<Entity, With<Problem>>,
    mut q_lion: Query<(Entity, &Children, &mut Animation), With<RunwayLion>>,
    mut q_manager: Query<&mut enemy::behavior::BehaviorManager>,
) {
    let Some(primary_player) = primary_player else {
        return;
    };
    if event.0 {
        commands.spawn(Sound::new("Runway_Correct"));
        let Ok((lion, lion_children, mut animation)) = q_lion.single_mut() else {
            return;
        };
        for child in lion_children.iter() {
            if let Ok(mut manager) = q_manager.get_mut(child) {
                manager.disabled = true;
            }
        }
        animation.replace("Runway_Lion_Hit", false, None);
        let entity = commands
            .spawn((
                ChildOf(lion),
                Transform::from_translation(vec3(0.0, -20.0, 0.1)),
                LionGetHit,
                bevy::sprite::Anchor::BOTTOM_CENTER,
            ))
            .observe(lion_get_hit_over)
            .id();
        commands.entity(entity).insert(
            Animation::new("Runway_Lightning", vec2(60.0, 200.0)).with_inform(AnimationInform {
                entity,
                index: vec![0],
            }),
        );
    } else {
        commands.spawn(Sound::new("Runway_Error"));
        commands.trigger(health::Damage {
            entity: primary_player.0,
            order: 250,
            value: 100,
        });
    }
    let Ok(problem) = q_problem.single() else {
        return;
    };
    commands.entity(problem).despawn();
}

fn lion_get_hit_over(
    event: On<AnimationComplete>,
    mut commands: Commands,
    mut q_parent: Query<(Entity, &mut Animation), With<RunwayLion>>,
) {
    commands.entity(event.entity).despawn();
    let Ok((entity, mut animation)) = q_parent.single_mut() else {
        return;
    };
    animation.replace("Runway_Lion_Running", false, None);
    let back = CONFIG.display.half_virtual_size.1 * 0.201;
    commands.spawn((
        ChildOf(entity),
        enemy::movements::Displacement {
            curve: |value| value,
            displace: Vec2::new(0.0, back),
            duration: Duration::from_secs_f32(1.0),
            notify: None,
        },
    ));
}
