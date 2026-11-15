use crate::prelude::*;

pub(super) struct FourWayPlugin;

impl Plugin for FourWayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            sprite_sheet_four_way.in_set(super::AnimationSystems::Suffix),
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
