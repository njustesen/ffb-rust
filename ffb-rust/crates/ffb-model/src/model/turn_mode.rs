use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.TurnMode.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TurnMode {
    REGULAR, SETUP, KICKOFF,
    PERFECT_DEFENCE, SOLID_DEFENCE,
    QUICK_SNAP, HIGH_KICK, START_GAME, BLITZ,
    TOUCHBACK, INTERCEPTION, END_GAME,
    SWARMING, KICKOFF_RETURN, WIZARD,
    PASS_BLOCK, DUMP_OFF, NO_PLAYERS_TO_FIELD,
    BOMB_HOME, BOMB_AWAY, BOMB_HOME_BLITZ, BOMB_AWAY_BLITZ,
    ILLEGAL_SUBSTITUTION, SELECT_BLITZ_TARGET,
    SELECT_GAZE_TARGET, SAFE_PAIR_OF_HANDS,
    SELECT_BLOCK_KIND, BETWEEN_TURNS,
    TRICKSTER,
    RAIDING_PARTY, HIT_AND_RUN, THEN_I_STARTED_BLASTIN,
}

impl TurnMode {
    pub fn get_name(self) -> &'static str {
        match self {
            TurnMode::REGULAR => "regular",
            TurnMode::SETUP => "setup",
            TurnMode::KICKOFF => "kickoff",
            TurnMode::PERFECT_DEFENCE => "perfectDefence",
            TurnMode::SOLID_DEFENCE => "solidDefence",
            TurnMode::QUICK_SNAP => "quickSnap",
            TurnMode::HIGH_KICK => "highKick",
            TurnMode::START_GAME => "startGame",
            TurnMode::BLITZ => "blitz",
            TurnMode::TOUCHBACK => "touchback",
            TurnMode::INTERCEPTION => "interception",
            TurnMode::END_GAME => "endGame",
            TurnMode::SWARMING => "swarming",
            TurnMode::KICKOFF_RETURN => "kickoffReturn",
            TurnMode::WIZARD => "wizard",
            TurnMode::PASS_BLOCK => "passBlock",
            TurnMode::DUMP_OFF => "dumpOff",
            TurnMode::NO_PLAYERS_TO_FIELD => "noPlayersToField",
            TurnMode::BOMB_HOME => "bombHome",
            TurnMode::BOMB_AWAY => "bombAway",
            TurnMode::BOMB_HOME_BLITZ => "bombHomeBlitz",
            TurnMode::BOMB_AWAY_BLITZ => "bombAwayBlitz",
            TurnMode::ILLEGAL_SUBSTITUTION => "illegalSubstitution",
            TurnMode::SELECT_BLITZ_TARGET => "selectBlitzTarget",
            TurnMode::SELECT_GAZE_TARGET => "selectGazeTarget",
            TurnMode::SAFE_PAIR_OF_HANDS => "safePairOfHands",
            TurnMode::SELECT_BLOCK_KIND => "selectBlockKind",
            TurnMode::BETWEEN_TURNS => "betweenTurns",
            TurnMode::TRICKSTER => "trickster",
            TurnMode::RAIDING_PARTY => "raidingParty",
            TurnMode::HIT_AND_RUN => "hitAndRun",
            TurnMode::THEN_I_STARTED_BLASTIN => "thenIStartedBlastin",
        }
    }

    pub fn check_negatraits(self) -> bool {
        !matches!(self, TurnMode::KICKOFF_RETURN | TurnMode::PASS_BLOCK) && !self.is_bomb_turn()
    }

    pub fn is_bomb_turn(self) -> bool {
        matches!(self, TurnMode::BOMB_HOME | TurnMode::BOMB_HOME_BLITZ | TurnMode::BOMB_AWAY | TurnMode::BOMB_AWAY_BLITZ)
    }

    pub fn allow_end_player_action(self) -> bool {
        !self.is_bomb_turn() && self != TurnMode::DUMP_OFF
    }

    pub fn is_check_for_active_players(self) -> bool {
        matches!(self, TurnMode::REGULAR | TurnMode::BLITZ | TurnMode::BETWEEN_TURNS)
    }

    pub fn force_dice_decoration_update(self) -> bool {
        self == TurnMode::TRICKSTER
    }

    pub fn for_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.get_name().eq_ignore_ascii_case(name))
    }

    pub fn is_basic_mode(self) -> bool {
        self == TurnMode::REGULAR || self == TurnMode::BLITZ
    }

    fn all() -> &'static [TurnMode] {
        &[
            Self::REGULAR, Self::SETUP, Self::KICKOFF, Self::PERFECT_DEFENCE, Self::SOLID_DEFENCE,
            Self::QUICK_SNAP, Self::HIGH_KICK, Self::START_GAME, Self::BLITZ, Self::TOUCHBACK,
            Self::INTERCEPTION, Self::END_GAME, Self::SWARMING, Self::KICKOFF_RETURN, Self::WIZARD,
            Self::PASS_BLOCK, Self::DUMP_OFF, Self::NO_PLAYERS_TO_FIELD, Self::BOMB_HOME, Self::BOMB_AWAY,
            Self::BOMB_HOME_BLITZ, Self::BOMB_AWAY_BLITZ, Self::ILLEGAL_SUBSTITUTION,
            Self::SELECT_BLITZ_TARGET, Self::SELECT_GAZE_TARGET, Self::SAFE_PAIR_OF_HANDS,
            Self::SELECT_BLOCK_KIND, Self::BETWEEN_TURNS, Self::TRICKSTER, Self::RAIDING_PARTY,
            Self::HIT_AND_RUN, Self::THEN_I_STARTED_BLASTIN,
        ]
    }
}
