use ffb_model::enums::{PlayerAction, ReRollSource, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_hypnotic_gaze_roll::ReportHypnoticGazeRoll;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::mechanics::is_skill_roll_successful;

/// BB2025 minimum roll for hypnotic gaze (fixed at 3 per AgilityMechanic).
const MINIMUM_ROLL_HYPNOTIC_GAZE: i32 = 3;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepHypnoticGaze.
///
/// Init params: GOTO_LABEL_ON_END (mandatory).
/// Sets: END_PLAYER_ACTION for all steps on the stack.
pub struct StepHypnoticGaze {
    /// Java: fGotoLabelOnEnd
    pub goto_label_on_end: String,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepHypnoticGaze {
    pub fn new(goto_label_on_end: String) -> Self {
        Self { goto_label_on_end, re_rolled_action: None, re_roll_source: None }
    }
}

impl Step for StepHypnoticGaze {
    fn id(&self) -> StepId { StepId::HypnoticGaze }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepHypnoticGaze {
    /// Java `AgilityMechanic.minimumRollHypnoticGaze` with `GazeModifierFactory.findModifiers`.
    fn minimum_roll_and_modifiers(game: &Game, player_id: Option<&str>) -> (i32, Vec<String>) {
        if game.rules != ffb_model::enums::Rules::Bb2020 {
            return (MINIMUM_ROLL_HYPNOTIC_GAZE, Vec::new());
        }
        let Some(p) = player_id.and_then(|id| game.player(id)) else {
            return (MINIMUM_ROLL_HYPNOTIC_GAZE, Vec::new());
        };
        use ffb_model::model::property::named_properties::NamedProperties;
        let variable = p.get_skill_int_value(NamedProperties::INFLICTS_CONFUSION);
        let base = if variable == 0 { p.agility_with_modifiers() } else { variable };
        let col = ffb_mechanics::modifiers::bb2020::gaze_modifier_collection::GazeModifierCollection::new();
        let ctx = ffb_mechanics::modifiers::gaze_modifier_context::GazeModifierContext::new(game, p);
        let found = col.find_applicable(&ctx);
        let total: i32 = found.iter().map(|m| m.get_modifier()).sum();
        ((base + total).max(2), found.iter().map(|m| m.get_name().to_string()).collect())
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: doGaze = (playerAction == GAZE) && (defender != null) && defender.team != actingTeam
        let is_gaze_action = game.acting_player.player_action == Some(PlayerAction::Gaze);
        let defender_id = game.defender_id.clone();

        let defender_is_opponent = defender_id.as_deref().map(|id| {
            let home_has = game.team_home.player(id).is_some();
            if game.home_playing { !home_has } else { home_has }
        }).unwrap_or(false);

        let mut do_gaze = is_gaze_action && defender_id.is_some() && defender_is_opponent;

        if !do_gaze {
            game.defender_id = None;
            return StepOutcome::next();
        }

        let acting_player_id = game.acting_player.player_id.clone();

        // Java: if (HYPNOTIC_GAZE == reRolledAction) { if (source == null || !useReRoll) doGaze = false }
        //       else { doGaze = gazeSkill.isPresent() && !hasSkillToCancelProperty }
        if self.re_rolled_action.as_deref() == Some("HYPNOTIC_GAZE") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                let pid = acting_player_id.as_deref().unwrap_or("");
                if !use_reroll(game, &source, pid, rng) {
                    do_gaze = false;
                }
            } else {
                do_gaze = false; // Player declined
            }
        } else {
            let has_gaze_skill = acting_player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::INFLICTS_CONFUSION))
                .unwrap_or(false);
            // hasSkillToCancelProperty — stub as false (not ported)
            do_gaze = has_gaze_skill;
        }

        let mut goto_end_label = true;

        if do_gaze {
            // Java: actingPlayer.markSkillUsed(gazeSkill.get())
            // ActingPlayer.markSkillUsed: the activation's set (and the player's only if the skill
            // is tracked outside the activation) -- the gaze may be declared again next
            // activation (vampire bb2025 seed 4, §K).
            if let Some(pid) = acting_player_id.clone() {
                crate::step::util_server_steps::mark_skill_used(game, &pid, SkillId::HypnoticGaze);
            }

            let roll = rng.d6();
            // Java `mechanic.minimumRollHypnoticGaze(player, GazeModifierFactory.findModifiers(..))`.
            // bb2025: a flat 3, no modifiers. bb2020 (this shared step runs it too): the
            // inflictsConfusion skill value, else agility, plus the tackle-zone modifier, floored
            // at 2 -- and the report names that modifier. Rust used the bb2025 flat 3 for both.
            let (minimum_roll, modifier_names) = Self::minimum_roll_and_modifiers(game, acting_player_id.as_deref());
            let successful = is_skill_roll_successful(roll, minimum_roll);

            let re_rolled = self.re_rolled_action.as_deref() == Some("HYPNOTIC_GAZE")
                && self.re_roll_source.is_some();
            let mut report = ReportHypnoticGazeRoll::new(
                acting_player_id.clone(),
                successful,
                roll,
                minimum_roll,
                re_rolled,
                defender_id.clone(),
            );
            report.base.roll_modifier_names = modifier_names;
            game.report_list.add(report);

            if successful {
                // Java: PlayerState oldVictimState = ...; if (!oldVictimState.isConfused())
                //         setPlayerState(defender, oldVictimState.changeConfused(true));
                // bb2020 StepHypnoticGaze: if (!isConfused() && !isHypnotized()) changeHypnotized(true).
                if let Some(ref did) = defender_id {
                    if let Some(old_state) = game.field_model.player_state(did) {
                        if game.rules == ffb_model::enums::Rules::Bb2020 {
                            if !old_state.is_confused() && !old_state.is_hypnotized() {
                                game.field_model.set_player_state(did, old_state.change_hypnotized(true));
                            }
                        } else if !old_state.is_confused() {
                            game.field_model.set_player_state(did, old_state.change_confused(true));
                        }
                    }
                }
            } else if self.re_rolled_action.is_none() {
                // Java: if (reRolledAction != HYPNOTIC_GAZE && askForReRollIfAvailable(...)) gotoEndLabel = false
                let pid = acting_player_id.as_deref().unwrap_or("");
                if let Some(prompt) = ask_for_reroll_if_available(game, "HYPNOTIC_GAZE", minimum_roll, false) {
                    self.re_rolled_action = Some("HYPNOTIC_GAZE".into());
                    self.re_roll_source = Some("TRR".into());
                    let _ = pid;
                    goto_end_label = false;
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
        }

        if goto_end_label {
            game.defender_id = None;
            let label = self.goto_label_on_end.clone();
            StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true))
        } else {
            StepOutcome::cont()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerAction, Rules, SkillId};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    /// bb2020 `AgilityMechanic.minimumRollHypnoticGaze`: agility plus the ONE tackle-zone gaze
    /// modifier (N-1 for N opposing tackle zones), floored at 2, the modifier named in the
    /// report. bb2025 is a flat 3 with no modifiers. Rust used the flat 3 for both, and the
    /// bb2020 collection matched all eight modifiers (§K bb2020 gaze).
    #[test]
    fn bb2020_gaze_minimum_counts_tacklezones() {
        for (rules, opponents, expected) in [
            (Rules::Bb2020, 1, (2, vec!["1 Tacklezone".to_string()])),
            (Rules::Bb2020, 2, (3, vec!["2 Tacklezones".to_string()])),
            (Rules::Bb2025, 2, (3, Vec::new())),
        ] {
            let mut game = Game::new(test_team("home", 0), test_team("away", 0), rules);
            game.home_playing = true;
            game.team_home.players.push(Player {
                id: "vamp".into(), name: "vamp".into(), nr: 1, position_id: "p".into(),
                movement: 6, strength: 4, agility: 2, passing: 3, armour: 9,
                starting_skills: vec![SkillWithValue { skill_id: SkillId::HypnoticGaze, value: None }],
                ..Default::default()
            });
            game.field_model.set_player_coordinate("vamp", FieldCoordinate::new(10, 7));
            game.field_model.set_player_state("vamp",
                ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING).change_active(true));
            for i in 0..opponents {
                let id = format!("opp{i}");
                game.team_away.players.push(Player {
                    id: id.clone(), name: id.clone(), nr: 2 + i, position_id: "p".into(),
                    movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
                    ..Default::default()
                });
                game.field_model.set_player_coordinate(&id, FieldCoordinate::new(11, 6 + i));
                game.field_model.set_player_state(&id,
                    ffb_model::enums::PlayerState::new(ffb_model::enums::PS_STANDING));
            }
            assert_eq!(StepHypnoticGaze::minimum_roll_and_modifiers(&game, Some("vamp")), expected,
                "{rules:?} with {opponents} adjacent opponents");
        }
    }

    #[test]
    fn non_gaze_action_returns_next_step() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Move);
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn no_action_returns_next_step() {
        let mut game = make_game();
        game.acting_player.player_action = None;
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn gaze_no_defender_returns_next_step() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = None;
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn gaze_with_defender_clears_defender_and_goes_to_end_label() {
        let mut game = make_game();
        // Setup: home team player has HypnoticGaze; away team player is defender
        let mut attacker = Player::default();
        attacker.id = "a1".into();
        attacker.starting_skills.push(SkillWithValue::new(SkillId::HypnoticGaze));
        game.team_home.players.push(attacker);
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(5, 5));

        let mut defender = Player::default();
        defender.id = "d1".into();
        game.team_away.players.push(defender);
        game.field_model.set_player_coordinate("d1", FieldCoordinate::new(6, 5));

        game.home_playing = true;
        game.acting_player.player_id = Some("a1".into());
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("d1".into());

        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
        // defender_id should be cleared
        assert!(game.defender_id.is_none());
    }

    /// Java: `if (!oldVictimState.isConfused()) setPlayerState(defender, oldVictimState.changeConfused(true))`.
    /// Before the fix, the Rust code called `change_hypnotized(true)` (a different bit) with an
    /// extra `!is_hypnotized()` guard not present in Java — so `is_confused()` would have stayed
    /// false and this test would have failed.
    #[test]
    fn successful_gaze_sets_defender_confused() {
        let mut game = make_game();
        let mut attacker = Player::default();
        attacker.id = "a1".into();
        attacker.starting_skills.push(SkillWithValue::new(SkillId::HypnoticGaze));
        game.team_home.players.push(attacker);

        let mut defender = Player::default();
        defender.id = "d1".into();
        game.team_away.players.push(defender);

        game.home_playing = true;
        game.acting_player.player_id = Some("a1".into());
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("d1".into());

        // Set active standing state for defender
        const ACTIVE_STANDING: ffb_model::enums::PlayerState = ffb_model::enums::PlayerState(0x101);
        game.field_model.set_player_state("d1", ACTIVE_STANDING);

        // Find seed that rolls >= 3 (success)
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            if rng.d6() >= MINIMUM_ROLL_HYPNOTIC_GAZE {
                let mut g = make_game();
                let mut a = Player::default();
                a.id = "a1".into();
                a.starting_skills.push(SkillWithValue::new(SkillId::HypnoticGaze));
                g.team_home.players.push(a);
                let mut d = Player::default();
                d.id = "d1".into();
                g.team_away.players.push(d);
                g.home_playing = true;
                g.acting_player.player_id = Some("a1".into());
                g.acting_player.player_action = Some(PlayerAction::Gaze);
                g.defender_id = Some("d1".into());
                g.field_model.set_player_state("d1", ACTIVE_STANDING);

                let mut s = StepHypnoticGaze::new("end".into());
                s.start(&mut g, &mut GameRng::new(seed));
                let state = g.field_model.player_state("d1").unwrap();
                assert!(state.is_confused(), "seed={seed}: roll passed but defender not confused");
                return;
            }
        }
        panic!("no seed rolls >= 3");
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepHypnoticGaze::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn gaze_adds_hypnotic_gaze_roll_report() {
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game();
        let mut attacker = Player::default();
        attacker.id = "a1".into();
        attacker.starting_skills.push(SkillWithValue::new(SkillId::HypnoticGaze));
        game.team_home.players.push(attacker);
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(5, 5));
        let mut defender = Player::default();
        defender.id = "d1".into();
        game.team_away.players.push(defender);
        game.field_model.set_player_coordinate("d1", FieldCoordinate::new(6, 5));
        game.home_playing = true;
        game.acting_player.player_id = Some("a1".into());
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("d1".into());
        game.turn_data_home.rerolls = 0;
        let mut step = StepHypnoticGaze::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::HYPNOTIC_GAZE_ROLL), "ReportHypnoticGazeRoll must be added");
    }

    #[test]
    fn gaze_marks_hypnotic_gaze_skill_used() {
        let mut game = make_game();
        let mut attacker = Player::default();
        attacker.id = "a1".into();
        attacker.starting_skills.push(SkillWithValue::new(SkillId::HypnoticGaze));
        game.team_home.players.push(attacker);
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(5, 5));
        let mut defender = Player::default();
        defender.id = "d1".into();
        game.team_away.players.push(defender);
        game.field_model.set_player_coordinate("d1", FieldCoordinate::new(6, 5));
        game.home_playing = true;
        game.acting_player.player_id = Some("a1".into());
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("d1".into());
        game.turn_data_home.rerolls = 0;
        let mut step = StepHypnoticGaze::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        // ActingPlayer.markSkillUsed: the activation's set (§K) -- the player may gaze again on a
        // later activation, which the eligibility list reads from the player's own set.
        assert!(game.acting_player.used_skills.contains(&SkillId::HypnoticGaze),
            "HypnoticGaze skill must be marked as used for the activation");
    }
}
