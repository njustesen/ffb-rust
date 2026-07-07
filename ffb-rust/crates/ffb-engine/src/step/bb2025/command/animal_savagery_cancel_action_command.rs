use ffb_model::enums::{PlayerAction, SkillId};
use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};

/// Cancels an action interrupted by Animal Savagery: marks the appropriate turn resource used
/// and clears the pass coordinate. Mirrors Java
/// `com.fumbbl.ffb.server.step.bb2025.command.AnimalSavageryCancelActionCommand`.
pub struct AnimalSavageryCancelActionCommand;

impl DeferredCommand for AnimalSavageryCancelActionCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::AnimalSavageryCancelAction }

    fn execute(&self, game: &mut Game) -> Vec<StepParameter> {
        if let Some(action) = game.acting_player.player_action {
            match action {
                PlayerAction::Blitz | PlayerAction::BlitzMove | PlayerAction::KickEmBlitz => {
                    game.turn_data_mut().blitz_used = true;
                }
                PlayerAction::KickTeamMate | PlayerAction::KickTeamMateMove => {
                    game.turn_data_mut().ktm_used = true;
                }
                PlayerAction::Pass | PlayerAction::PassMove => {
                    game.turn_data_mut().pass_used = true;
                }
                PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove => {
                    game.turn_data_mut().ttm_used = true;
                }
                PlayerAction::HandOver | PlayerAction::HandOverMove => {
                    game.turn_data_mut().hand_over_used = true;
                }
                PlayerAction::Foul | PlayerAction::FoulMove => {
                    // SneakiestOfTheLot (NamedProperties.allowsAdditionalFoul) lets a player
                    // foul again without consuming the team's foul action.
                    let has_additional_foul = {
                        let pid = game.acting_player.player_id.as_deref();
                        pid.and_then(|id| game.player(id))
                            .map(|p| p.has_skill(SkillId::SneakiestOfTheLot))
                            .unwrap_or(false)
                    };
                    if !has_additional_foul {
                        game.turn_data_mut().foul_used = true;
                    }
                }
                PlayerAction::SecureTheBall => {
                    game.turn_data_mut().secure_the_ball_used = true;
                }
                PlayerAction::Punt | PlayerAction::PuntMove => {
                    game.turn_data_mut().punt_used = true;
                }
                _ => {}
            }
        }
        game.pass_coordinate = None;
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PlayerAction};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn blitz_action_sets_blitz_used() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Blitz);
        let cmd = AnimalSavageryCancelActionCommand;
        cmd.execute(&mut game);
        assert!(game.turn_data().blitz_used);
    }

    #[test]
    fn pass_action_sets_pass_used() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let cmd = AnimalSavageryCancelActionCommand;
        cmd.execute(&mut game);
        assert!(game.turn_data().pass_used);
    }

    #[test]
    fn clears_pass_coordinate() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(5, 5));
        let cmd = AnimalSavageryCancelActionCommand;
        cmd.execute(&mut game);
        assert!(game.pass_coordinate.is_none());
    }

    #[test]
    fn returns_empty_params() {
        let mut game = make_game();
        let cmd = AnimalSavageryCancelActionCommand;
        let params = cmd.execute(&mut game);
        assert!(params.is_empty());
    }
    #[test]
    fn is_zero_sized_unit_struct() {
        assert_eq!(std::mem::size_of::<AnimalSavageryCancelActionCommand>(), 0);
    }
}
