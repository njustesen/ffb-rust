use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::model::field_model::FieldModel;
use crate::model::team::{Team, TurnData};
use crate::types::{
    BlockResult, FieldCoordinate, Half, InducementState, PlayerId, PlayerAction, TeamId, TurnMode, Weather,
};

// ── Acting player ─────────────────────────────────────────────────────────────

/// Pending state for movement team-reroll dialog.
/// Stored in ActingPlayer when a dodge/GFI fails and a team reroll should be offered.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PendingMoveState {
    /// Square the player needs to move to if the reroll succeeds (dodge) or is already at (GFI).
    pub current_dest: FieldCoordinate,
    /// Remaining path after current_dest to execute if reroll succeeds.
    pub remaining_after: Vec<FieldCoordinate>,
    /// Minimum die roll needed to succeed.
    pub min_roll: u8,
    /// True if this is a GFI roll (player already moved to current_dest).
    pub is_gfi: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ActingPlayer {
    pub player_id: PlayerId,
    pub team: TeamId,
    pub movement_used: u8,
    pub has_blitzed: bool,
    pub has_passed: bool,
    pub has_handed_off: bool,
    pub has_fouled: bool,
    pub current_action: Option<PlayerAction>,
    /// Set when WildAnimal roll failed: player may only Block or Blitz.
    #[serde(default)]
    pub wild_animal_restricted: bool,
    /// Set when the player has Frenzy and just pushed/pow-pushed the defender:
    /// the attacker MUST follow up and make a second block against the same defender.
    #[serde(default)]
    pub frenzy_second_block_required: bool,
    /// Set when the Pro skill re-roll has been used this activation.
    #[serde(default)]
    pub pro_reroll_used: bool,
    /// Set when the player has made their block (used for Blitz to prevent double block).
    #[serde(default)]
    pub has_blocked: bool,
    /// Pending move reroll state: set when a dodge/GFI failed and team reroll was offered.
    #[serde(default)]
    pub pending_move: Option<PendingMoveState>,
    /// Pending pass target: set when a pass failed and team reroll was offered.
    #[serde(default)]
    pub pending_pass_target: Option<FieldCoordinate>,
    /// True if the Pass skill reroll has already been used this activation.
    #[serde(default)]
    pub pass_skill_reroll_used: bool,
    /// Pending pickup square: set when a ball pickup failed and team reroll was offered.
    #[serde(default)]
    pub pending_pickup_at: Option<FieldCoordinate>,
    /// Pending catch square: set when a catch failed and team reroll was offered.
    #[serde(default)]
    pub pending_catch_at: Option<FieldCoordinate>,
}

impl ActingPlayer {
    pub fn new(player_id: PlayerId, team: TeamId) -> Self {
        Self {
            player_id,
            team,
            movement_used: 0,
            has_blitzed: false,
            has_passed: false,
            has_handed_off: false,
            has_fouled: false,
            has_blocked: false,
            current_action: None,
            wild_animal_restricted: false,
            frenzy_second_block_required: false,
            pro_reroll_used: false,
            pending_move: None,
            pending_pass_target: None,
            pass_skill_reroll_used: false,
            pending_pickup_at: None,
            pending_catch_at: None,
        }
    }
}


// ── Dialog state — what decision the engine is waiting on ─────────────────────

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum DialogState {
    #[default]
    None,
    SelectPlayer {
        candidates: Vec<PlayerId>,
    },
    SelectMoveTarget {
        targets: Vec<(FieldCoordinate, f64)>,
    },
    SelectBlockTarget {
        targets: Vec<PlayerId>,
    },
    SelectBlockDice {
        dice: Vec<BlockResult>,
        /// True if defender (not attacker) is choosing
        defender_chooses: bool,
    },
    SelectReroll {
        action_name: String,
        reroll_available: bool,
        skill_reroll_available: bool,
    },
    /// Offer a reroll on block dice before the player picks which die to use.
    /// Stores the original dice in case the player declines.
    SelectBlockReroll {
        dice: Vec<crate::types::BlockResult>,
        defender_chooses: bool,
        reroll_available: bool,
    },
    SelectPush {
        options: Vec<FieldCoordinate>,
    },
    SelectInjury,
    SelectApothecary {
        original: crate::types::CasualtyType,
        reroll: crate::types::CasualtyType,
    },
    SelectKickTarget,
    SelectKickoffReturn,
    SelectHighKickPlayer,
    /// Offer the attacker a voluntary follow-up after a push.
    /// `square` is the square the defender just vacated (guaranteed empty and adjacent to attacker).
    SelectFollowup {
        square: FieldCoordinate,
    },
}

// ── Game result ───────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct GameResult {
    pub score_home: u8,
    pub score_away: u8,
    pub finished: bool,
    pub winner: Option<TeamId>,
    /// Total turns played
    pub turns_played: u16,
}

impl GameResult {
    pub fn score_delta(&self, team: TeamId) -> i8 {
        match team {
            TeamId::Home => self.score_home as i8 - self.score_away as i8,
            TeamId::Away => self.score_away as i8 - self.score_home as i8,
        }
    }
}

// ── Game options ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct GameOptions {
    pub ruleset: Ruleset,
    pub max_turns_per_half: u8,
    pub overtime: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Ruleset {
    /// Base marker: rules common to all editions. Never used as an actual game ruleset.
    Common,
    Bb2025,
    Bb2020,
    Bb2016,
}

impl Ruleset {
    /// Returns `true` if `self` is `other` or transitively extends `other`.
    ///
    /// Hierarchy: `Bb2016`, `Bb2020`, `Bb2025` all extend `Common`.
    /// Sibling rulesets do not extend each other.
    pub fn is_or_extends(self, other: Ruleset) -> bool {
        if self == other {
            return true;
        }
        match self {
            Ruleset::Common => false,
            Ruleset::Bb2016 | Ruleset::Bb2020 | Ruleset::Bb2025 => other == Ruleset::Common,
        }
    }
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            ruleset: Ruleset::Bb2025,
            max_turns_per_half: 8,
            overtime: false,
        }
    }
}

// ── Game state ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub field: FieldModel,
    pub home: Team,
    pub away: Team,
    pub turn_data_home: TurnData,
    pub turn_data_away: TurnData,
    pub half: Half,
    pub turn_mode: TurnMode,
    /// True when the home team is the active (acting) team this turn.
    pub home_is_active: bool,
    pub acting_player: Option<ActingPlayer>,
    pub dialog: DialogState,
    pub result: GameResult,
    pub options: GameOptions,
    /// Home team is on offense (attacking toward increasing x)?
    pub home_is_offense: bool,
    pub inducement_state_home: InducementState,
    pub inducement_state_away: InducementState,
    /// Players that have already activated (taken an action) this turn.
    #[serde(default)]
    pub players_activated_this_turn: HashSet<PlayerId>,
    /// Players that have been hypnotized this turn (by HypnoticGaze).
    /// Hypnotized players do not exert tackle zones for the remainder of the turn.
    /// Consumers of tackle zone data should check this set and subtract 1 zone per
    /// hypnotized player when computing effective tackle zones.
    #[serde(default)]
    pub hypnotized_this_turn: HashSet<PlayerId>,
}

impl GameState {
    pub fn new(home: Team, away: Team) -> Self {
        Self {
            field: FieldModel::new(),
            home,
            away,
            turn_data_home: TurnData::default(),
            turn_data_away: TurnData::default(),
            half: Half::First,
            turn_mode: TurnMode::StartGame,
            home_is_active: true,
            acting_player: None,
            dialog: DialogState::None,
            result: GameResult::default(),
            options: GameOptions::default(),
            home_is_offense: true,
            inducement_state_home: InducementState::default(),
            inducement_state_away: InducementState::default(),
            players_activated_this_turn: HashSet::new(),
            hypnotized_this_turn: HashSet::new(),
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────

    pub fn active_team(&self) -> &Team {
        if self.home_is_active { &self.home } else { &self.away }
    }

    pub fn active_team_mut(&mut self) -> &mut Team {
        if self.home_is_active { &mut self.home } else { &mut self.away }
    }

    pub fn inactive_team(&self) -> &Team {
        if self.home_is_active { &self.away } else { &self.home }
    }

    pub fn inactive_team_mut(&mut self) -> &mut Team {
        if self.home_is_active { &mut self.away } else { &mut self.home }
    }

    pub fn team(&self, id: TeamId) -> &Team {
        match id { TeamId::Home => &self.home, TeamId::Away => &self.away }
    }

    pub fn team_mut(&mut self, id: TeamId) -> &mut Team {
        match id { TeamId::Home => &mut self.home, TeamId::Away => &mut self.away }
    }

    pub fn active_team_id(&self) -> TeamId {
        if self.home_is_active { TeamId::Home } else { TeamId::Away }
    }

    pub fn active_turn_data(&self) -> &TurnData {
        if self.home_is_active { &self.turn_data_home } else { &self.turn_data_away }
    }

    pub fn active_turn_data_mut(&mut self) -> &mut TurnData {
        if self.home_is_active { &mut self.turn_data_home } else { &mut self.turn_data_away }
    }

    /// Effective tackle zones on `coord` from `team`, excluding hypnotized players.
    /// Hypnotized players (from HypnoticGaze) do not exert tackle zones this turn.
    pub fn effective_tackle_zones_on(&self, coord: crate::types::FieldCoordinate, team: crate::types::TeamId) -> u8 {
        let base = self.field.tackle_zones_on(coord, team);
        if self.hypnotized_this_turn.is_empty() {
            return base;
        }
        // Count hypnotized players adjacent to `coord` that belong to `team`
        let hypnotized_adjacent = coord.neighbors()
            .filter(|&n| {
                if let Some(pid) = self.field.player_at(n) {
                    self.field.player_team(pid) == Some(team)
                        && self.hypnotized_this_turn.contains(pid)
                        && self.field.player_state(pid).map(|s| s.is_active()).unwrap_or(false)
                } else {
                    false
                }
            })
            .count() as u8;
        base.saturating_sub(hypnotized_adjacent)
    }

    pub fn weather(&self) -> Weather {
        self.field.weather
    }

    pub fn is_finished(&self) -> bool {
        self.result.finished
    }

    // ── Serialization helpers ─────────────────────────────────────────────

    /// Fast binary clone via bincode. Used by MCTS for state snapshots.
    /// Uses Rust's `Clone` derive (which deep-clones all fields including the
    /// FieldModel placement box and HashMaps). The Team index HashMap is included
    /// in the derive'd Clone since it's not `#[serde(skip)]` — if it were skipped
    /// for network serialization, we would need rebuild_index() here.
    pub fn fast_clone(&self) -> Self {
        self.clone()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> GameState {
        let home = Team::new("home".into(), "Reavers".into(), "Human".into(), 3, true);
        let away = Team::new("away".into(), "Raiders".into(), "Orc".into(), 3, false);
        GameState::new(home, away)
    }

    #[test]
    fn initial_state() {
        let s = make_state();
        assert_eq!(s.half, Half::First);
        assert_eq!(s.turn_mode, TurnMode::StartGame);
        assert!(s.home_is_active);
        assert!(s.acting_player.is_none());
        assert!(!s.result.finished);
    }

    #[test]
    fn active_team_returns_home() {
        let s = make_state();
        assert_eq!(s.active_team().name, "Reavers");
    }

    #[test]
    fn active_team_returns_away_when_toggled() {
        let mut s = make_state();
        s.home_is_active = false;
        assert_eq!(s.active_team().name, "Raiders");
    }

    #[test]
    fn fast_clone_preserves_state() {
        let s = make_state();
        let c = s.fast_clone();
        assert_eq!(c.half, s.half);
        assert_eq!(c.home.name, s.home.name);
        assert_eq!(c.away.name, s.away.name);
        assert_eq!(c.result.score_home, s.result.score_home);
    }

    #[test]
    fn fast_clone_is_independent() {
        let s = make_state();
        let mut c = s.fast_clone();
        c.result.score_home = 3;
        assert_eq!(s.result.score_home, 0, "original should be unmodified");
    }

    // ── Ruleset hierarchy (ported from RulesTest.java) ────────────────────────

    #[test]
    fn bb2016_is_or_extends_common() {
        assert!(Ruleset::Bb2016.is_or_extends(Ruleset::Common));
    }

    #[test]
    fn bb2020_is_or_extends_common() {
        assert!(Ruleset::Bb2020.is_or_extends(Ruleset::Common));
    }

    #[test]
    fn bb2025_is_or_extends_common() {
        assert!(Ruleset::Bb2025.is_or_extends(Ruleset::Common));
    }

    #[test]
    fn every_ruleset_is_or_extends_itself() {
        for rule in [Ruleset::Common, Ruleset::Bb2016, Ruleset::Bb2020, Ruleset::Bb2025] {
            assert!(rule.is_or_extends(rule), "{:?} should is_or_extend itself", rule);
        }
    }

    #[test]
    fn bb2020_does_not_extend_bb2016() {
        assert!(!Ruleset::Bb2020.is_or_extends(Ruleset::Bb2016));
    }

    #[test]
    fn common_does_not_extend_bb2020() {
        assert!(!Ruleset::Common.is_or_extends(Ruleset::Bb2020));
    }
}
