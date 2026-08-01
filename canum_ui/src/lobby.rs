pub mod boss;
pub mod coin;
pub mod lock;

use crate::prelude::*;

pub(super) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((coin::CoinPlugin, boss::BossPlugin, lock::LockPlugin));
    }
}
