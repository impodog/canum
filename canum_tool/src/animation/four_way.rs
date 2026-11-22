use crate::prelude::*;

pub(super) struct FourWayPlugin;

impl Plugin for FourWayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (select_four_way, sprite_sheet_four_way)
                .chain()
                .in_set(super::AnimationSystems::Suffix),
        );
    }
}

/// The four directions supported by [`FourWay`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FourWayDirection {
    #[default]
    Down,
    Up,
    Left,
    Right,
}
impl FourWayDirection {
    /// The suffix appended to the base sprite sheet name.
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Right => "_FWRight",
            Self::Up => "_FWUp",
            Self::Left => "_FWLeft",
            Self::Down => "_FWDown",
        }
    }

    /// Picks the cardinal direction closest to `velocity` using the dominant axis.
    ///
    /// Returns `None` if both velocity components are at or below `threshold`.
    pub fn from_velocity(velocity: Vec2, threshold: f32) -> Option<Self> {
        if velocity.x.abs().max(velocity.y.abs()) <= threshold {
            return None;
        }
        if velocity.x.abs() >= velocity.y.abs() {
            Some(if velocity.x > 0.0 {
                Self::Right
            } else {
                Self::Left
            })
        } else if velocity.y > 0.0 {
            Some(Self::Up)
        } else {
            Some(Self::Down)
        }
    }
}

/// Selects one of four directional sprite sheets based on [`FourWay::direction`].
///
/// The applied name is the base `SpriteSheet` name with `_FWRight`, `_FWUp`,
/// `_FWLeft`, or `_FWDown` appended.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FourWay {
    pub direction: FourWayDirection,
}
impl FourWay {
    pub const fn new(direction: FourWayDirection) -> Self {
        Self { direction }
    }
}

/// Automatically updates [`FourWay`] from the entity's `ForcedVelocity`, so
/// that the sprite faces where it is going.
///
/// Velocities at or below [`FourWaySelector::threshold`] keep the previous
/// direction, preventing the animation from flickering when the entity stops.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(FourWay)]
pub struct FourWaySelector {
    pub threshold: f32,
}
impl FourWaySelector {
    pub const fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}
impl Default for FourWaySelector {
    fn default() -> Self {
        Self { threshold: 10.0 }
    }
}

/// The suffixes appended by [`FourWay`].
pub(super) const FOUR_WAY_SUFFIXES: [&str; 4] = [
    FourWayDirection::Right.suffix(),
    FourWayDirection::Up.suffix(),
    FourWayDirection::Left.suffix(),
    FourWayDirection::Down.suffix(),
];

/// Checks whether `name` is `base` with one of the four way suffixes appended.
pub(super) fn is_four_way_name(name: &str, base: &str) -> bool {
    name.strip_prefix(base)
        .is_some_and(|suffix| FOUR_WAY_SUFFIXES.contains(&suffix))
}

fn base_sprite_name(name: &str) -> &str {
    FOUR_WAY_SUFFIXES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(name)
}

fn select_four_way(
    mut q_four_way: Query<(&mut FourWay, &FourWaySelector, &movements::ForcedVelocity)>,
) {
    q_four_way
        .par_iter_mut()
        .for_each(|(mut four_way, selector, forced_velocity)| {
            let Some(direction) =
                FourWayDirection::from_velocity(**forced_velocity, selector.threshold)
            else {
                return;
            };
            if four_way.direction != direction {
                four_way.direction = direction;
            }
        });
}

fn sprite_sheet_four_way(mut q_sprite_sheet: Query<(&mut SpriteSheet, &FourWay)>) {
    q_sprite_sheet
        .par_iter_mut()
        .for_each(|(mut sprite_sheet, four_way)| {
            let base_name = base_sprite_name(&sprite_sheet.name);
            let name = format!("{base_name}{}", four_way.direction.suffix());
            if sprite_sheet.name != name {
                sprite_sheet.name = name;
            }
        });
}
