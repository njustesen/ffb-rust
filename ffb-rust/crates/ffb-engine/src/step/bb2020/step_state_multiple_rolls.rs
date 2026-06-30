use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.StepStateMultipleRolls` (BB2020).
///
/// This class is a **data struct** (implements IJsonSerializable and SingleReRollUseState)
/// that holds shared mutable roll-sequence state. In Java it is embedded into other BB2020
/// steps (e.g. StepBlockRollMultiple, StepDauntlessMultiple) by composition.
///
/// In Rust it also acts as a no-op Step (id=StateMultipleRolls) for the driver — the real
/// logic is carried by the enclosing step. The fields exactly mirror the Java fields.
pub struct StepStateMultipleRolls {
    /// Java: goToLabelOnFailure
    pub goto_label_on_failure: String,
    /// Java: reRollAvailableAgainst
    pub re_roll_available_against: Vec<String>,
    /// Java: blockTargets
    pub block_targets: Vec<String>,
    /// Java: firstRun
    pub first_run: bool,
    /// Java: teamReRollAvailable
    pub team_re_roll_available: bool,
    /// Java: proReRollAvailable
    pub pro_re_roll_available: bool,
    /// Java: consummateAvailable
    pub consummate_available: bool,
    /// Java: reRollSource (stored as name string; full enum porting deferred)
    pub re_roll_source: Option<String>,
    /// Java: singleUseReRollSource
    pub single_use_re_roll_source: Option<String>,
    /// Java: reRollTarget
    pub re_roll_target: Option<String>,
    /// Java: minimumRolls (player-id → minimum required roll)
    pub minimum_rolls: std::collections::HashMap<String, i32>,
    /// Java: initialCount
    pub initial_count: i32,
    /// Java: playerIdForSingleUseReRoll (implements SingleReRollUseState.getId)
    pub player_id_for_single_use_re_roll: Option<String>,
}

impl StepStateMultipleRolls {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            re_roll_available_against: Vec::new(),
            block_targets: Vec::new(),
            first_run: true,
            team_re_roll_available: false,
            pro_re_roll_available: false,
            consummate_available: false,
            re_roll_source: None,
            single_use_re_roll_source: None,
            re_roll_target: None,
            minimum_rolls: std::collections::HashMap::new(),
            initial_count: 0,
            player_id_for_single_use_re_roll: None,
        }
    }

    // ── SingleReRollUseState interface (Java) ────────────────────────────────

    /// Java: SingleReRollUseState.getId()
    pub fn get_id(&self) -> Option<&str> {
        self.player_id_for_single_use_re_roll.as_deref()
    }

    /// Java: SingleReRollUseState.setId(String)
    pub fn set_id(&mut self, player_id: String) {
        self.player_id_for_single_use_re_roll = Some(player_id);
    }

    /// Java: SingleReRollUseState.setReRollSource(ReRollSource)
    pub fn set_re_roll_source(&mut self, source: Option<String>) {
        self.re_roll_source = source;
    }
}

impl Default for StepStateMultipleRolls {
    fn default() -> Self { Self::new() }
}

impl Step for StepStateMultipleRolls {
    fn id(&self) -> StepId { StepId::StateMultipleRolls }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn default_fields() {
        let s = StepStateMultipleRolls::new();
        assert!(s.first_run, "firstRun defaults to true (Java)");
        assert!(!s.team_re_roll_available);
        assert!(!s.pro_re_roll_available);
        assert!(!s.consummate_available);
        assert_eq!(s.initial_count, 0);
        assert!(s.block_targets.is_empty());
        assert!(s.minimum_rolls.is_empty());
    }

    #[test]
    fn single_use_re_roll_state_get_set() {
        let mut s = StepStateMultipleRolls::new();
        assert!(s.get_id().is_none());
        s.set_id("player42".into());
        assert_eq!(s.get_id(), Some("player42"));
    }

    #[test]
    fn set_re_roll_source() {
        let mut s = StepStateMultipleRolls::new();
        s.set_re_roll_source(Some("PRO".into()));
        assert_eq!(s.re_roll_source.as_deref(), Some("PRO"));
        s.set_re_roll_source(None);
        assert!(s.re_roll_source.is_none());
    }

    #[test]
    fn step_start_returns_next() {
        let mut step = StepStateMultipleRolls::new();
        let mut game = make_game();
        let out = step.start(&mut game, &mut ffb_model::util::rng::GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepStateMultipleRolls::new();
        assert!(!step.set_parameter(&StepParameter::NrOfDice(3)));
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
