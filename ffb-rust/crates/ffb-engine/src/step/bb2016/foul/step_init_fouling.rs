/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.foul.StepInitFouling`.
///
/// Initialises a foul sequence:
/// - `Action::EndTurn` → publish EndTurn, goto end label.
/// - `Action::Foul { target_id }` → record the foul defender, then execute.
/// - Otherwise: if acting player hasn't fouled and target is valid, register the foul.
///
/// Init parameter: `GOTO_LABEL_ON_END` (mandatory), `FOUL_DEFENDER_ID` (optional).
use ffb_model::model::game::Game;
use ffb_model::model::game_result::PlayerResult;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitFouling` (bb2016/foul).
pub struct StepInitFouling {
    /// Java: `fGotoLabelOnEnd` — init param (mandatory).
    goto_label_on_end: String,
    /// Java: `fFoulDefenderId` — the chosen foul target.
    foul_defender_id: Option<String>,
    /// Java: `fEndTurn` — true when the player chose to end their turn.
    end_turn: bool,
    /// Java: `fEndPlayerAction` — true when player deselected without ending turn.
    end_player_action: bool,
}

impl StepInitFouling {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            foul_defender_id: None,
            end_turn: false,
            end_player_action: false,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        if self.end_turn {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndPlayerAction(true));
        }

        // Validate conditions before registering the foul
        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };
        let foul_defender_id = match &self.foul_defender_id {
            Some(id) => id.clone(),
            None => return StepOutcome::next(),
        };

        let attacker_exists = game.player(&attacker_id).is_some();
        let has_fouled = game.acting_player.has_fouled;
        let defender_exists = game.player(&foul_defender_id).is_some();
        // Java: !foulDefender.hasSkillProperty(NamedProperties.preventBeingFouled)
        let defender_unfoulable = game.player(&foul_defender_id)
            .map(|p| p.has_skill_property(NamedProperties::PREVENT_BEING_FOULED))
            .unwrap_or(false);

        if attacker_exists && !has_fouled && defender_exists && !defender_unfoulable {
            game.defender_id = Some(foul_defender_id);
            game.acting_player.has_fouled = true;
            game.turn_data_mut().turn_started = true;
            game.concession_possible = false;

            // increment foul count on acting player's result
            let player_results = if game.home_playing {
                &mut game.game_result.home.player_results
            } else {
                &mut game.game_result.away.player_results
            };
            let pr = player_results.entry(attacker_id).or_insert_with(PlayerResult::default);
            pr.fouls += 1;
            game.turn_data_mut().foul_used = true;

            StepOutcome::next()
        } else {
            StepOutcome::next()
        }
    }
}

impl Default for StepInitFouling {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitFouling {
    fn id(&self) -> StepId { StepId::InitFouling }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                self.end_turn = true;
                self.execute_step(game)
            }
            Action::Foul { target_id } => {
                self.foul_defender_id = Some(target_id.clone());
                self.execute_step(game)
            }
            _ => StepOutcome::next(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(s)    => { self.goto_label_on_end = s.clone(); true }
            StepParameter::FoulDefenderId(s)    => { self.foul_defender_id = Some(s.clone()); true }
            StepParameter::EndTurn(v)           => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)   => { self.end_player_action = *v; true }
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
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
    }

    #[test]
    fn id_is_init_fouling() {
        assert_eq!(StepInitFouling::new().id(), StepId::InitFouling);
    }

    #[test]
    fn end_turn_publishes_and_gotos_label() {
        let mut step = StepInitFouling::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::EndTurn(true));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn end_player_action_publishes_and_gotos_label() {
        let mut step = StepInitFouling::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::EndPlayerAction(true));
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::GotoLabel));
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn valid_foul_registers_all_fields() {
        let mut step = StepInitFouling::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        step.set_parameter(&StepParameter::FoulDefenderId("def".into()));
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, "att");
        add_player(&mut game, "def");
        game.acting_player.player_id = Some("att".into());
        let mut rng = GameRng::new(0);
        let outcome = step.start(&mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.defender_id.as_deref(), Some("def"));
        assert!(game.acting_player.has_fouled);
        assert!(game.turn_data().turn_started);
        assert!(!game.concession_possible);
        assert!(game.turn_data().foul_used);
        assert_eq!(game.game_result.home.player_results.get("att").map(|p| p.fouls), Some(1));
    }

    #[test]
    fn already_fouled_skips_registration() {
        let mut step = StepInitFouling::new();
        step.set_parameter(&StepParameter::FoulDefenderId("def".into()));
        let mut game = make_game();
        add_player(&mut game, "att");
        add_player(&mut game, "def");
        game.acting_player.player_id = Some("att".into());
        game.acting_player.has_fouled = true;
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn foul_action_sets_defender_and_executes() {
        let mut step = StepInitFouling::new();
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end".into()));
        let mut game = make_game();
        game.home_playing = true;
        add_player(&mut game, "att");
        add_player(&mut game, "def");
        game.acting_player.player_id = Some("att".into());
        let mut rng = GameRng::new(0);
        let outcome = step.handle_command(&Action::Foul { target_id: "def".into() }, &mut game, &mut rng);
        assert!(matches!(outcome.action, StepAction::NextStep));
        assert_eq!(game.defender_id.as_deref(), Some("def"));
    }
}
