use crate::prelude::*;

pub(super) struct CoinPlugin;

impl Plugin for CoinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(canum_play::setup::PlayState::Shop), spawn_coin);
        app.add_observer(update_coin);
    }
}

fn spawn_coin(
    mut commands: Commands,
    q_top_right: Query<Entity, With<crate::TopRight>>,
    save: Res<Save>,
    fonts: Res<crate::Fonts>,
) {
    let Ok(top_right) = q_top_right.single() else {
        return;
    };
    commands.spawn((
        ChildOf(top_right),
        crate::lobby::coin::coin(&fonts, save.progress.coins),
    ));
}

fn update_coin(
    _event: On<canum_play::setup::shop::PurchaseItemSuccess>,
    mut q_coin: Query<&mut Text, With<crate::lobby::coin::CoinNumber>>,
    save: Res<Save>,
) {
    let Ok(mut text) = q_coin.single_mut() else {
        return;
    };
    text.0 = format!("{}", save.progress.coins);
}
