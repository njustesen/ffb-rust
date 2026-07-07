/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.ttm.TtmToCrowdHandler`.
///
/// Helper class that handles throwing a TTM player out of bounds into the crowd.
/// Called from Swoop steps when the thrown player lands out of bounds.
///
/// Java logic (`handle`):
///   1. Set player state to FALLING.
///   2. UtilServerInjury.handleInjury → produces InjuryResult.
///   3. Publish INJURY_RESULT.
///   4. If hasBall: publish CATCH_SCATTER_THROW_IN_MODE=THROW_IN, THROW_IN_COORDINATE, END_TURN.
///   5. Publish THROWN_PLAYER_COORDINATE = null  (end loop sentinel).
use ffb_model::enums::{ApothecaryMode, PS_FALLING, PlayerState};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryTypeServer;
use crate::step::util_server_injury;
use crate::step::framework::{CatchScatterThrowInMode, StepParameter};

/// Java: `TtmToCrowdHandler` (mixed/ttm).
///
/// Not a `Step` — used as a helper called from Swoop steps.
pub struct TtmToCrowdHandler;

impl TtmToCrowdHandler {
    pub fn new() -> Self { Self }

    /// Java: `handle(game, step, thrownPlayer, endCoordinate, hasBall, injuryTypeServer)`
    ///
    /// Publishes parameters to the step stack representing the crowd-injury outcome.
    /// Returns the list of parameters to publish (mirrors Java's `step.publishParameter` calls).
    ///
    /// `player_id` — the thrown player's ID.
    /// `end_coordinate` — the out-of-bounds coordinate where the player lands.
    /// `has_ball` — whether the thrown player was carrying the ball.
    /// `injury_type` — the injury type to apply (caller determines TrapDoor/TTMLanding/etc).
    pub fn handle(
        game: &mut Game,
        rng: &mut GameRng,
        player_id: &str,
        end_coordinate: FieldCoordinate,
        has_ball: bool,
        injury_type: &mut dyn InjuryTypeServer,
    ) -> Vec<StepParameter> {
        // Java: game.getFieldModel().setPlayerState(thrownPlayer, new PlayerState(FALLING))
        game.field_model.set_player_state(player_id, PlayerState::new(PS_FALLING));

        // Java: InjuryResult injuryResult = UtilServerInjury.handleInjury(step, injuryTypeServer, null, thrownPlayer, endCoordinate, null, null, ApothecaryMode.THROWN_PLAYER)
        let injury_result = util_server_injury::handle_injury(
            game, rng, injury_type,
            None, player_id, end_coordinate, None, None,
            ApothecaryMode::ThrownPlayer,
        );
        injury_result.apply_to(game);

        let mut params: Vec<StepParameter> = vec![
            // publish INJURY_RESULT
            StepParameter::InjuryResult(Box::new(injury_result.clone())),
        ];

        if has_ball {
            // Java: publish CATCH_SCATTER_THROW_IN_MODE = THROW_IN
            params.push(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ThrowIn));
            // Java: publish THROW_IN_COORDINATE
            params.push(StepParameter::ThrowInCoordinate(end_coordinate));
            // Java: publish END_TURN = true
            params.push(StepParameter::EndTurn(true));
        }

        // Java: publish THROWN_PLAYER_COORDINATE = null  (ends the swoop loop)
        params.push(StepParameter::ThrownPlayerCoordinate(None));

        params
    }
}

impl Default for TtmToCrowdHandler {
    fn default() -> Self { Self::new() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PS_STANDING};

    fn make_game() -> Game {
        use ffb_model::model::game::Game;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_to_field(game: &mut Game, player_id: &str) {
        use ffb_model::enums::{PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        game.team_home.players.push(Player {
            id: player_id.into(), name: player_id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
});
        game.field_model.set_player_coordinate(player_id, FieldCoordinate::new(5, 5));
        game.field_model.set_player_state(player_id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn handle_applies_injury_and_changes_state_from_standing() {
        use ffb_model::util::rng::GameRng;
        use crate::injury::injuryType::injury_type_ttm_landing::InjuryTypeTTMLanding;
        let mut game = make_game();
        add_player_to_field(&mut game, "p1");
        let coord = FieldCoordinate::new(0, 5);
        let mut rng = GameRng::new(0);
        let mut inj = InjuryTypeTTMLanding::new();
        TtmToCrowdHandler::handle(&mut game, &mut rng, "p1", coord, false, &mut inj);
        // Player state changed from STANDING by injury application.
        let state = game.field_model.player_state("p1").unwrap();
        assert_ne!(state.base(), PS_STANDING, "player state must have changed from STANDING after injury");
    }

    #[test]
    fn without_ball_publishes_injury_and_null_coordinate() {
        use ffb_model::util::rng::GameRng;
        use crate::injury::injuryType::injury_type_ttm_landing::InjuryTypeTTMLanding;
        let mut game = make_game();
        add_player_to_field(&mut game, "p1");
        let coord = FieldCoordinate::new(0, 5);
        let mut rng = GameRng::new(0);
        let mut inj = InjuryTypeTTMLanding::new();
        let params = TtmToCrowdHandler::handle(&mut game, &mut rng, "p1", coord, false, &mut inj);
        // Should contain: INJURY_RESULT, THROWN_PLAYER_COORDINATE(None)
        assert_eq!(params.len(), 2);
        assert!(matches!(params[0], StepParameter::InjuryResult(_)));
        assert!(matches!(params[1], StepParameter::ThrownPlayerCoordinate(None)));
    }

    #[test]
    fn with_ball_publishes_throw_in_and_end_turn() {
        use ffb_model::util::rng::GameRng;
        use crate::injury::injuryType::injury_type_ttm_landing::InjuryTypeTTMLanding;
        let mut game = make_game();
        add_player_to_field(&mut game, "p1");
        let coord = FieldCoordinate::new(0, 5);
        let mut rng = GameRng::new(0);
        let mut inj = InjuryTypeTTMLanding::new();
        let params = TtmToCrowdHandler::handle(&mut game, &mut rng, "p1", coord, true, &mut inj);
        // Should contain: INJURY_RESULT, CATCH_SCATTER_THROW_IN_MODE, THROW_IN_COORDINATE,
        //                 END_TURN, THROWN_PLAYER_COORDINATE(None)
        assert_eq!(params.len(), 5);
        assert!(matches!(params[0], StepParameter::InjuryResult(_)));
        assert!(matches!(params[1], StepParameter::CatchScatterThrowInMode(_)));
        assert!(matches!(params[2], StepParameter::ThrowInCoordinate(_)));
        assert!(matches!(params[3], StepParameter::EndTurn(true)));
        assert!(matches!(params[4], StepParameter::ThrownPlayerCoordinate(None)));
    }

    #[test]
    fn new_constructs() {
        let _ = TtmToCrowdHandler::new();
        let _ = TtmToCrowdHandler::default();
    }
    #[test]
    fn new_and_default_create_equivalent_instances() {
        let _a = TtmToCrowdHandler::new();
        let _b = TtmToCrowdHandler::default();
    }
}
