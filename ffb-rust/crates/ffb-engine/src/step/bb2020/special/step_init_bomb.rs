use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_bomb_explodes_after_catch::ReportBombExplodesAfterCatch;
use ffb_model::report::report_bomb_out_of_bounds::ReportBombOutOfBounds;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::model::skill_use::SkillUse;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initialises the bomb throw sequence (BB2020).
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.special.StepInitBomb`.
pub struct StepInitBomb {
    goto_label_on_end: Option<String>,
    catcher_id: Option<String>,
    pass_fumble: bool,
    bomb_out_of_bounds: bool,
    dont_drop_fumble: bool,
    /// Java: explodeSkillUsed — None = not yet decided, Some(true/false) = skill used/declined.
    explode_skill_used: Option<bool>,
}

impl StepInitBomb {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: None,
            catcher_id: None,
            pass_fumble: false,
            bomb_out_of_bounds: false,
            dont_drop_fumble: false,
            explode_skill_used: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: if fPassFumble → fCatcherId = null
        if self.pass_fumble {
            self.catcher_id = None;
        }
        // Java: if fBombOutOfBounds → fCatcherId = null
        if self.bomb_out_of_bounds {
            self.catcher_id = None;
        }

        if self.catcher_id.is_some() {
            // Java: if explodeSkillUsed == null → default false (no dialog in headless path)
            let explode_skill_used = self.explode_skill_used.unwrap_or(false);
            let explodes = if explode_skill_used {
                true
            } else {
                // Java: int roll = getDiceRoller().rollDice(6); explodes = roll >= 4
                let roll = rng.d6();
                let explodes = roll >= 4;
                // Java: addReport(new ReportBombExplodesAfterCatch(fCatcherId, explodes, roll))
                let catcher_id = self.catcher_id.clone().unwrap_or_default();
                game.report_list.add(ReportBombExplodesAfterCatch::new(catcher_id, explodes, roll));
                explodes
            };
            if explodes {
                self.catcher_id = None;
            }
        }

        if self.catcher_id.is_none() {
            let bomb_coordinate = game.field_model.bomb_coordinate;
            if bomb_coordinate.is_none() {
                // Java: if (!dontDropFumble) addReport(new ReportBombOutOfBounds())
                if !self.dont_drop_fumble {
                    game.report_list.add(ReportBombOutOfBounds::new());
                }
            }
            // Java: leaveStep(null) → NEXT_STEP
            StepOutcome::next()
                .publish(StepParameter::CatcherId(self.catcher_id.clone()))
        } else {
            // Java: leaveStep(fGotoLabelOnEnd) → GOTO_LABEL
            if let Some(ref label) = self.goto_label_on_end.clone() {
                StepOutcome::goto(label)
                    .publish(StepParameter::CatcherId(self.catcher_id.clone()))
            } else {
                StepOutcome::next()
                    .publish(StepParameter::CatcherId(self.catcher_id.clone()))
            }
        }
    }
}

impl Default for StepInitBomb {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitBomb {
    fn id(&self) -> StepId { StepId::InitBomb }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_USE_SKILL with canForceBombExplosion skill
        // → addReport(new ReportSkillUse(..., SkillUse.FORCE_BOMB_EXPLOSION))
        if let Action::UseSkill { skill_id, use_skill } = action {
            self.explode_skill_used = Some(*use_skill);
            game.report_list.add(ReportSkillUse::new(
                None,
                *skill_id,
                *use_skill,
                SkillUse::FORCE_BOMB_EXPLOSION,
            ));
            // Java: if (explodeSkillUsed) actingPlayer.markSkillUsed(clientCommandUseSkill.getSkill())
            if *use_skill {
                if let Some(pid) = game.acting_player.player_id.clone() {
                    game.mark_skill_used(&pid, *skill_id);
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::BombOutOfBounds(v) => {
                self.bomb_out_of_bounds = *v;
                true
            }
            StepParameter::CatcherId(v) => {
                self.catcher_id = v.clone();
                true
            }
            StepParameter::PassFumble(v) => {
                self.pass_fumble = *v;
                true
            }
            StepParameter::GotoLabelOnEnd(v) => {
                self.goto_label_on_end = Some(v.clone());
                true
            }
            StepParameter::DontDropFumble(v) => {
                self.dont_drop_fumble = *v;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_model::report::report_id::ReportId;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_init_bomb() {
        let step = StepInitBomb::new();
        assert_eq!(step.id(), StepId::InitBomb);
    }

    #[test]
    fn no_catcher_returns_next_step() {
        let mut step = StepInitBomb::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert_eq!(outcome.action, StepAction::NextStep);
        // CatcherId(None) should be published
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))));
    }

    #[test]
    fn pass_fumble_clears_catcher() {
        let mut step = StepInitBomb::new();
        // Set a catcher first
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-1".to_string()))));
        // Set pass fumble
        assert!(step.set_parameter(&StepParameter::PassFumble(true)));

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);

        // Should return NextStep and publish CatcherId(None)
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))));
    }

    #[test]
    fn bomb_out_of_bounds_clears_catcher() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-2".to_string()))));
        assert!(step.set_parameter(&StepParameter::BombOutOfBounds(true)));

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);

        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::CatcherId(None))));
    }

    #[test]
    fn catcher_with_goto_label_returns_goto() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-3".to_string()))));
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("label_catch".to_string())));
        // Java: explodeSkillUsed = false → bomb does NOT explode (skill declined) → catcher retained
        // Bypass dice roll by pre-setting explode_skill_used to Some(false)
        step.explode_skill_used = Some(false);

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);

        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("label_catch"));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::CatcherId(Some(_)))));
    }

    #[test]
    fn bomb_explodes_after_catch_report_added_on_dice_roll() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-4".to_string()))));
        // explode_skill_used = None → will roll dice → adds ReportBombExplodesAfterCatch

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);

        // Java: addReport(new ReportBombExplodesAfterCatch(...)) when dice are rolled
        assert!(game.report_list.has_report(ReportId::BOMB_EXPLODES_AFTER_CATCH));
    }

    #[test]
    fn bomb_out_of_bounds_report_added_when_no_bomb_coordinate() {
        let mut step = StepInitBomb::new();
        // No catcher → goes to bomb_coordinate check; game.field_model.bomb_coordinate = None

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);

        assert_eq!(outcome.action, StepAction::NextStep);
        // Java: addReport(new ReportBombOutOfBounds()) when bomb_coordinate is None and !dontDropFumble
        assert!(game.report_list.has_report(ReportId::BOMB_OUT_OF_BOUNDS));
    }

    #[test]
    fn bomb_out_of_bounds_report_suppressed_by_dont_drop_fumble() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::DontDropFumble(true)));

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);

        // Java: if (!dontDropFumble) → no report when dontDropFumble is true
        assert!(!game.report_list.has_report(ReportId::BOMB_OUT_OF_BOUNDS));
    }

    #[test]
    fn skill_use_report_added_by_handle_command() {
        use ffb_mechanics::skills::SkillId;
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-5".to_string()))));

        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::Bombardier, use_skill: true };
        step.handle_command(&action, &mut game, &mut rng);

        // Java: addReport(new ReportSkillUse(..., SkillUse.FORCE_BOMB_EXPLOSION))
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
    }

    #[test]
    fn skill_marked_used_when_explode_skill_used_true() {
        use ffb_mechanics::skills::SkillId;
        use ffb_model::model::player::Player;
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::CatcherId(Some("player-6".to_string()))));

        let mut game = make_game();
        let pid = "player-6".to_string();
        game.team_home.players.push(Player { id: pid.clone(), ..Default::default() });
        game.acting_player.player_id = Some(pid.clone());
        let mut rng = GameRng::new(0);
        let action = Action::UseSkill { skill_id: SkillId::Bombardier, use_skill: true };
        step.handle_command(&action, &mut game, &mut rng);

        // Java: if (explodeSkillUsed) actingPlayer.markSkillUsed(clientCommandUseSkill.getSkill())
        let player = game.player(&pid).expect("player exists");
        assert!(player.used_skills.contains(&SkillId::Bombardier));
    }

    #[test]
    fn set_bomb_out_of_bounds_accepted() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::BombOutOfBounds(true)));
    }

    #[test]
    fn set_parameter_unknown_returns_false() {
        let mut step = StepInitBomb::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
