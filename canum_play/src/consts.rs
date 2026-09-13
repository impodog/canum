/// Damage and shield order constants
pub mod order {
    pub const ENEMY_PROJ: u8 = 50;
    pub const ENEMY_MINION: u8 = 110;
    pub const ENEMY_BOSS: u8 = 120;
    pub const PLAYER_PROJ: u8 = 50;
    pub const PLAYER_PROJ_WEAK: u8 = 40;
    pub const PLAYER_PROJ_STRONG: u8 = 60;
    pub const DASH_INVINC: u8 = 230;
    pub const HEALTH_INVINC: u8 = 220;
    pub const EXOSKELETON: u8 = 201;
}

/// The damage numbers are classified into ONE, TWO, THREE... Each represents how much damage they do to integer health bars.
/// And they are further specified into WEAK, MID, STRONG. This is reserved for other health-related factors.
pub mod damage {
    pub const ONE_WEAK: i32 = 50;
    pub const ONE_MID: i32 = 75;
    pub const ONE_STRONG: i32 = 100;
    pub const TWO_WEAK: i32 = 150;
    pub const TWO_MID: i32 = 175;
    pub const TWO_STRONG: i32 = 200;
    pub const THREE_WEAK: i32 = 250;
    pub const THREE_MID: i32 = 275;
    pub const THREE_STRONG: i32 = 300;

    /// Converts bare damage value into the number of damages that the player takes.
    pub fn damage_number(value: i32) -> i32 {
        if value <= 0 {
            0
        } else if value < TWO_WEAK {
            1
        } else if value < THREE_WEAK {
            2
        } else {
            3
        }
    }
}
