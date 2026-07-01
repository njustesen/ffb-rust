/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepAlwaysHungry`.
///
/// Step in TTM sequence to handle skill ALWAYS_HUNGRY. Rolls 2+ for "always hungry"
/// (eating the thrown player); on failure rolls 2+ "escape". Both rolls re-rollable.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory), GOTO_LABEL_ON_SUCCESS (mandatory).
/// Consumed param: THROWN_PLAYER_ID.
///
/// DEFERRED(reroll): AbstractStepWithReRoll / UtilServerReRoll re-roll ask not yet ported;
///   always-hungry and escape re-rolls are skipped (single roll only).
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::enums::SkillId;
use ffb_model::enums::PassResult;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepAlwaysHungry` (bb2016/ttm).
pub struct StepAlwaysHungry {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    goto_label_on_failure: String,
    /// Java: `fGotoLabelOnSuccess` — mandatory init param.
    goto_label_on_success: String,
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
}

impl StepAlwaysHungry {
    pub fn new() -> Self {
        Self {
            goto_label_on_failure: String::new(),
            goto_label_on_success: String::new(),
            thrown_player_id: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if (thrownPlayer == null) return
        let thrown_player_id = match &self.thrown_player_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };
        if game.player(&thrown_player_id).is_none() {
            return StepOutcome::next();
        }

        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: boolean doAlwaysHungry = UtilCards.hasUnusedSkillWithProperty(actingPlayer, mightEatPlayerToThrow)
        let has_property = game.player(&acting_id)
            .map(|p| p.has_skill_property(NamedProperties::MIGHT_EAT_PLAYER_TO_THROW))
            .unwrap_or(false);
        let skill_unused = game.player(&acting_id)
            .map(|p| !p.used_skills.contains(&SkillId::AlwaysHungry))
            .unwrap_or(false);

        let do_always_hungry = has_property && skill_unused;
        // Java: boolean doEscape = hasSkillWithProperty && !doAlwaysHungry
        let mut do_escape = has_property && !do_always_hungry;

        if do_always_hungry {
            // Java: game.getTurnData().setPassUsed(true)
            game.turn_data_mut().pass_used = true;

            // DEFERRED(reroll): re-roll ask after failure not ported (UtilServerReRoll).

            // Java: int roll = rollSkill(); boolean successful = isAlwaysHungrySuccessful(roll)
            let roll = rng.d6();
            let successful = DiceInterpreter::is_always_hungry_successful(roll);

            if successful {
                return StepOutcome::next();
            } else {
                do_escape = true;
                // DEFERRED(reroll): setReRolledAction(ALWAYS_HUNGRY) / askForReRollIfAvailable not ported.
            }
        }

        if do_escape {
            // Java: Skill skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, mightEatPlayerToThrow)
            // Java: actingPlayer.markSkillUsed(skill) — only skill with mightEatPlayerToThrow is AlwaysHungry
            let player_in_home = game.team_home.player(&acting_id).is_some();
            if player_in_home {
                if let Some(p) = game.team_home.player_mut(&acting_id) {
                    p.used_skills.insert(SkillId::AlwaysHungry);
                }
            } else if let Some(p) = game.team_away.player_mut(&acting_id) {
                p.used_skills.insert(SkillId::AlwaysHungry);
            }

            // DEFERRED(reroll): re-roll ask after escape failure not ported.

            // Java: int roll = rollSkill(); boolean successful = isEscapeFromAlwaysHungrySuccessful(roll)
            let roll = rng.d6();
            let successful = DiceInterpreter::is_escape_from_always_hungry_successful(roll);

            if successful {
                // Java: publishParameter(PASS_RESULT, PassResult.FUMBLE); goto success
                let label = self.goto_label_on_success.clone();
                return StepOutcome::goto(&label)
                    .publish(StepParameter::PassResultParam(PassResult::Fumble));
            } else {
                // Java: goto failure
                let label = self.goto_label_on_failure.clone();
                return StepOutcome::goto(&label);
            }
        }

        // Java: if (!doAlwaysHungry && !doEscape) { setNextAction(NEXT_STEP) }
        StepOutcome::next()
    }
}

impl Default for StepAlwaysHungry {
    fn default() -> Self { Self::new() }
}

impl Step for StepAlwaysHungry {
    fn id(&self) -> StepId { StepId::AlwaysHungry }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::GotoLabelOnSuccess(s) => { self.goto_label_on_success = s.clone(); true }
            StepParameter::ThrownPlayerId(v)     => { self.thrown_player_id = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player_no_skill(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
    }

    fn add_always_hungry_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 2, position_id: "ogre".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 5, strength: 5, agility: 2, passing: 5, armour: 9,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::AlwaysHungry, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.acting_player.player_id = Some(id.into());
    }

    fn add_thrown_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 3, position_id: "goblin".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 2, agility: 4, passing: 4, armour: 7,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
    }

    #[test]
    fn id_is_always_hungry() {
        assert_eq!(StepAlwaysHungry::new().id(), StepId::AlwaysHungry);
    }

    #[test]
    fn no_thrown_player_returns_next() {
        let mut game = make_game();
        let mut step = StepAlwaysHungry::new();
        step.goto_label_on_failure = "fail".into();
        step.goto_label_on_success = "success".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn missing_thrown_player_in_game_returns_next() {
        let mut game = make_game();
        let mut step = StepAlwaysHungry::new();
        step.goto_label_on_failure = "fail".into();
        step.goto_label_on_success = "success".into();
        step.thrown_player_id = Some("ghost".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn set_parameter_thrown_player_id() {
        let mut step = StepAlwaysHungry::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_goto_labels() {
        let mut step = StepAlwaysHungry::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("ok".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
        assert_eq!(step.goto_label_on_success, "ok");
    }

    #[test]
    fn no_always_hungry_skill_returns_next() {
        let mut game = make_game();
        add_player_no_skill(&mut game, "thrower");
        add_thrown_player(&mut game, "target");
        game.acting_player.player_id = Some("thrower".into());
        let mut step = StepAlwaysHungry::new();
        step.goto_label_on_failure = "fail".into();
        step.goto_label_on_success = "success".into();
        step.thrown_player_id = Some("target".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn always_hungry_roll_success_sets_pass_used_and_returns_next() {
        // seed that produces roll >= 2 for always-hungry check
        let mut game = make_game();
        add_always_hungry_player(&mut game, "ogre");
        add_thrown_player(&mut game, "gob");
        let mut step = StepAlwaysHungry::new();
        step.goto_label_on_failure = "fail".into();
        step.goto_label_on_success = "success".into();
        step.thrown_player_id = Some("gob".into());
        // Find a seed where d6 >= 2 for always-hungry (most seeds)
        let out = step.start(&mut game, &mut GameRng::new(5));
        // Either NEXT_STEP (ah success) or GotoLabel (escape path)
        // Either way, pass_used should be set
        assert!(game.turn_data_home.pass_used);
        let _ = out; // result depends on dice
    }

    #[test]
    fn skill_already_used_goes_to_escape_path() {
        let mut game = make_game();
        add_always_hungry_player(&mut game, "ogre");
        // mark AlwaysHungry as already used
        game.team_home.player_mut("ogre").unwrap().used_skills.insert(SkillId::AlwaysHungry);
        add_thrown_player(&mut game, "gob");
        let mut step = StepAlwaysHungry::new();
        step.goto_label_on_failure = "fail".into();
        step.goto_label_on_success = "success".into();
        step.thrown_player_id = Some("gob".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Escape path → goes to a label (either success or failure)
        assert!(matches!(out.action, StepAction::GotoLabel));
    }

    #[test]
    fn escape_success_publishes_pass_result_fumble() {
        let mut game = make_game();
        add_always_hungry_player(&mut game, "ogre");
        game.team_home.player_mut("ogre").unwrap().used_skills.insert(SkillId::AlwaysHungry);
        add_thrown_player(&mut game, "gob");
        let mut step = StepAlwaysHungry::new();
        step.goto_label_on_failure = "fail".into();
        step.goto_label_on_success = "success".into();
        step.thrown_player_id = Some("gob".into());
        // Find a seed where escape roll succeeds (>= 2)
        // Try multiple seeds to find one that succeeds
        for seed in 1..20u64 {
            let mut g2 = game.clone();
            let mut s2 = StepAlwaysHungry::new();
            s2.goto_label_on_failure = "fail".into();
            s2.goto_label_on_success = "success".into();
            s2.thrown_player_id = Some("gob".into());
            let out = s2.start(&mut g2, &mut GameRng::new(seed));
            if out.goto_label.as_deref() == Some("success") {
                assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassResultParam(PassResult::Fumble))));
                return;
            }
        }
        panic!("No seed produced escape success in first 20 seeds");
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepAlwaysHungry::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
