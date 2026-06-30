/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepAlwaysHungry`.
///
/// Step in TTM sequence to handle skill ALWAYS_HUNGRY. Rolls 2+ for "always hungry"
/// (eating the thrown player); on failure rolls 2+ "escape". Both rolls re-rollable.
///
/// Init params: GOTO_LABEL_ON_FAILURE (mandatory), GOTO_LABEL_ON_SUCCESS (mandatory).
/// Optional init: IS_KICKED_PLAYER.
/// Consumed param: THROWN_PLAYER_ID.
///
/// BB2020 difference vs BB2025: uses `pass_used = true` (Java: `setPassUsed`)
/// instead of `ttm_used = true` for non-kicked throws.
use ffb_model::enums::{PassResult, ReRollSource};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `StepAlwaysHungry` (bb2020/ttm).
pub struct StepAlwaysHungry {
    /// Java: `fGotoLabelOnFailure` — mandatory init param.
    pub goto_label_on_failure: String,
    /// Java: `fGotoLabelOnSuccess` — mandatory init param.
    pub goto_label_on_success: String,
    /// Java: `fThrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// Java: `isKicked` — optional init param (BB2020: sets ktmUsed vs passUsed).
    pub is_kicked: bool,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepAlwaysHungry {
    pub fn new(goto_label_on_failure: String, goto_label_on_success: String) -> Self {
        Self {
            goto_label_on_failure,
            goto_label_on_success,
            thrown_player_id: None,
            is_kicked: false,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepAlwaysHungry {
    fn default() -> Self { Self::new(String::new(), String::new()) }
}

impl Step for StepAlwaysHungry {
    fn id(&self) -> StepId { StepId::AlwaysHungry }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(s) => { self.goto_label_on_failure = s.clone(); true }
            StepParameter::GotoLabelOnSuccess(s) => { self.goto_label_on_success = s.clone(); true }
            StepParameter::ThrownPlayerId(v)     => { self.thrown_player_id = v.clone(); true }
            StepParameter::IsKickedPlayer(v)     => { self.is_kicked = *v; true }
            _ => false,
        }
    }
}

impl StepAlwaysHungry {
    /// Java: `executeStep()`
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Guard: thrownPlayer must exist.
        let thrown_player_id = match self.thrown_player_id.as_ref() {
            Some(id) if game.player(id).is_some() => id.clone(),
            _ => return StepOutcome::next(),
        };

        let acting_player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: doAlwaysHungry = hasUnusedSkillWithProperty(actingPlayer, mightEatPlayerToThrow)
        let mut do_always_hungry = game.player(&acting_player_id)
            .map(|p| UtilCards::has_unused_skill_with_property(p, NamedProperties::MIGHT_EAT_PLAYER_TO_THROW))
            .unwrap_or(false);
        // Java: doEscape = hasSkillWithProperty(actingPlayer.getPlayer(), mightEatPlayerToThrow) && !doAlwaysHungry
        let has_eat_skill = game.player(&acting_player_id)
            .map(|p| UtilCards::has_skill_with_property(p, NamedProperties::MIGHT_EAT_PLAYER_TO_THROW))
            .unwrap_or(false);
        let mut do_escape = has_eat_skill && !do_always_hungry;

        if do_always_hungry {
            // Java: if (isKicked) setKtmUsed(true) else setPassUsed(true)
            // NOTE: BB2020 uses setPassUsed (not setTtmUsed like BB2025)
            if self.is_kicked {
                game.turn_data_mut().ktm_used = true;
            } else {
                game.turn_data_mut().pass_used = true;
            }

            // Java: if (ALWAYS_HUNGRY == reRolledAction) { if (source == null || !useReRoll) doEscape=true; doAlwaysHungry=false }
            if self.re_rolled_action.as_deref() == Some("ALWAYS_HUNGRY") {
                let consumed = if let Some(ref source_name) = self.re_roll_source.clone() {
                    let source = ReRollSource::new(source_name.as_str());
                    use_reroll(game, &source, &acting_player_id)
                } else {
                    false // player declined
                };
                if !consumed {
                    do_escape = true;
                    do_always_hungry = false;
                }
            }

            if do_always_hungry {
                let roll = rng.d6();
                let successful = roll >= 2;
                // Java: reRolled = (reRolledAction == ALWAYS_HUNGRY && reRollSource != null)
                let re_rolled = self.re_rolled_action.as_deref() == Some("ALWAYS_HUNGRY")
                    && self.re_roll_source.is_some();
                let event = GameEvent::AlwaysHungry {
                    player_id: acting_player_id.clone(),
                    roll,
                    success: successful,
                };

                if successful {
                    return StepOutcome::next().with_event(event);
                }

                do_escape = true;
                // Java: if (reRolledAction != ALWAYS_HUNGRY) { setReRolledAction; if (askForReRoll) doEscape=false; }
                if !re_rolled {
                    if let Some(prompt) = ask_for_reroll_if_available(game, "ALWAYS_HUNGRY", 2, false) {
                        self.re_rolled_action = Some("ALWAYS_HUNGRY".into());
                        self.re_roll_source = Some("TRR".into());
                        do_escape = false;
                        return StepOutcome::cont().with_event(event).with_prompt(prompt);
                    }
                }
                // No re-roll or already re-rolled → fall through to doEscape
                let _ = event; // event is moved above; escape block emits its own event
            }
        }

        if do_escape {
            // Java: skill = getUnusedSkillWithProperty(actingPlayer, mightEatPlayerToThrow); markSkillUsed(skill)
            let skill_id = game.player(&acting_player_id)
                .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::MIGHT_EAT_PLAYER_TO_THROW));
            if let Some(sid) = skill_id {
                let is_home = game.team_home.player(&acting_player_id).is_some();
                if is_home {
                    if let Some(p) = game.team_home.player_mut(&acting_player_id) { p.used_skills.insert(sid); }
                } else if let Some(p) = game.team_away.player_mut(&acting_player_id) {
                    p.used_skills.insert(sid);
                }
            }

            // Java: if (ESCAPE == reRolledAction) { if (source == null || !useReRoll) goto failure; return }
            if self.re_rolled_action.as_deref() == Some("ESCAPE") {
                let consumed = if let Some(ref source_name) = self.re_roll_source.clone() {
                    let source = ReRollSource::new(source_name.as_str());
                    use_reroll(game, &source, &thrown_player_id)
                } else {
                    false // player declined
                };
                if !consumed {
                    return StepOutcome::goto(&self.goto_label_on_failure);
                }
            }

            let roll = rng.d6();
            let successful = roll >= 2;
            let escape_event = GameEvent::EscapeRoll {
                player_id: thrown_player_id.clone(),
                roll,
                success: successful,
            };

            if successful {
                return StepOutcome::goto(&self.goto_label_on_success)
                    .with_event(escape_event)
                    .publish(StepParameter::PassResultParam(PassResult::Fumble));
            }

            // Java: if (reRolledAction != ESCAPE) { setReRolledAction(ESCAPE); if (askForReRoll) return; }
            if self.re_rolled_action.as_deref() != Some("ESCAPE") {
                if let Some(prompt) = ask_for_reroll_if_available(game, "ESCAPE", 2, false) {
                    self.re_rolled_action = Some("ESCAPE".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_event(escape_event).with_prompt(prompt);
                }
            }
            return StepOutcome::goto(&self.goto_label_on_failure).with_event(escape_event);
        }

        // Java: if (!doAlwaysHungry && !doEscape) → NEXT_STEP
        StepOutcome::next()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::util::rng::GameRng;

    fn add_player_with_skills(game: &mut Game, team_home: bool, id: &str, skills: Vec<SkillId>) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        };
        if team_home { game.team_home.players.push(player); }
        else { game.team_away.players.push(player); }
    }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn make_game_with_always_hungry() -> Game {
        let mut game = make_game();
        game.home_playing = true;
        add_player_with_skills(&mut game, true, "thrower", vec![SkillId::AlwaysHungry]);
        add_player_with_skills(&mut game, true, "thrown", vec![]);
        game.acting_player.player_id = Some("thrower".into());
        game
    }

    fn seed_for_d6(target: i32) -> u64 {
        for s in 0u64..10_000 {
            if GameRng::new(s).d6() == target { return s; }
        }
        panic!("no seed for d6={}", target);
    }

    // ── id ──────────────────────────────────────────────────────────────────

    #[test]
    fn id_is_always_hungry() {
        assert_eq!(StepAlwaysHungry::default().id(), StepId::AlwaysHungry);
    }

    // ── no thrown player ────────────────────────────────────────────────────

    #[test]
    fn no_thrown_player_returns_next() {
        let mut game = make_game();
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // ── set_parameter ───────────────────────────────────────────────────────

    #[test]
    fn set_parameter_thrown_player_id() {
        let mut step = StepAlwaysHungry::default();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerId(Some("p1".into()))));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn set_parameter_goto_labels() {
        let mut step = StepAlwaysHungry::default();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("fail".into())));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("ok".into())));
        assert_eq!(step.goto_label_on_failure, "fail");
        assert_eq!(step.goto_label_on_success, "ok");
    }

    #[test]
    fn set_parameter_is_kicked() {
        let mut step = StepAlwaysHungry::default();
        assert!(step.set_parameter(&StepParameter::IsKickedPlayer(true)));
        assert!(step.is_kicked);
    }

    #[test]
    fn unknown_parameter_returns_false() {
        let mut step = StepAlwaysHungry::default();
        assert!(!step.set_parameter(&StepParameter::ThrowScatter(true)));
    }

    // ── no skill → no roll ──────────────────────────────────────────────────

    #[test]
    fn no_always_hungry_skill_returns_next() {
        let mut game = make_game();
        game.home_playing = true;
        add_player_with_skills(&mut game, true, "thrower", vec![]); // no AlwaysHungry
        add_player_with_skills(&mut game, true, "thrown", vec![]);
        game.acting_player.player_id = Some("thrower".into());
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    // ── pass_used set (BB2020 specific) ─────────────────────────────────────

    #[test]
    fn pass_used_set_when_not_kicked() {
        let seed = seed_for_d6(6);
        let mut game = make_game_with_always_hungry();
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        step.is_kicked = false;
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(game.turn_data_home.pass_used, "BB2020: setPassUsed(true) for non-kicked");
        assert!(!game.turn_data_home.ktm_used);
    }

    #[test]
    fn ktm_used_set_when_kicked() {
        let seed = seed_for_d6(6);
        let mut game = make_game_with_always_hungry();
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        step.is_kicked = true;
        step.start(&mut game, &mut GameRng::new(seed));
        assert!(game.turn_data_home.ktm_used);
        assert!(!game.turn_data_home.pass_used);
    }

    // ── successful always-hungry roll ────────────────────────────────────────

    #[test]
    fn successful_always_hungry_roll_returns_next() {
        let seed = seed_for_d6(6); // 6 >= 2 → success
        let mut game = make_game_with_always_hungry();
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::AlwaysHungry { success: true, .. })));
    }

    // ── failed roll with no TRR ───────────────────────────────────────────

    #[test]
    fn failed_roll_no_trr_goes_to_escape_then_goto_label() {
        let seed = seed_for_d6(1); // 1 < 2 → always hungry fails
        let mut game = make_game_with_always_hungry();
        // No TRR → no re-roll offered → falls through to escape roll → GotoLabel
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert!(
            matches!(out.action, StepAction::GotoLabel),
            "no TRR → immediate escape: got {:?}", out.action
        );
    }

    // ── failed roll with TRR → offer re-roll ─────────────────────────────────

    #[test]
    fn failed_roll_with_trr_offers_always_hungry_reroll() {
        let seed = seed_for_d6(1); // 1 < 2 → always hungry fails
        let mut game = make_game_with_always_hungry();
        game.turn_data_home.rerolls = 1;
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue, "TRR available → offer re-roll");
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("ALWAYS_HUNGRY"));
    }

    // ── decline always-hungry re-roll → escape ──────────────────────────────

    #[test]
    fn decline_always_hungry_reroll_falls_to_escape() {
        let mut game = make_game_with_always_hungry();
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        step.re_rolled_action = Some("ALWAYS_HUNGRY".into());
        step.re_roll_source = Some("TRR".into());
        // handle_command with false clears re_roll_source → consumption fails → doEscape=true → GotoLabel
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    // ── escape succeeds → goto_on_success + PassResult::Fumble ──────────────

    #[test]
    fn successful_escape_publishes_pass_result_fumble() {
        // Force do_escape path: skill already used (always_hungry already consumed)
        let mut game = make_game_with_always_hungry();
        if let Some(p) = game.team_home.player_mut("thrower") {
            p.used_skills.insert(SkillId::AlwaysHungry);
        }
        let seed = seed_for_d6(6); // escape roll succeeds
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("ok"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::PassResultParam(PassResult::Fumble))));
        assert!(out.events.iter().any(|e| matches!(e, GameEvent::EscapeRoll { success: true, .. })));
    }

    // ── escape fails with TRR → offer escape re-roll ─────────────────────────

    #[test]
    fn failed_escape_with_trr_offers_escape_reroll() {
        let mut game = make_game_with_always_hungry();
        if let Some(p) = game.team_home.player_mut("thrower") {
            p.used_skills.insert(SkillId::AlwaysHungry);
        }
        game.turn_data_home.rerolls = 1;
        let seed = seed_for_d6(1); // 1 < 2 → escape fails
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        let out = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(out.action, StepAction::Continue, "TRR for escape → offer re-roll");
        assert!(out.prompt.is_some());
        assert_eq!(step.re_rolled_action.as_deref(), Some("ESCAPE"));
    }

    // ── decline escape re-roll → goto failure ─────────────────────────────────

    #[test]
    fn decline_escape_reroll_gotos_failure() {
        let mut game = make_game_with_always_hungry();
        if let Some(p) = game.team_home.player_mut("thrower") {
            p.used_skills.insert(SkillId::AlwaysHungry);
        }
        let mut step = StepAlwaysHungry::new("fail".into(), "ok".into());
        step.thrown_player_id = Some("thrown".into());
        step.re_rolled_action = Some("ESCAPE".into());
        step.re_roll_source = Some("TRR".into());
        let out = step.handle_command(
            &Action::UseReRoll { use_reroll: false },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    // ── new stores labels ─────────────────────────────────────────────────────

    #[test]
    fn new_stores_labels() {
        let s = StepAlwaysHungry::new("fail".into(), "ok".into());
        assert_eq!(s.goto_label_on_failure, "fail");
        assert_eq!(s.goto_label_on_success, "ok");
    }
}
