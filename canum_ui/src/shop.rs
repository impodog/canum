use crate::prelude::*;

mod coin;

pub(super) struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((coin::CoinPlugin,));
    }
}
