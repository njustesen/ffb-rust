use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, VictimStateKey};
use crate::step::framework::{Step, StepAction, StepOutcome, StepParameter};
use crate::step::framework::StepId;
use crate::step::util_server_injury::drop_player;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.StepHandleDropPlayerContext.
/// Identical in logic to the BB2025 version — see bb2025/shared/step_handle_drop_player_context.rs.
pub struct StepHandleDropPlayerContext {
    pub drop_player_context: Option<Box<DropPlayerContext>>,
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepHandleDropPlayerContext {
    pub fn new() -> Self {
        Self { drop_player_context: None, re_rolled_action: None, re_roll_source: None }
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
        if let StepParameter::DropPlayerContext(ctx) = param {
            self.drop_player_context = Some(ctx.clone());
            return true;
        }
        false
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

        let mut out = StepOutcome::next();

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
            if ctx.end_turn {
                out = out.publish(StepParameter::EndTurn(true));
            }
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
            out = out.publish(StepParameter::EndTurn(true));
        }

        out = out.publish(StepParameter::InjuryResult(injury_result.clone()));

        if let Some(label) = &ctx.label {
            if !label.is_empty() {
                out.action = StepAction::GotoLabel;
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
        Game::new(home, away, Rules::Bb2020)
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

    fn make_dpc(player_id: &str) -> Box<DropPlayerContext> {
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        Box::new(DropPlayerContext {
            injury_result: Some(Box::new(InjuryResult {
                injury_context: ctx, knocked_out: false, rip: false,
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
    fn publishes_injury_result() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        let mut step = StepHandleDropPlayerContext::new();
        step.drop_player_context = Some(make_dpc("p1"));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn drops_player() {
        let mut game = make_game();
        add_player(&mut game, "p1");
        let mut step = StepHandleDropPlayerContext::new();
        step.drop_player_context = Some(make_dpc("p1"));
        step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), ffb_model::enums::PS_PRONE);
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
    fn set_parameter_drop_player_context_accepted() {
        let mut step = StepHandleDropPlayerContext::new();
        let dpc = DropPlayerContext { injury_result: Some(Box::new(InjuryResult::new(ApothecaryMode::Defender))), ..DropPlayerContext::new() };
        assert!(step.set_parameter(&StepParameter::DropPlayerContext(Box::new(dpc))));
        assert!(step.drop_player_context.is_some());
    }
}
