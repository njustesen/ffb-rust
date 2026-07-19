use ffb_model::enums::PlayerAction;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Initializes the punt action: waits for a target coordinate or end-turn, then validates
/// that the acting player has an unused punt skill and sets up scatter parameters.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.punt.StepInitPunt`.
pub struct StepInitPunt {
    pub goto_label_on_end: String,
    pub coordinate_to: Option<FieldCoordinate>,
    pub end_turn: bool,
    pub end_player_action: bool,
    pub punt_to_crowd: Option<bool>,
}

impl StepInitPunt {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            coordinate_to: None,
            end_turn: false,
            end_player_action: false,
            punt_to_crowd: None,
        }
    }
}

impl Step for StepInitPunt {
    fn id(&self) -> StepId { StepId::InitPunt }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => { self.end_turn = true; }
            Action::Punt { coord } => { self.coordinate_to = Some(*coord); }
            _ => {}
        }
        self.execute_step(game, rng)
    }
}

impl StepInitPunt {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_end.clone();

        if self.end_turn {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CheckForgo(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        let player_action = game.acting_player.player_action;

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::goto(&label),
        };

        // Java: UtilCards.getUnusedSkillWithProperty(actingPlayer, NamedProperties.canPunt)
        let has_punt_skill = game
            .team_home
            .player(&player_id)
            .or_else(|| game.team_away.player(&player_id))
            .map(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_PUNT).is_some())
            .unwrap_or(false);

        // Java: actingPlayer.getPlayerAction() == PlayerAction.PUNT && skill != null
        if player_action != Some(PlayerAction::Punt) || !has_punt_skill {
            return StepOutcome::goto(&label);
        }

        let player_coord = match game.field_model.player_coordinate(&player_id) {
            Some(c) => c,
            None => return StepOutcome::goto(&label),
        };

        // Determine punt-to-crowd: auto-false when not on a sideline or endzone.
        // Java: !SIDELINE_LOWER.isInBounds && !SIDELINE_UPPER.isInBounds && !ENDZONE_AWAY && !ENDZONE_HOME.
        // FieldCoordinate bounds: x in [0,25], y in [0,14]; sidelines are y==0 or y==14; endzones x==0 or x==25.
        if self.punt_to_crowd.is_none() {
            let on_boundary = player_coord.y == 0 || player_coord.y == 14
                || player_coord.x == 0 || player_coord.x == 25;
            if !on_boundary {
                self.punt_to_crowd = Some(false);
            }
        }

        if self.punt_to_crowd == Some(true) {
            return StepOutcome::next()
                .publish(StepParameter::Touchback(true));
        }

        if let Some(coord) = self.coordinate_to {
            let from = player_coord;
            game.field_model.clear_move_squares();
            return StepOutcome::next()
                .publish(StepParameter::CoordinateTo(coord))
                .publish(StepParameter::CoordinateFrom(from));
        }

        // Wait for coordinate selection — TODO: show move squares (requires MoveSquare model).
        StepOutcome::cont()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PlayerAction};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // 1. EndTurn command → GotoLabel + EndTurn + CheckForgo published
    #[test]
    fn end_turn_goto_label() {
        let mut game = make_game();
        let mut step = StepInitPunt::new("endLabel".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("endLabel"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    // 2. PlayerAction != Punt → GotoLabel (skip)
    #[test]
    fn wrong_player_action_goto_end() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Move);
        let mut step = StepInitPunt::new("endLabel".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    // 3. Punt action with player off-boundary → puntToCrowd auto-false, Continue waiting for coord
    #[test]
    fn punt_action_interior_waits_for_coord() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;

        let mut game = make_game();
        // Place a player on an interior square (not on boundary)
        let player = Player {
            id: "punter".into(), name: "punter".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue::new(ffb_model::enums::SkillId::Punt)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        let coord = FieldCoordinate::new(12, 7); // interior
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("punter", coord);
        game.acting_player.player_id = Some("punter".into());
        game.acting_player.player_action = Some(PlayerAction::Punt);

        let mut step = StepInitPunt::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // punt_to_crowd auto-set to false; no coord provided → Continue
        assert_eq!(out.action, StepAction::Continue);
    }

    // 4. Punt with punt_to_crowd = true → NextStep + Touchback published
    #[test]
    fn punt_to_crowd_publishes_touchback() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;

        let mut game = make_game();
        let player = Player {
            id: "punter2".into(), name: "p".into(), nr: 2,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue::new(ffb_model::enums::SkillId::Punt)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("punter2", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("punter2".into());
        game.acting_player.player_action = Some(PlayerAction::Punt);

        let mut step = StepInitPunt::new("end".into());
        step.punt_to_crowd = Some(true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::Touchback(true))));
    }

    // 5. Punt coord provided → NextStep + CoordinateTo + CoordinateFrom published
    #[test]
    fn with_coord_publishes_coordinate_to_and_from() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;

        let mut game = make_game();
        let player = Player {
            id: "punter3".into(), name: "p".into(), nr: 3,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue::new(ffb_model::enums::SkillId::Punt)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        let from = FieldCoordinate::new(8, 7);
        let to = FieldCoordinate::new(9, 7);
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("punter3", from);
        game.acting_player.player_id = Some("punter3".into());
        game.acting_player.player_action = Some(PlayerAction::Punt);

        let mut step = StepInitPunt::new("end".into());
        step.punt_to_crowd = Some(false);
        step.coordinate_to = Some(to);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    // 6. Java: `actingPlayer.getPlayerAction() == PlayerAction.PUNT && skill != null` — a player
    // with PlayerAction::Punt but WITHOUT the Punt skill (canPunt property) must fall through
    // to the goto-label branch, not be allowed to proceed with the punt.
    #[test]
    fn punt_action_without_punt_skill_goes_to_label() {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;

        let mut game = make_game();
        let player = Player {
            id: "punter_noskill".into(), name: "p".into(), nr: 4,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("punter_noskill", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("punter_noskill".into());
        game.acting_player.player_action = Some(PlayerAction::Punt);

        let mut step = StepInitPunt::new("endLabel".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("endLabel"));
    }
}
