/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.inducements.StepPlayCard`.
///
/// Step to play a card.
///
/// Needs to be initialized with stepParameter CARD (stored as card_id).
/// Needs to be initialized with stepParameter HOME_TEAM.
///
/// Handles CLIENT_PLAYER_CHOICE (card target selection) and CLIENT_SETUP_PLAYER
/// (illegal substitution flow) and CLIENT_END_TURN (end of illegal substitution).
///
/// Stub: UtilServerCards, UtilServerSetup, UtilServerDialog are not yet fully
/// translated — the parameter handling and command routing are ported; full card
/// activation logic is deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepPlayCard` (mixed/inducements, BB2016 + BB2020).
#[derive(Debug, Default)]
pub struct StepPlayCard {
    /// Java: `fCard` — init parameter (mandatory, stored as card_id string).
    card_id: Option<String>,
    /// Java: `fHomeTeam` — init parameter (mandatory).
    home_team: bool,
    /// Java: `fIllegalSubstitution`
    illegal_substitution: bool,
    /// Java: `fSetupPlayerId`
    setup_player_id: Option<String>,

    // Transient fields (not serialized in Java)
    /// Java: `fPlayerId` (transient)
    player_id: Option<String>,
    /// Java: `fOpponentId` (transient)
    opponent_id: Option<String>,
    /// Java: `fEndCardPlaying` (transient)
    end_card_playing: bool,
}

impl StepPlayCard {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, _game: &mut Game) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState()) — not ported

        if self.end_card_playing {
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)
            return StepOutcome::next();
        }

        if self.player_id.is_some() {
            // Java: playCardOnPlayer()
            return self.play_card_on_player();
        }

        // Java: else if (fCard.getTarget().isPlayedOnPlayer()) { show dialog }
        // Stub: Card target detection not yet ported → proceed to playCardOnTurn
        // Java: else { playCardOnTurn() }
        self.play_card_on_turn()
    }

    fn play_card_on_turn(&mut self) -> StepOutcome {
        // Java: boolean doNextStep = UtilServerCards.activateCard(this, fCard, fHomeTeam, null)
        // Stub: UtilServerCards not yet ported → return NEXT_STEP
        StepOutcome::next()
    }

    fn play_card_on_player(&mut self) -> StepOutcome {
        // Java: boolean doNextStep = UtilServerCards.activateCard / playCardWithBlockablePlayerSelection
        // Stub: card activation not yet ported → return NEXT_STEP
        StepOutcome::next()
    }
}

impl Step for StepPlayCard {
    fn id(&self) -> StepId { StepId::PlayCard }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: case CLIENT_PLAYER_CHOICE:
            //   if (PlayerChoiceMode.BLOCK == mode) fOpponentId = playerId
            //   else { fPlayerId = playerId; if (!provided(playerId)) fEndCardPlaying = true }
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
                    self.end_card_playing = true;
                } else {
                    self.player_id = Some(player_id.clone());
                }
            }
            // Java: case CLIENT_PLAYER_CHOICE with PlayerChoiceMode.BLOCK — opponent selection
            Action::Block { defender_id } => {
                self.opponent_id = Some(defender_id.clone());
            }
            // Java: case CLIENT_END_TURN with illegal substitution:
            //   fEndCardPlaying = true; process setup player if available
            Action::EndTurn => {
                if self.illegal_substitution {
                    self.end_card_playing = true;
                    self.setup_player_id = None;
                    self.illegal_substitution = false;
                }
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CardId(v) => { self.card_id = v.clone(); true }
            StepParameter::HomeTeam(v) => { self.home_team = *v; true }
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
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_play_card() {
        assert_eq!(StepPlayCard::new().id(), StepId::PlayCard);
    }

    #[test]
    fn start_returns_next_by_default() {
        let mut step = StepPlayCard::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_card_playing_flag_causes_next_step() {
        let mut step = StepPlayCard::new();
        step.end_card_playing = true;
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_card_id() {
        let mut step = StepPlayCard::new();
        step.set_parameter(&StepParameter::CardId(Some("my_card".into())));
        assert_eq!(step.card_id, Some("my_card".into()));
    }

    #[test]
    fn set_parameter_home_team() {
        let mut step = StepPlayCard::new();
        step.set_parameter(&StepParameter::HomeTeam(true));
        assert!(step.home_team);
    }

    #[test]
    fn handle_select_player_empty_id_ends_card_playing() {
        let mut step = StepPlayCard::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::SelectPlayer { player_id: "".into() },
            &mut game, &mut rng,
        );
        assert!(step.end_card_playing);
    }

    #[test]
    fn handle_select_player_stores_id() {
        let mut step = StepPlayCard::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::SelectPlayer { player_id: "p1".into() },
            &mut game, &mut rng,
        );
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn handle_end_turn_with_illegal_substitution_ends_card_playing() {
        let mut step = StepPlayCard::new();
        step.illegal_substitution = true;
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert!(step.end_card_playing);
        assert!(!step.illegal_substitution);
    }
}
