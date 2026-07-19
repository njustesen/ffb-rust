use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::option::game_option_id::BOMB_BOUNCES_ON_EMPTY_SQUARES;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.special.StepRecheckExplodeSkill (BB2025).
///
/// Rechecks whether a bomb should explode on the Bombardier's own square after a mid-bounce
/// catch. Only runs the recheck when NOT skipped, the `BOMB_BOUNCES_ON_EMPTY_SQUARES` option
/// is enabled, and a catcher was actually assigned; otherwise falls straight through.
///
/// Init/runtime params: SKIP (default true — set false by `StepInitBomb` when the bomb was
/// caught mid-bounce and the acting player still holds the explode skill), CATCHER_ID.
pub struct StepRecheckExplodeSkill {
    /// Java: catcherId.
    pub catcher_id: Option<String>,
    /// Java: explodeSkillUsed — set from CLIENT_USE_SKILL command.
    pub explode_skill_used: Option<bool>,
    /// Java: skip (defaults to `true`).
    pub skip: bool,
}

impl StepRecheckExplodeSkill {
    pub fn new() -> Self {
        Self { catcher_id: None, explode_skill_used: None, skip: true }
    }
}

impl Default for StepRecheckExplodeSkill {
    fn default() -> Self { Self::new() }
}

impl Step for StepRecheckExplodeSkill {
    fn id(&self) -> StepId { StepId::RecheckExplodeSkill }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: `if (clientCommandUseSkill.getSkill().hasSkillProperty(canForceBombExplosion))`
        // gates the whole branch (setting explodeSkillUsed, report, markSkillUsed).
        if let Action::UseSkill { skill_id, use_skill } = action {
            if skill_id.properties().contains(&NamedProperties::CAN_FORCE_BOMB_EXPLOSION) {
                self.explode_skill_used = Some(*use_skill);
                if let Some(pid) = game.acting_player.player_id.clone() {
                    // Java: addReport(new ReportSkillUse(playerId, skill, isSkillUsed, FORCE_BOMB_EXPLOSION))
                    game.report_list.add(ReportSkillUse::new(
                        Some(pid.clone()),
                        *skill_id,
                        *use_skill,
                        SkillUse::FORCE_BOMB_EXPLOSION,
                    ));
                    if *use_skill {
                        // Java: getGameState().getGame().getActingPlayer().markSkillUsed(skill)
                        let is_home = game.team_home.player(&pid).is_some();
                        if is_home {
                            if let Some(p) = game.team_home.player_mut(&pid) { p.used_skills.insert(*skill_id); }
                        } else if let Some(p) = game.team_away.player_mut(&pid) {
                            p.used_skills.insert(*skill_id);
                        }
                    }
                }
            }
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::Skip(v) => { self.skip = *v; true }
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepRecheckExplodeSkill {
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        game.turn_data_mut().bomb_used = true;

        if !self.skip
            && game.options.is_enabled(BOMB_BOUNCES_ON_EMPTY_SQUARES)
            && self.catcher_id.is_some()
        {
            // Java: Skill explodeSkill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canForceBombExplosion)
            let has_explode_skill = game.acting_player.player_id.as_deref()
                .and_then(|pid| game.player(pid))
                .map(|p| p.has_unused_skill_with_property(NamedProperties::CAN_FORCE_BOMB_EXPLOSION))
                .unwrap_or(false);

            if has_explode_skill {
                if self.explode_skill_used.is_none() {
                    // Java: show dialog → CONTINUE, wait for CLIENT_USE_SKILL
                    return StepOutcome::cont();
                }
            } else if self.explode_skill_used.is_none() {
                self.explode_skill_used = Some(false);
            }

            if self.explode_skill_used == Some(true) {
                self.catcher_id = None;
            }
        }

        // Java: leaveStep() → publishParameter(CATCHER_ID=catcherId); NEXT_STEP
        StepOutcome::next().publish(StepParameter::CatcherId(self.catcher_id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn enable_bounce_option(game: &mut Game) {
        game.options.set(BOMB_BOUNCES_ON_EMPTY_SQUARES, "true");
    }

    #[test]
    fn default_skip_true_passes_straight_through() {
        // Java default: `skip = true`, so the whole recheck block is bypassed regardless of
        // catcher/skill state — this was previously missing (the old stub always ran the
        // recheck whenever there was no acting player).
        let mut game = make_game();
        enable_bounce_option(&mut game);
        let mut step = StepRecheckExplodeSkill::new();
        step.catcher_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatcherId(Some(id)) if id == "p1")));
    }

    #[test]
    fn skip_false_without_catcher_passes_through() {
        let mut game = make_game();
        enable_bounce_option(&mut game);
        let mut step = StepRecheckExplodeSkill::new();
        step.skip = false;
        // catcher_id is None → gate condition false → straight through
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))));
    }

    #[test]
    fn skip_false_bounce_disabled_passes_through() {
        let mut game = make_game();
        // bounce option NOT enabled
        let mut step = StepRecheckExplodeSkill::new();
        step.skip = false;
        step.catcher_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatcherId(Some(id)) if id == "p1")));
    }

    #[test]
    fn recheck_active_with_explode_skill_waits_for_choice() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerAction};

        let mut game = make_game();
        enable_bounce_option(&mut game);
        let p = Player {
            id: "b1".into(), name: "b1".into(), nr: 1,
            position_id: "pos".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Kaboom, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(p);
        game.acting_player.set_player("b1".into(), PlayerAction::Block);

        let mut step = StepRecheckExplodeSkill::new();
        step.skip = false;
        step.catcher_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue, "should wait for skill use choice");
    }

    #[test]
    fn explode_choice_true_clears_catcher() {
        let mut game = make_game();
        enable_bounce_option(&mut game);
        let mut step = StepRecheckExplodeSkill::new();
        step.skip = false;
        step.catcher_id = Some("p1".into());
        step.explode_skill_used = Some(true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))),
            "explode choice = true should clear catcher_id");
    }

    #[test]
    fn handle_use_skill_for_unrelated_skill_is_ignored() {
        // Regression: previously ANY UseSkill command set explode_skill_used, regardless
        // of the reported skill's property.
        let mut game = make_game();
        let mut step = StepRecheckExplodeSkill::new();
        let action = Action::UseSkill { skill_id: SkillId::Block, use_skill: true };
        step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.explode_skill_used.is_none());
    }

    #[test]
    fn handle_use_skill_kaboom_sets_choice() {
        let mut game = make_game();
        let mut step = StepRecheckExplodeSkill::new();
        let action = Action::UseSkill { skill_id: SkillId::Kaboom, use_skill: true };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.explode_skill_used, Some(true));
    }

    #[test]
    fn set_parameter_skip_and_catcher_id() {
        let mut step = StepRecheckExplodeSkill::new();
        assert!(step.set_parameter(&StepParameter::Skip(false)));
        assert!(!step.skip);
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("x".into()))));
        assert_eq!(step.catcher_id.as_deref(), Some("x"));
    }

    #[test]
    fn set_parameter_unrelated_returns_false() {
        let mut step = StepRecheckExplodeSkill::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
