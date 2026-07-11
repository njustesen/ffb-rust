/// 1:1 translation of com.fumbbl.ffb.client.ActionKey (Java enum).
///
/// Each variant identifies a client keyboard action; the string is the
/// associated `IClientProperty` key name used to look up the bound key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ActionKey {
    PLAYER_MOVE_NORTH,
    PLAYER_MOVE_NORTHEAST,
    PLAYER_MOVE_EAST,
    PLAYER_MOVE_SOUTHEAST,
    PLAYER_MOVE_SOUTH,
    PLAYER_MOVE_SOUTHWEST,
    PLAYER_MOVE_WEST,
    PLAYER_MOVE_NORTHWEST,

    PLAYER_SELECT,
    PLAYER_CYCLE_RIGHT,
    PLAYER_CYCLE_LEFT,

    PLAYER_ACTION_BLOCK,
    PLAYER_ACTION_BLITZ,
    PLAYER_ACTION_FOUL,
    PLAYER_ACTION_MOVE,
    PLAYER_ACTION_STAND_UP,
    PLAYER_ACTION_HAND_OVER,
    PLAYER_ACTION_PASS,
    PLAYER_ACTION_JUMP,
    PLAYER_ACTION_END_MOVE,
    PLAYER_ACTION_STAB,
    PLAYER_ACTION_CHAINSAW,
    PLAYER_ACTION_GAZE,
    PLAYER_ACTION_GAZE_ZOAT,
    PLAYER_ACTION_FUMBLEROOSKIE,
    PLAYER_ACTION_PROJECTILE_VOMIT,
    PLAYER_ACTION_RANGE_GRID,
    PLAYER_ACTION_HAIL_MARY_PASS,
    PLAYER_ACTION_MULTIPLE_BLOCK,
    PLAYER_ACTION_FRENZIED_RUSH,
    PLAYER_ACTION_SHOT_TO_NOTHING,
    PLAYER_ACTION_SHOT_TO_NOTHING_BOMB,
    PLAYER_ACTION_TREACHEROUS,
    PLAYER_ACTION_WISDOM,
    PLAYER_ACTION_BEER_BARREL_BASH,
    PLAYER_ACTION_RAIDING_PARTY,
    PLAYER_ACTION_LOOK_INTO_MY_EYES,
    PLAYER_ACTION_BALEFUL_HEX,
    PLAYER_ACTION_HIT_AND_RUN,
    PLAYER_ACTION_KICK_EM_BLOCK,
    PLAYER_ACTION_KICK_EM_BLITZ,
    PLAYER_ACTION_GORED,
    PLAYER_ACTION_BLACK_INK,
    PLAYER_ACTION_CATCH_OF_THE_DAY,
    PLAYER_ACTION_BOUNDING_LEAP,
    PLAYER_ACTION_BREATHE_FIRE,
    PLAYER_ACTION_THEN_I_STARTED_BLASTIN,
    PLAYER_ACTION_THE_FLASHING_BLADE,
    PLAYER_ACTION_VICIOUS_VINES,
    PLAYER_ACTION_FURIOUS_OUTBURST,
    PLAYER_ACTION_MORE_ACTIONS,
    PLAYER_ACTION_SECURE_THE_BALL,
    PLAYER_ACTION_SLASHING_NAILS,
    PLAYER_ACTION_AUTO_GAZE_ZOAT,
    PLAYER_ACTION_FORGO,
    PLAYER_ACTION_INCORPOREAL,
    PLAYER_ACTION_CHOMP,
    PLAYER_ACTION_PUNT,

    TOOLBAR_TURN_END,
    TOOLBAR_ILLEGAL_PROCEDURE,

    RESIZE_LARGER,
    RESIZE_SMALLER,
    RESIZE_RESET,
    RESIZE_SMALLER2,
    RESIZE_LARGER2,
    RESIZE_RESET2,
    RESIZE_SMALLER3,
    RESIZE_LARGER3,
    RESIZE_RESET3,
    RESIZE_SMALLER4,

    MENU_SETUP_LOAD,
    MENU_SETUP_SAVE,
    MENU_REPLAY,
}

impl ActionKey {
    /// Java: fPropertyName (the `IClientProperty.KEY_*` constant value).
    pub fn property_name(self) -> &'static str {
        use ActionKey::*;
        match self {
            PLAYER_MOVE_NORTH => "key.player.move.north",
            PLAYER_MOVE_NORTHEAST => "key.player.move.northeast",
            PLAYER_MOVE_EAST => "key.player.move.east",
            PLAYER_MOVE_SOUTHEAST => "key.player.move.southeast",
            PLAYER_MOVE_SOUTH => "key.player.move.south",
            PLAYER_MOVE_SOUTHWEST => "key.player.move.southwest",
            PLAYER_MOVE_WEST => "key.player.move.west",
            PLAYER_MOVE_NORTHWEST => "key.player.move.northwest",

            PLAYER_SELECT => "key.player.select",
            PLAYER_CYCLE_RIGHT => "key.player.cycle.right",
            PLAYER_CYCLE_LEFT => "key.player.cycle.left",

            PLAYER_ACTION_BLOCK => "key.player.action.block",
            PLAYER_ACTION_BLITZ => "key.player.action.blitz",
            PLAYER_ACTION_FOUL => "key.player.action.foul",
            PLAYER_ACTION_MOVE => "key.player.action.move",
            PLAYER_ACTION_STAND_UP => "key.player.action.standup",
            PLAYER_ACTION_HAND_OVER => "key.player.action.handover",
            PLAYER_ACTION_PASS => "key.player.action.pass",
            PLAYER_ACTION_JUMP => "key.player.action.jump",
            PLAYER_ACTION_END_MOVE => "key.player.action.endMove",
            PLAYER_ACTION_STAB => "key.player.action.stab",
            PLAYER_ACTION_CHAINSAW => "key.player.action.chainsaw",
            PLAYER_ACTION_GAZE => "key.player.action.gaze",
            PLAYER_ACTION_GAZE_ZOAT => "key.player.action.gazeZoat",
            PLAYER_ACTION_FUMBLEROOSKIE => "key.player.action.fumblerooskie",
            PLAYER_ACTION_PROJECTILE_VOMIT => "key.player.action.projectileVomit",
            PLAYER_ACTION_RANGE_GRID => "key.player.action.rangeGrid",
            PLAYER_ACTION_HAIL_MARY_PASS => "key.player.action.hailMaryPass",
            PLAYER_ACTION_MULTIPLE_BLOCK => "key.player.action.multipleBlock",
            PLAYER_ACTION_FRENZIED_RUSH => "key.player.action.frenziedRush",
            PLAYER_ACTION_SHOT_TO_NOTHING => "key.player.action.shotToNothing",
            PLAYER_ACTION_SHOT_TO_NOTHING_BOMB => "key.player.action.shotToNothingBomb",
            PLAYER_ACTION_TREACHEROUS => "key.player.action.treacherous",
            PLAYER_ACTION_WISDOM => "key.player.action.wisdom",
            PLAYER_ACTION_BEER_BARREL_BASH => "key.player.action.beerBarrelBash",
            PLAYER_ACTION_RAIDING_PARTY => "key.player.action.raidingParty",
            PLAYER_ACTION_LOOK_INTO_MY_EYES => "key.player.action.lookIntoMyEyes",
            PLAYER_ACTION_BALEFUL_HEX => "key.player.action.balefulHex",
            PLAYER_ACTION_HIT_AND_RUN => "key.player.action.hitAndRun",
            PLAYER_ACTION_KICK_EM_BLOCK => "key.player.action.kickEmBlock",
            PLAYER_ACTION_KICK_EM_BLITZ => "key.player.action.kickEmBlitz",
            PLAYER_ACTION_GORED => "key.player.action.goredByTheBull",
            PLAYER_ACTION_BLACK_INK => "key.player.action.blackInk",
            PLAYER_ACTION_CATCH_OF_THE_DAY => "key.player.action.catchOfTheDay",
            PLAYER_ACTION_BOUNDING_LEAP => "key.player.action.boundingLeap",
            PLAYER_ACTION_BREATHE_FIRE => "key.player.action.breatheFire",
            PLAYER_ACTION_THEN_I_STARTED_BLASTIN => "key.player.action.thenIStartedBlastin",
            PLAYER_ACTION_THE_FLASHING_BLADE => "key.player.action.theFlashingBlade",
            PLAYER_ACTION_VICIOUS_VINES => "key.player.action.viciousVines",
            PLAYER_ACTION_FURIOUS_OUTBURST => "key.player.action.furiousOutburst",
            PLAYER_ACTION_MORE_ACTIONS => "key.player.action.moreActions",
            PLAYER_ACTION_SECURE_THE_BALL => "key.player.action.secureTheBall",
            PLAYER_ACTION_SLASHING_NAILS => "key.player.action.slashingNails",
            PLAYER_ACTION_AUTO_GAZE_ZOAT => "key.player.action.autoGazeZoat",
            PLAYER_ACTION_FORGO => "key.player.action.forgo",
            PLAYER_ACTION_INCORPOREAL => "key.player.action.incorporeal",
            PLAYER_ACTION_CHOMP => "key.player.action.chomp",
            PLAYER_ACTION_PUNT => "key.player.action.punt",

            TOOLBAR_TURN_END => "key.toolbar.turn.end",
            TOOLBAR_ILLEGAL_PROCEDURE => "key.toolbar.illegal.procedure",

            RESIZE_LARGER => "key.resize.larger",
            RESIZE_SMALLER => "key.resize.smaller",
            RESIZE_RESET => "key.resize.reset",
            RESIZE_SMALLER2 => "key.resize.smaller2",
            RESIZE_LARGER2 => "key.resize.larger2",
            RESIZE_RESET2 => "key.resize.reset2",
            RESIZE_SMALLER3 => "key.resize.smaller3",
            RESIZE_LARGER3 => "key.resize.larger3",
            RESIZE_RESET3 => "key.resize.reset3",
            RESIZE_SMALLER4 => "key.resize.smaller4",

            MENU_SETUP_LOAD => "key.menu.setup.load",
            MENU_SETUP_SAVE => "key.menu.setup.save",
            MENU_REPLAY => "key.menu.replay",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_directions_have_distinct_property_names() {
        let moves = [
            ActionKey::PLAYER_MOVE_NORTH,
            ActionKey::PLAYER_MOVE_NORTHEAST,
            ActionKey::PLAYER_MOVE_EAST,
            ActionKey::PLAYER_MOVE_SOUTHEAST,
            ActionKey::PLAYER_MOVE_SOUTH,
            ActionKey::PLAYER_MOVE_SOUTHWEST,
            ActionKey::PLAYER_MOVE_WEST,
            ActionKey::PLAYER_MOVE_NORTHWEST,
        ];
        let names: Vec<&str> = moves.iter().map(|k| k.property_name()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), names.len());
    }

    #[test]
    fn player_move_north_property_name() {
        assert_eq!(ActionKey::PLAYER_MOVE_NORTH.property_name(), "key.player.move.north");
    }

    #[test]
    fn menu_replay_property_name() {
        assert_eq!(ActionKey::MENU_REPLAY.property_name(), "key.menu.replay");
    }

    #[test]
    fn resize_reset_property_name() {
        assert_eq!(ActionKey::RESIZE_RESET.property_name(), "key.resize.reset");
    }
}
