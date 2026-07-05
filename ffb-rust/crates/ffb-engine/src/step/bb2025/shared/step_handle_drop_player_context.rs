use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, VictimStateKey};
use crate::step::framework::{Step, StepOutcome, StepParameter};
use crate::step::framework::StepId;
use crate::step::util_server_injury::drop_player;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.shared.StepHandleDropPlayerContext.
///
/// Processes a pending DropPlayerContext (injury follow-up after SteadyFooting fails).
/// Drops the player (unless already dropped or armour break required but not achieved),
/// publishes END_TURN if flagged, and routes to the context's label if one is set.
///
/// Stubbed: ModifiedInjuryContext / Pro skill re-roll — these require ModifiedInjuryContext
/// which is not yet ported. The step skips that branch and proceeds directly to the drop.
///
/// Consumed parameters: DROP_PLAYER_CONTEXT.
/// Published: parameters from drop_player() + INJURY_RESULT + END_TURN + GOTO_LABEL.
pub struct StepHandleDropPlayerContext {
    /// Java: dropPlayerContext
    pub drop_player_context: Option<Box<DropPlayerContext>>,
    // AbstractStepWithReRoll stubs
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepHandleDropPlayerContext {
    pub fn new() -> Self {
        Self {
            drop_player_context: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Default for StepHandleDropPlayerContext {
    fn default() -> Self { Self::new() }
}

impl Step for StepHandleDropPlayerContext {
    fn id(&self) -> StepId { StepId::HandleDropPlayerContext }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::DropPlayerContext(ctx) => {
                self.drop_player_context = Some(ctx.clone());
                true
            }
            _ => false,
        }
    }
}

impl StepHandleDropPlayerContext {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let ctx = match &self.drop_player_context {
            Some(c) => c,
            None => return StepOutcome::next(),
        };
        let injury_result = match &ctx.injury_result {
            Some(r) => r,
            None => return StepOutcome::next(),
        };

        // Java: if (injuryResult.injuryContext().getModifiedInjuryContext() != null && !injuryResult.isAlreadyReported())
        // ModifiedInjuryContext not yet ported — skip that branch entirely.

        let mut out = StepOutcome::next();

        // Java: if (!alreadyDropped && (!requiresArmourBreak || armorBroken)) → dropPlayer(...)
        let should_drop = !ctx.already_dropped
            && (!ctx.requires_armour_break || injury_result.injury_context().is_armor_broken());

        if should_drop {
            let player_id = match &ctx.player_id {
                Some(id) => id.clone(),
                None => return StepOutcome::next(),
            };
            let drop_params = drop_player(game, &player_id, ctx.eligible_for_safe_pair_of_hands);
            for p in drop_params {
                out = out.publish(p);
            }

            // Java: if (endTurn) publishParameter(END_TURN, true)
            if ctx.end_turn {
                out = out.publish(StepParameter::EndTurn(true));
            }

            // Java: if (victimStateKey != null) publishParameter(victimStateKey, defenderState)
            if let Some(vsk) = ctx.victim_state_key {
                let defender_state = game.field_model.player_state(
                    game.acting_player.player_id.as_deref().unwrap_or("")
                );
                if let Some(state) = defender_state {
                    let param = match vsk {
                        VictimStateKey::OldDefenderState => StepParameter::OldDefenderState(state),
                        VictimStateKey::ThrownPlayerState => StepParameter::ThrownPlayerState(state),
                        VictimStateKey::OldPlayerState => StepParameter::OldPlayerState(state),
                        VictimStateKey::KickedPlayerState => StepParameter::KickedPlayerState(state),
                    };
                    out = out.publish(param);
                }
            }

            // Java: additionalVictimStateKeys loop (same pattern)
            for &vsk in &ctx.additional_victim_state_keys {
                let defender_state = game.field_model.player_state(
                    game.acting_player.player_id.as_deref().unwrap_or("")
                );
                if let Some(state) = defender_state {
                    let param = match vsk {
                        VictimStateKey::OldDefenderState => StepParameter::OldDefenderState(state),
                        VictimStateKey::ThrownPlayerState => StepParameter::ThrownPlayerState(state),
                        VictimStateKey::OldPlayerState => StepParameter::OldPlayerState(state),
                        VictimStateKey::KickedPlayerState => StepParameter::KickedPlayerState(state),
                    };
                    out = out.publish(param);
                }
            }
        } else if !ctx.already_dropped && ctx.end_turn_without_knockdown && ctx.end_turn {
            // Java: else if (!alreadyDropped && endTurnWithoutKnockdown && endTurn)
            out = out.publish(StepParameter::EndTurn(true));
        }

        // Java: publishParameter(INJURY_RESULT, injuryResult) — always
        out = out.publish(StepParameter::InjuryResult(injury_result.clone()));

        // Java: if (StringTool.isProvided(label)) GOTO_LABEL(label)
        if let Some(label) = &ctx.label {
            if !label.is_empty() {
                out.action = crate::step::framework::StepAction::GotoLabel;
                out.goto_label = Some(label.clone());
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drop_player_context::DropPlayerContext;
    use crate::injury::{InjuryContext, InjuryResult};
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{ApothecaryMode, Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(PS_STANDING));
    }

    fn make_injury_result() -> Box<InjuryResult> {
        Box::new(InjuryResult::new(ApothecaryMode::Defender))
    }

    fn make_dpc(player_id: &str) -> Box<DropPlayerContext> {
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        Box::new(DropPlayerContext {
            injury_result: Some(Box::new(InjuryResult {
                injury_context: ctx,
                knocked_out: false,
                rip: false,
            })),
            player_id: Some(player_id.to_owned()),
            apothecary_mode: Some(ApothecaryMode::Defender),
            eligible_for_safe_pair_of_hands: true,
            ..DropPlayerContext::new()
        })
    }

    #[test]
    fn no_context_returns_next() {
        let mut game = make_game();
        let mut step = StepHandleDropPlayerContext::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_drop_player_context_accepted() {
        let mut step = StepHandleDropPlayerContext::new();
        let dpc = DropPlayerContext { injury_result: Some(make_injury_result()), ..DropPlayerContext::new() };
        assert!(step.set_parameter(&StepParameter::DropPlayerContext(Box::new(dpc))));
        assert!(step.drop_player_context.is_some());
    }

    #[test]
    fn publishes_injury_result() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        step.drop_player_context = Some(make_dpc("p1"));

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn drops_player_when_not_already_dropped() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        step.drop_player_context = Some(make_dpc("p1"));

        step.start(&mut game, &mut GameRng::new(0));

        // Player should now be prone
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), ffb_model::enums::PS_PRONE);
    }

    #[test]
    fn does_not_drop_when_already_dropped() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        dpc.already_dropped = true;
        step.drop_player_context = Some(dpc);

        step.start(&mut game, &mut GameRng::new(0));

        // Player should still be standing
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STANDING);
    }

    #[test]
    fn goto_label_when_set() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        dpc.label = Some("MY_LABEL".into());
        step.drop_player_context = Some(dpc);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("MY_LABEL"));
    }

    #[test]
    fn publishes_end_turn_when_flagged() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        dpc.end_turn = true;
        step.drop_player_context = Some(dpc);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn no_drop_when_armour_break_required_but_not_broken() {
        let mut game = make_game();
        add_player(&mut game, "p1");

        let mut step = StepHandleDropPlayerContext::new();
        let mut dpc = make_dpc("p1");
        // Override: requires armour break but armor_broken = false
        if let Some(ref mut ir) = dpc.injury_result {
            ir.injury_context.armor_broken = false;
        }
        dpc.requires_armour_break = true;
        step.drop_player_context = Some(dpc);

        step.start(&mut game, &mut GameRng::new(0));

        // Should NOT have dropped the player
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STANDING);
    }
}
