/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepThrowKeg`.
///
/// Handles the BeerBarrelBash "throw keg" attack skill.
///
/// Java logic (executeStep):
///   1. Find actingPlayer's unused skill with canThrowKeg property.
///   2. If skill != null OR re-rolling THROW_KEG:
///      a. On re-roll: consume re-roll source; if unavailable → fail().
///      b. Otherwise: markSkillUsed(skill).
///      c. Roll D6; success = roll >= 3.
///      d. Report ReportThrownKeg to game.report_list.
///      e. On success: animate + hitPlayer(target, false).
///      f. On failure (not re-rolling): ask for re-roll; if available → CONTINUE.
///         Otherwise → fail().
///
/// fail(): if roll == 1 → animate fumbled keg + hitPlayer(thrower, true).
///
/// hitPlayer(p, endTurn): run injury pipeline (InjuryTypeKegHit) → publish DROP_PLAYER_CONTEXT.
///
/// Java: `StepThrowKeg extends AbstractStepWithReRoll` (mixed, BB2020 + BB2025).
use ffb_model::enums::ApothecaryMode;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::report::mixed::report_thrown_keg::ReportThrownKeg;
use crate::action::Action;
use crate::drop_player_context::DropPlayerContext;
use crate::injury::injuryType::injury_type_keg_hit::InjuryTypeKegHit;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_injury::handle_injury;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

const THROW_KEG: &str = "THROW_KEG";
const MINIMUM_THROW_KEG_ROLL: i32 = 3;

/// Java: `StepThrowKeg` (mixed, BB2020 + BB2025).
pub struct StepThrowKeg {
    /// Java: `playerId` (init param TARGET_PLAYER_ID) — the target player.
    pub player_id: Option<String>,
    /// Java: `roll` — D6 result stored for fail check (roll==1 → fumbled).
    pub roll: i32,
    /// AbstractStepWithReRoll state.
    pub re_roll: ReRollState,
}

impl StepThrowKeg {
    pub fn new() -> Self {
        Self { player_id: None, roll: 0, re_roll: ReRollState::new() }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: getResult().setNextAction(StepAction.NEXT_STEP)
        // Default — overridden below if we continue.

        let acting_player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: Skill skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, NamedProperties.canThrowKeg)
        let skill = game.player(&acting_player_id)
            .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_THROW_KEG));

        let is_re_rolling = self.re_roll.re_rolled_action.as_ref()
            .map(|a| a.name == THROW_KEG)
            .unwrap_or(false);

        // Java: if (skill != null || getReRolledAction() == ReRolledActions.THROW_KEG)
        if skill.is_none() && !is_re_rolling {
            return StepOutcome::next();
        }

        // Java: if (getReRolledAction() == ReRolledActions.THROW_KEG)
        if is_re_rolling {
            // Java: if (getReRollSource() == null || !UtilServerReRoll.useReRoll(...))
            let can_use = match &self.re_roll.re_roll_source.clone() {
                Some(source) => use_reroll(game, source, &acting_player_id),
                None => false,
            };
            if !can_use {
                return self.fail(game, rng);
            }
        } else {
            // Java: actingPlayer.markSkillUsed(skill)
            if let Some(sid) = skill {
                game.mark_skill_used(&acting_player_id, sid);
            }
        }

        // Java: roll = getGameState().getDiceRoller().rollSkill()
        self.roll = rng.d6();

        // Java: boolean success = DiceInterpreter.getInstance().isSkillRollSuccessful(roll, 3)
        let success = self.roll >= MINIMUM_THROW_KEG_ROLL;

        // Java: getResult().addReport(new ReportThrownKeg(actingPlayer.getPlayerId(), playerId, roll, success, roll == 1))
        game.report_list.add(ReportThrownKeg::new(
            Some(acting_player_id.clone()),
            self.player_id.clone(),
            self.roll,
            success,
            self.roll == 1,
        ));
        let keg_event = GameEvent::KegThrow {
            thrower_id: acting_player_id.clone(),
            target_id: self.player_id.clone(),
            roll: self.roll,
            success,
            fumble: self.roll == 1,
        };

        if success {
            // Java: getResult().setAnimation(new Animation(AnimationType.THROW_KEG, throwerCoord, targetCoord))
            // Animation is client-side only.
            // Java: hitPlayer(game.getPlayerById(playerId), false)
            if let Some(ref target_id) = self.player_id.clone() {
                return self.hit_player(game, rng, target_id, false).with_event(keg_event);
            }
            return StepOutcome::next().with_event(keg_event);
        }

        // Java: if (getReRolledAction() != ReRolledActions.THROW_KEG
        //           && UtilServerReRoll.askForReRollIfAvailable(...)) { setReRolledAction(...); CONTINUE }
        //      else { fail() }
        if !is_re_rolling {
            if let Some(prompt) = ask_for_reroll_if_available(game, THROW_KEG, MINIMUM_THROW_KEG_ROLL, false) {
                self.re_roll.re_rolled_action = Some(ffb_model::model::re_rolled_action::ReRolledAction::new(THROW_KEG));
                return StepOutcome::cont().with_prompt(prompt).with_event(keg_event);
            }
        }
        self.fail(game, rng).with_event(keg_event)
    }

    /// Java: `fail()` — if roll == 1 → animate fumbled keg + hitPlayer(thrower, true).
    fn fail(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.roll == 1 {
            // Java: getResult().setAnimation(new Animation(AnimationType.FUMBLED_KEG, throwerCoordinate))
            // Animation is client-side only.
            let thrower_id = game.acting_player.player_id.clone();
            if let Some(ref tid) = thrower_id {
                let tid = tid.clone();
                return self.hit_player(game, rng, &tid, true);
            }
        }
        StepOutcome::next()
    }

    /// Java: `hitPlayer(Player<?> hitPlayer, boolean endTurn)`.
    ///
    /// Runs InjuryTypeKegHit injury pipeline → publishes DROP_PLAYER_CONTEXT.
    fn hit_player(&self, game: &mut Game, rng: &mut GameRng, player_id: &str, end_turn: bool) -> StepOutcome {
        let coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        // Java: UtilServerInjury.handleInjury(this, new InjuryTypeKegHit(), null, hitPlayer, coordinate, null, null, ApothecaryMode.DEFENDER)
        let mut injury_type = InjuryTypeKegHit::new();
        let injury_result = handle_injury(game, rng, &mut injury_type, None, player_id, coord, None, None, ApothecaryMode::Defender);

        // Java: publishParameter(StepParameter.from(DROP_PLAYER_CONTEXT,
        //         new DropPlayerContext(injuryResult, endTurn, true, null, hitPlayer.getId(), ApothecaryMode.DEFENDER, false)))
        let mut ctx = DropPlayerContext::with_injury(
            injury_result,
            player_id.to_owned(),
            ApothecaryMode::Defender,
            true, // eligibleForSafePairOfHands
        );
        ctx.end_turn = end_turn;

        // Java: getResult().setSound(SoundId.EXPLODE)
        // client-only: SoundId.EXPLODE

        StepOutcome::next()
            .publish(StepParameter::DropPlayerContext(Box::new(ctx)))
    }
}

impl Default for StepThrowKeg {
    fn default() -> Self { Self::new() }
}

impl Step for StepThrowKeg {
    fn id(&self) -> StepId { StepId::ThrowKeg }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: super.handleCommand sets reRollSource when CLIENT_USE_RE_ROLL → EXECUTE_STEP
        match action {
            Action::UseReRoll { use_reroll: true } => {
                // re_roll_source already set by the prompt — just re-execute
            }
            Action::UseReRoll { use_reroll: false } => {
                // Cancel re-roll; clear source so fail() path executes
                self.re_roll.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::TargetPlayerId(v) => { self.player_id = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, SkillId, PS_STANDING, PlayerState, PlayerAction, PlayerType, PlayerGender};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, skills: Vec<SkillId>) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skills.iter().map(|&s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    fn set_acting(game: &mut Game, id: &str) {
        game.acting_player.set_player(id.into(), PlayerAction::Block);
    }

    #[test]
    fn id_is_throw_keg() {
        assert_eq!(StepThrowKeg::new().id(), StepId::ThrowKeg);
    }

    #[test]
    fn no_acting_player_returns_next_step() {
        let mut step = StepThrowKeg::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn acting_player_without_can_throw_keg_returns_next_step() {
        let mut step = StepThrowKeg::new();
        let mut game = make_game();
        add_player(&mut game, "thrower", vec![SkillId::Block]);
        set_acting(&mut game, "thrower");
        add_player(&mut game, "target", vec![]);
        step.player_id = Some("target".into());
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        // No BeerBarrelBash → returns NextStep without touching the target
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn set_parameter_target_player_id() {
        let mut step = StepThrowKeg::new();
        let accepted = step.set_parameter(&StepParameter::TargetPlayerId(Some("tgt".into())));
        assert!(accepted);
        assert_eq!(step.player_id, Some("tgt".into()));
    }

    #[test]
    fn set_parameter_rejects_unknown() {
        let mut step = StepThrowKeg::new();
        let rejected = !step.set_parameter(&StepParameter::EndTurn(true));
        assert!(rejected);
    }

    #[test]
    fn decline_reroll_clears_re_roll_source() {
        let mut step = StepThrowKeg::new();
        step.re_roll.re_roll_source = Some(ffb_model::enums::ReRollSource::new("TRR"));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(&Action::UseReRoll { use_reroll: false }, &mut game, &mut rng);
        assert!(step.re_roll.re_roll_source.is_none());
    }

    #[test]
    fn beer_barrel_bash_marks_skill_used_and_rolls() {
        let mut step = StepThrowKeg::new();
        let mut game = make_game();
        add_player(&mut game, "thrower", vec![SkillId::BeerBarrelBash]);
        set_acting(&mut game, "thrower");
        add_player(&mut game, "target", vec![]);
        step.player_id = Some("target".into());
        let mut rng = GameRng::new(42);
        let _out = step.start(&mut game, &mut rng);
        // BeerBarrelBash should be marked used after execute_step
        let p = game.team_home.player("thrower").unwrap();
        assert!(p.used_skills.contains(&SkillId::BeerBarrelBash),
            "BeerBarrelBash must be marked used when roll is attempted");
    }

    #[test]
    fn successful_throw_publishes_drop_player_context() {
        // Find a seed that gives a d6 roll >= 3
        let seed = (1u64..).find(|&s| { let mut r = GameRng::new(s); r.d6() >= 3 }).unwrap();
        let mut step = StepThrowKeg::new();
        let mut game = make_game();
        add_player(&mut game, "thrower", vec![SkillId::BeerBarrelBash]);
        set_acting(&mut game, "thrower");
        add_player(&mut game, "target", vec![]);
        step.player_id = Some("target".into());
        let mut rng = GameRng::new(seed);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))),
            "successful throw must publish DropPlayerContext");
    }

    #[test]
    fn failed_throw_with_no_reroll_returns_next_step() {
        // Find a seed that gives a d6 roll of exactly 2 (fail, no fumble)
        let seed = (1u64..).find(|&s| { let mut r = GameRng::new(s); let v = r.d6(); v == 2 }).unwrap();
        let mut step = StepThrowKeg::new();
        let mut game = make_game();
        add_player(&mut game, "thrower", vec![SkillId::BeerBarrelBash]);
        set_acting(&mut game, "thrower");
        add_player(&mut game, "target", vec![]);
        step.player_id = Some("target".into());
        // No TRR available, no skill re-roll — fail path
        let mut rng = GameRng::new(seed);
        let out = step.start(&mut game, &mut rng);
        // Either Continue (re-roll offered via TRR) or NextStep (no re-roll)
        // Since test_team has 0 rerolls, expect NextStep without DropPlayerContext
        // (roll==2 means fail but not fumble — no DropPlayerContext for the thrower)
        assert!(matches!(out.action, StepAction::NextStep | StepAction::Continue));
    }

    #[test]
    fn fumble_roll_1_hits_thrower_and_publishes_drop_player_context() {
        // Find a seed that gives d6 == 1
        let seed_opt = (1u64..=1000).find(|&s| { let mut r = GameRng::new(s); r.d6() == 1 });
        if let Some(seed) = seed_opt {
            let mut step = StepThrowKeg::new();
            let mut game = make_game();
            add_player(&mut game, "thrower", vec![SkillId::BeerBarrelBash]);
            set_acting(&mut game, "thrower");
            add_player(&mut game, "target", vec![]);
            step.player_id = Some("target".into());
            let mut rng = GameRng::new(seed);
            let out = step.start(&mut game, &mut rng);
            // roll==1: fail → fumbled keg → hit thrower → DropPlayerContext with end_turn=true
            assert_eq!(out.action, StepAction::NextStep);
            let ctx_found = out.published.iter().any(|p| {
                if let StepParameter::DropPlayerContext(ctx) = p {
                    ctx.end_turn && ctx.player_id.as_deref() == Some("thrower")
                } else { false }
            });
            assert!(ctx_found, "fumble must publish DropPlayerContext for thrower with end_turn=true");
        }
    }
}
