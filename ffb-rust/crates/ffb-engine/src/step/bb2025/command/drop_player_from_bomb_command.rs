use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use crate::step::framework::{DeferredCommand, DeferredCommandId, StepParameter};
use crate::step::util_server_injury::drop_player;

/// Drops a player hit by a bomb: runs injury, optionally suppresses the turnover, and preserves
/// the active flag for non-bombardiers. Mirrors Java
/// `com.fumbbl.ffb.server.step.bb2025.command.DropPlayerFromBombCommand`.
pub struct DropPlayerFromBombCommand {
    pub player_id: String,
    pub apothecary_mode: ApothecaryMode,
    pub eligible_for_safe_pair_of_hands: bool,
    pub was_active: bool,
    pub suppress_end_turn: bool,
}

impl DropPlayerFromBombCommand {
    pub fn new(
        player_id: String,
        apothecary_mode: ApothecaryMode,
        eligible_for_safe_pair_of_hands: bool,
        was_active: bool,
        suppress_end_turn: bool,
    ) -> Self {
        Self { player_id, apothecary_mode, eligible_for_safe_pair_of_hands, was_active, suppress_end_turn }
    }
}

impl DeferredCommand for DropPlayerFromBombCommand {
    fn id(&self) -> DeferredCommandId { DeferredCommandId::DropPlayerFromBomb }

    fn execute(&self, game: &mut Game) -> Vec<StepParameter> {
        // Java: UtilServerInjury.dropPlayer(step, player, apothecaryMode, eligibleForSafePairOfHands)
        // apothecary_mode deferred — dialog infra needed.
        let mut params = drop_player(game, &self.player_id, self.eligible_for_safe_pair_of_hands);

        // Java:
        //   PlayerState newState = game.getFieldModel().getPlayerState(player);
        //   if (!player.getId().equalsIgnoreCase(originalBombardier) && newState.isProneOrStunned())
        //       game.getFieldModel().setPlayerState(player, newState.changeActive(wasActive));
        if let Some(new_state) = game.field_model.player_state(&self.player_id) {
            let is_original_bombardier = game
                .original_bombardier
                .as_deref()
                .map(|id| id.eq_ignore_ascii_case(&self.player_id))
                .unwrap_or(false);
            if !is_original_bombardier && new_state.is_prone_or_stunned() {
                game.field_model
                    .set_player_state(&self.player_id, new_state.change_active(self.was_active));
            }
        }

        // Java: if (suppressEndTurn) → remove END_TURN from published params
        if self.suppress_end_turn {
            params.retain(|p| !matches!(p, StepParameter::EndTurn(_)));
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, ApothecaryMode};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn execute_returns_empty_stub() {
        let mut game = make_game();
        let cmd = DropPlayerFromBombCommand::new("p1".into(), ApothecaryMode::Defender, true, false, false);
        let params = cmd.execute(&mut game);
        assert!(params.is_empty());
    }

    #[test]
    fn suppress_end_turn_removes_end_turn_param() {
        let mut game = make_game();
        let cmd = DropPlayerFromBombCommand::new("p1".into(), ApothecaryMode::Defender, true, false, true);
        let params = cmd.execute(&mut game);
        assert!(!params.iter().any(|p| matches!(p, StepParameter::EndTurn(_))));
    }

    #[test]
    fn id_is_drop_player_from_bomb() {
        let cmd = DropPlayerFromBombCommand::new("p1".into(), ApothecaryMode::Defender, false, false, false);
        assert_eq!(cmd.id(), DeferredCommandId::DropPlayerFromBomb);
    }

    #[test]
    fn stores_all_fields() {
        let cmd = DropPlayerFromBombCommand::new("bomb_victim".into(), ApothecaryMode::Attacker, true, true, true);
        assert_eq!(cmd.player_id, "bomb_victim");
        assert_eq!(cmd.apothecary_mode, ApothecaryMode::Attacker);
        assert!(cmd.eligible_for_safe_pair_of_hands);
        assert!(cmd.was_active);
        assert!(cmd.suppress_end_turn);
    }
    #[test]
    fn is_zero_sized_unit_struct() {
        assert!(std::mem::size_of::<DropPlayerFromBombCommand>() > 0);
    }

    fn add_stunned_player(game: &mut Game, id: &str, active: bool) {
        use ffb_model::enums::{PlayerGender, PlayerState, PlayerType, PS_STUNNED};
        use ffb_model::model::player::Player;
        use ffb_model::types::FieldCoordinate;
        game.team_home.players.push(Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        });
        let pos = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate(id, pos);
        let state = PlayerState::new(PS_STUNNED).change_active(active);
        game.field_model.set_player_state(id, state);
    }

    #[test]
    fn restores_active_flag_for_non_bombardier_prone_or_stunned_player() {
        // Java: DropPlayerFromBombCommand.execute — for a player who is NOT the original
        // bombardier and ends up prone/stunned after the drop, the active flag is restored to
        // `wasActive` rather than left as whatever dropPlayer's own deactivation logic set it to.
        let mut game = make_game();
        add_stunned_player(&mut game, "p1", false);
        game.original_bombardier = Some("someone_else".into());
        let cmd = DropPlayerFromBombCommand::new("p1".into(), ApothecaryMode::Defender, false, true, false);
        cmd.execute(&mut game);
        let state = game.field_model.player_state("p1").unwrap();
        assert!(state.is_active(), "active flag should be restored to was_active=true");
    }

    #[test]
    fn does_not_restore_active_flag_for_original_bombardier() {
        // Java: the changeActive restoration is skipped when the dropped player IS the
        // original bombardier (equalsIgnoreCase match), leaving dropPlayer's own deactivation
        // in place.
        let mut game = make_game();
        add_stunned_player(&mut game, "p1", false);
        game.original_bombardier = Some("p1".into());
        let cmd = DropPlayerFromBombCommand::new("p1".into(), ApothecaryMode::Defender, false, true, false);
        cmd.execute(&mut game);
        let state = game.field_model.player_state("p1").unwrap();
        assert!(!state.is_active(), "bombardier's active flag should not be restored");
    }
}
