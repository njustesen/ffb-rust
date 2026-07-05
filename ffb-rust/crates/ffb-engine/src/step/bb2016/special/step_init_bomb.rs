/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.special.StepInitBomb`.
///
/// Initialization step of the bomb sequence (BB2016 Bombardier skill).
///
/// If catcher_id is set (bomb caught cleanly) → goto `goto_label_on_end`.
/// Otherwise (pass_fumble or bomb_out_of_bounds or dropped) → resolve bomb explosion:
/// - Clear range_ruler.
/// - Get bomb_coordinate from FieldModel; if null → ReportBombOutOfBounds + NEXT_STEP.
/// - Otherwise: clear bomb_coordinate, find adjacent players, push SpecialEffect sequences.
/// - Publish BOMB_EXPLODED(true).
/// - NEXT_STEP.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), CATCHER_ID (optional), PASS_FUMBLE (mandatory).
/// Optional init: BOMB_OUT_OF_BOUNDS (consumed).
/// Param: BOMB_OUT_OF_BOUNDS.
/// Publishes: CATCHER_ID, BOMB_EXPLODED.
///
/// client-only: BloodSpot / PS_HIT_BY_BOMB — field model visual decoration, client display only.
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};
use crate::step::generator::bb2016::special_effect::{SpecialEffect as SpecialEffectGenerator, SpecialEffectParams};

/// Java: `StepInitBomb` (bb2016/special).
pub struct StepInitBomb {
    /// Java: `fGotoLabelOnEnd` — mandatory init param.
    goto_label_on_end: String,
    /// Java: `fCatcherId` — optional init param.
    catcher_id: Option<String>,
    /// Java: `fPassFumble` — mandatory init param.
    pass_fumble: bool,
    /// Java: `fBombCoordinate` — computed at runtime.
    bomb_coordinate: Option<ffb_model::types::FieldCoordinate>,
    /// Java: `fBombOutOfBounds`
    bomb_out_of_bounds: bool,
}

impl StepInitBomb {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            catcher_id: None,
            pass_fumble: false,
            bomb_coordinate: None,
            bomb_out_of_bounds: false,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        game.field_model.range_ruler = None;
        // If fumbled or out-of-bounds: clear catcher.
        if self.pass_fumble || self.bomb_out_of_bounds {
            self.catcher_id = None;
        }
        if self.catcher_id.is_none() {
            // Bomb hit the ground — resolve explosion.
            self.bomb_coordinate = game.field_model.bomb_coordinate;
            let out = if self.bomb_coordinate.is_none() {
                // Bomb went out of bounds with no landing square.
                // Java: getResult().addReport(new ReportBombOutOfBounds())
                StepOutcome::next()
                    .with_event(GameEvent::BombOutOfBounds { coord: ffb_model::types::FieldCoordinate::new(0, 0) })
                    .publish(StepParameter::CatcherId(None))
            } else {
                // Clear bomb from field.
                let coord = self.bomb_coordinate.unwrap();
                game.field_model.bomb_coordinate = None;
                // client-only: BloodSpot(HIT_BY_BOMB) — field model visual decoration, client display only

                // Collect affected players: the bomb square + all 8 adjacent squares.
                let mut coords = vec![coord];
                coords.extend(game.field_model.adjacent_on_pitch(coord));
                let affected: Vec<String> = coords.iter()
                    .filter_map(|c| game.field_model.player_at(*c).cloned())
                    .collect();

                let mut out = StepOutcome::next()
                    .publish(StepParameter::BombExploded(true))
                    .publish(StepParameter::CatcherId(None));
                for player_id in affected {
                    let seq = SpecialEffectGenerator::build_sequence(&SpecialEffectParams {
                        special_effect: Some("BOMB".to_string()),
                        player_id: Some(player_id),
                        roll_for_effect: true,
                    });
                    out = out.push_seq(seq);
                }
                out
            };
            return out;
        }
        // Bomb caught → goto the pass-catch label.
        StepOutcome::goto(&self.goto_label_on_end)
            .publish(StepParameter::CatcherId(self.catcher_id.clone()))
    }
}

impl Default for StepInitBomb {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitBomb {
    fn id(&self) -> StepId { StepId::InitBomb }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s)   => { self.goto_label_on_end = s.clone(); true }
            StepParameter::CatcherId(v)        => { self.catcher_id = v.clone(); true }
            StepParameter::PassFumble(v)       => { self.pass_fumble = *v; true }
            StepParameter::BombOutOfBounds(v)  => { self.bomb_out_of_bounds = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_init_bomb() {
        assert_eq!(StepInitBomb::new().id(), StepId::InitBomb);
    }

    #[test]
    fn no_catcher_no_bomb_coord_returns_next() {
        let mut game = make_game();
        let mut step = StepInitBomb::new();
        step.goto_label_on_end = "catch".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // bomb_coordinate == None → out of bounds path → NEXT_STEP.
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn catcher_present_goto_label() {
        let mut game = make_game();
        let mut step = StepInitBomb::new();
        step.goto_label_on_end = "catch".into();
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::GotoLabel));
        assert_eq!(out.goto_label.as_deref(), Some("catch"));
    }

    #[test]
    fn pass_fumble_clears_catcher() {
        let mut game = make_game();
        let mut step = StepInitBomb::new();
        step.goto_label_on_end = "catch".into();
        step.catcher_id = Some("c1".into());
        step.pass_fumble = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Fumble clears catcher → NEXT_STEP, not GOTO.
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn bomb_coordinate_clears_and_publishes_exploded() {
        let mut game = make_game();
        game.field_model.bomb_coordinate = Some(FieldCoordinate { x: 10, y: 7 });
        let mut step = StepInitBomb::new();
        step.goto_label_on_end = "catch".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BombExploded(true))));
        assert!(game.field_model.bomb_coordinate.is_none());
    }

    #[test]
    fn set_parameter_goto_label_on_end() {
        let mut step = StepInitBomb::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("x".into())));
        assert_eq!(step.goto_label_on_end, "x");
    }

    #[test]
    fn bomb_on_occupied_square_pushes_special_effect_sequence() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use std::collections::HashSet;
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        game.field_model.bomb_coordinate = Some(coord);
        // Place a player at the bomb square.
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                    ..Default::default()
});
        game.field_model.set_player_coordinate("p1", coord);
        let mut step = StepInitBomb::new();
        step.goto_label_on_end = "catch".into();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty(), "should push SpecialEffect sequence for player at bomb square");
    }
}
