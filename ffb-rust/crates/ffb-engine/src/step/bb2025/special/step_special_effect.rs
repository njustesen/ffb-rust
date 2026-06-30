use ffb_model::model::special_effect::SpecialEffect;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};

/// Applies a special inducement effect (Wizard Lightning Bolt / Fire Ball / Bomb etc.).
// Java executeStep logic:
//   player = game.getPlayerById(fPlayerId)
//   if player == null -> return (implicit continue)
//
//   state = fieldModel.getPlayerState(player)
//   isStanding = !state.isProneOrStunned && !state.isStunned
//   isActive = state.isActive
//   successful = true
//
//   if fRollForEffect:
//     roll = diceRoller.rollWizardSpell()
//     successful = DiceInterpreter.isSpecialEffectSuccessful(fSpecialEffect, player, roll)
//     report ReportSpecialEffectRoll(fSpecialEffect, player.id, roll, successful)
//   else:
//     report ReportSpecialEffectRoll(fSpecialEffect, player.id, 0, true)
//
//   if successful:
//     playerCoordinate = fieldModel.getPlayerCoordinate(player)
//
//     if fSpecialEffect==ZAP && player instanceof RosterPlayer:
//       ... ZappedPlayer creation + team replacement ...
//       if ballCoordinate==playerCoordinate: push CSTI(SCATTER_BALL)
//
//     if fSpecialEffect==FIREBALL:
//       publish SteadyFootingContext(handleInjury(InjuryTypeFireball), [DropPlayerCommand])
//
//     if fSpecialEffect==BOMB:
//       bombFromHome/bombFromAway = (turnMode == BOMB_HOME / BOMB_AWAY / blitz variants)
//       playerHitIsFromBombTeam = ...
//       suppressEndTurn logic (BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER option)
//       if bombardier is self && !bomberTurnoverIgnored: suppressEndTurn=false
//       publish SteadyFootingContext(handleInjury(InjuryTypeBombWithModifier/ForSpp), [DropPlayerFromBombCommand])
//
//     // check end turn
//     if isStanding:
//       actingTeam = (based on turnMode BOMB_HOME/BOMB_AWAY or normal)
//       if actingTeam.hasPlayer && fSpecialEffect!=FIREBALL && !suppressEndTurn:
//         publish END_TURN=true
//     NEXT_STEP
//   else:
//     GOTO fGotoLabelOnFailure
//
// Unported utilities:
//   TODO: DiceInterpreter.isSpecialEffectSuccessful
//   TODO: ZappedPlayer creation / team.addPlayer / communication.sendZapPlayer
//   TODO: UtilServerInjury.handleInjury (InjuryTypeFireball / InjuryTypeBombWithModifier / ForSpp)
//   TODO: SteadyFootingContext / DeferredCommand (DropPlayerCommand, DropPlayerFromBombCommand)
//   TODO: game.turnData.passState.getOriginalBombardier
//   TODO: game.options (BOMB_TEAM_MATE_KNOCK_DOWN_CAUSES_TURNOVER, BOMBER_PLACED_PRONE_IGNORES_TURNOVER)
//   TODO: SpecialEffect.ZAP / FIREBALL / BOMB variants
//
// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.special.StepSpecialEffect`.
pub struct StepSpecialEffect {
    /// Java: fGotoLabelOnFailure (mandatory init param)
    pub goto_label_on_failure: String,
    /// Java: fPlayerId (mandatory init param)
    pub player_id: Option<String>,
    /// Java: fRollForEffect (mandatory init param)
    pub roll_for_effect: bool,
    /// Java: fSpecialEffect (mandatory init param)
    pub special_effect: Option<SpecialEffect>,
}

impl StepSpecialEffect {
    pub fn new(goto_label_on_failure: String) -> Self {
        Self {
            goto_label_on_failure,
            player_id: None,
            roll_for_effect: false,
            special_effect: None,
        }
    }
}

impl Default for StepSpecialEffect {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepSpecialEffect {
    fn id(&self) -> StepId { StepId::SpecialEffect }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepSpecialEffect {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Guard: player must exist.
        let player_exists = self.player_id.as_deref()
            .map(|id| game.player(id).is_some())
            .unwrap_or(false);
        if !player_exists {
            return StepOutcome::next();
        }

        // TODO: roll = diceRoller.rollWizardSpell() if roll_for_effect
        // TODO: successful = DiceInterpreter.isSpecialEffectSuccessful(special_effect, player, roll)
        // TODO: report ReportSpecialEffectRoll
        //
        // TODO: if successful:
        //   ZAP path: ZappedPlayer + scatter ball if on ball
        //   FIREBALL path: handleInjury(InjuryTypeFireball); publish SteadyFootingContext
        //   BOMB path: handleInjury(InjuryTypeBombWithModifier/ForSpp); publish SteadyFootingContext
        //              suppressEndTurn logic
        //   if isStanding && actingTeam has player && !FIREBALL && !suppressEndTurn:
        //     publish END_TURN=true
        //   NEXT_STEP
        // else:
        //   GOTO goto_label_on_failure
        //
        // Stub: no roll ported yet; always treat as successful (matches "no effect" neutral path).
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_no_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSpecialEffect::new("fail".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn new_stores_goto_label() {
        let s = StepSpecialEffect::new("failure_label".into());
        assert_eq!(s.goto_label_on_failure, "failure_label");
    }

    #[test]
    fn default_roll_for_effect_is_false() {
        let s = StepSpecialEffect::default();
        assert!(!s.roll_for_effect);
    }

    #[test]
    fn set_parameter_always_returns_false() {
        let mut step = StepSpecialEffect::default();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepSpecialEffect::new("fail".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
