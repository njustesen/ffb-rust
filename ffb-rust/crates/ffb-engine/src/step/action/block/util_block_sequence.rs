/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.UtilBlockSequence (COMMON).
///
/// Utility that initialises a pushback sequence: clears pushback squares, finds the starting
/// pushback square, and handles the Strip Ball skill (forceOpponentToDropBallOnPushback).
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_cards::UtilCards;
use crate::step::framework::StepParameter;
use crate::util::util_server_pushback::UtilServerPushback;

/// Java: UtilBlockSequence.initPushback(step) — initialises pushback parameters.
///
/// Returns a list of StepParameter values that should be published: always includes
/// STARTING_PUSHBACK_SQUARE (with real direction from attacker→defender geometry);
/// may include CATCH_SCATTER_THROW_IN_MODE and BALL_KNOCKED_LOSE when StripBall fires.
pub fn init_pushback(game: &mut Game) -> Vec<StepParameter> {
    let mut params = Vec::new();

    // Java: game.getFieldModel().clearPushbackSquares()
    game.field_model.pushback_squares.clear();

    // Java: UtilServerPushback.findStartingSquare(attackerCoord, defenderCoord, isHomePlaying)
    let attacker_coord = game.acting_player.player_id.as_deref()
        .and_then(|id| game.field_model.player_coordinate(id));
    let defender_coord = game.defender_id.as_deref()
        .and_then(|id| game.field_model.player_coordinate(id));

    if let (Some(ac), Some(dc)) = (attacker_coord, defender_coord) {
        let starting_sq = UtilServerPushback::find_starting_square(ac, dc, game.home_playing);
        params.push(StepParameter::StartingPushbackSquare(starting_sq));
    }

    // Java: Strip Ball. skillCanForceOpponentToDropBall =
    //   attacker.getSkillWithProperty(forceOpponentToDropBallOnPushback)
    // If the attacker has Strip Ball AND the defender is an opposing ball CARRIER
    // (defenderCoordinate.equals(ballCoordinate)), the ball is knocked loose and scatters
    // (SCATTER_BALL + BALL_KNOCKED_LOSE + STEAL_BALL report) — UNLESS the defender has a
    // cancelling skill (Sure Hands / Monstrous Mouth) AND still has tacklezones (bb2025
    // SkillMechanic.canPreventStripBall(state) == state.hasTacklezones()). This path fires from
    // BOTH StepBlockChoice (POW/PUSHBACK/tackled-dodge) and StepBlockDodge (a stumble the defender
    // dodges but is still pushed) — skaven seed 52 i=147: a Blitzer (Strip Ball) stumbles a Gutter
    // Runner (Dodge) carrier; the DODGE_BLOCK path still strips the ball.
    let attacker_id = game.acting_player.player_id.clone();
    let defender_id = game.defender_id.clone();
    let strip_skill = attacker_id.as_deref()
        .and_then(|id| game.player(id))
        .and_then(|p| p.all_skill_ids()
            .find(|s| s.properties().contains(&NamedProperties::FORCE_OPPONENT_TO_DROP_BALL_ON_PUSHBACK)));
    let defender_is_carrier = defender_coord.is_some()
        && defender_coord == game.field_model.ball_coordinate;
    let opponents = match (attacker_id.as_deref(), defender_id.as_deref()) {
        (Some(a), Some(d)) => game.player_team_id(a) != game.player_team_id(d),
        _ => false,
    };

    if let (Some(strip), true, true) = (strip_skill, defender_is_carrier, opponents) {
        use ffb_model::model::skill_use::SkillUse;
        use ffb_model::report::report_skill_use::ReportSkillUse;
        use ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode;
        // Java: skillCanCounterOpponentForcingDropBall = UtilCards.getSkillCancelling(defender, strip)
        let cancel = defender_id.as_deref()
            .and_then(|id| game.player(id))
            .and_then(|p| UtilCards::get_skill_cancelling_property(p, NamedProperties::FORCE_OPPONENT_TO_DROP_BALL_ON_PUSHBACK));
        // bb2025 SkillMechanic.canPreventStripBall(state) == state.hasTacklezones()
        let can_prevent = defender_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .map(|s| s.has_tacklezones())
            .unwrap_or(false);
        if let (Some(cancel_skill), true) = (cancel, can_prevent) {
            if let Some(did) = defender_id.clone() {
                game.report_list.add(ReportSkillUse::new(Some(did), cancel_skill, true, SkillUse::CANCEL_STRIP_BALL));
            }
        } else {
            if let Some(aid) = attacker_id.clone() {
                game.report_list.add(ReportSkillUse::new(Some(aid), strip, true, SkillUse::STEAL_BALL));
            }
            params.push(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));
            params.push(StepParameter::BallKnockedLoose(true));
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Direction, Rules};
    use ffb_model::types::{FieldCoordinate, PushbackSquare};

    #[test]
    fn init_pushback_clears_pushback_squares() {
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        game.field_model.pushback_squares.push(PushbackSquare::new(FieldCoordinate::new(1, 1), Direction::North, false));
        init_pushback(&mut game);
        assert!(game.field_model.pushback_squares.is_empty());
    }

    #[test]
    fn init_pushback_returns_starting_square_at_defender_position_with_direction() {
        use ffb_model::types::PushbackSquare;
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        game.home_playing = true;
        // Attacker north of defender → direction = South
        game.acting_player.player_id = Some("att".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(8, 3));
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(8, 4));
        let params = init_pushback(&mut game);
        // Starting square is at defender's coordinate, direction South (attacker is north)
        assert!(params.iter().any(|p| matches!(p, StepParameter::StartingPushbackSquare(Some(sq))
            if sq.coordinate == FieldCoordinate::new(8, 4) && sq.direction == Direction::South)));
    }

    // Strip Ball (skaven seed 52 i=147): a Strip Ball attacker pushing an opposing ball carrier
    // knocks the ball loose. This path is shared by StepBlockChoice AND StepBlockDodge (the
    // dodged-stumble push), so it lives here in the common util.
    fn strip_setup(defender_skills: Vec<ffb_model::model::skill_def::SkillWithValue>, def_standing: bool) -> Game {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::{SkillId, PS_STANDING, PS_FALLING, PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        use ffb_model::enums::PlayerState;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut att = Player { id: "att".into(), name: "att".into(), nr: 2, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9, ..Default::default() };
        att.starting_skills = vec![SkillWithValue::new(SkillId::StripBall)];
        game.team_away.players.push(att);
        let mut def = Player { id: "def".into(), name: "def".into(), nr: 3, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9, ..Default::default() };
        def.starting_skills = defender_skills;
        game.team_home.players.push(def);
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(6, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING));
        game.field_model.set_player_state("def", PlayerState::new(if def_standing { PS_STANDING } else { PS_FALLING }));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5)); // def carries the ball
        game.field_model.ball_in_play = true;
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game
    }

    #[test]
    fn strip_ball_pushing_carrier_publishes_scatter_and_knocked_loose() {
        use ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode;
        let mut game = strip_setup(vec![], true);
        let params = init_pushback(&mut game);
        assert!(params.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
        assert!(params.iter().any(|p| matches!(p, StepParameter::BallKnockedLoose(true))));
    }

    #[test]
    fn strip_ball_cancelled_by_sure_hands_with_tacklezones() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        use ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode;
        let mut game = strip_setup(vec![SkillWithValue::new(SkillId::SureHands)], true);
        let params = init_pushback(&mut game);
        assert!(!params.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))),
            "Sure Hands + tacklezones cancels Strip Ball");
    }

    #[test]
    fn strip_ball_not_cancelled_when_carrier_prone() {
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::enums::SkillId;
        use ffb_model::model::catch_scatter_throw_in_mode::CatchScatterThrowInMode;
        // bb2025 canPreventStripBall == hasTacklezones(): a prone Sure Hands carrier can't prevent.
        let mut game = strip_setup(vec![SkillWithValue::new(SkillId::SureHands)], false);
        let params = init_pushback(&mut game);
        assert!(params.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
    }

    #[test]
    fn init_pushback_no_defender_no_starting_square() {
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        let params = init_pushback(&mut game);
        assert!(params.iter().all(|p| !matches!(p, StepParameter::StartingPushbackSquare(_))));
    }

    #[test]
    fn init_pushback_no_attacker_no_starting_square() {
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(8, 4));
        // No acting player set
        let params = init_pushback(&mut game);
        assert!(params.iter().all(|p| !matches!(p, StepParameter::StartingPushbackSquare(_))));
    }
    #[test]
    fn init_pushback_with_empty_board_clears_squares() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        assert!(game.field_model.pushback_squares.is_empty());
        init_pushback(&mut game);
        assert!(game.field_model.pushback_squares.is_empty());
    }
}
