/// 1:1 translation of com.fumbbl.ffb.server.util.ServerUtilBlock.
///
/// Methods that depend on not-yet-ported helpers (UtilPlayer, ServerUtilPlayer,
/// TurnMode::forceDiceDecorationUpdate, NamedProperties) are stubbed with TODO
/// comments referencing the Java source lines.
use ffb_model::model::game::Game;
use ffb_model::enums::{PlayerState, PlayerAction, PS_BLOCKED, PS_STANDING};
use ffb_model::model::dice_decoration::DiceDecoration;
use ffb_model::model::block_kind::BlockKind;
use ffb_model::types::FieldCoordinate;

pub struct ServerUtilBlock;

impl ServerUtilBlock {
    /// Java: ServerUtilBlock.updateDiceDecorations(GameState)
    /// Delegates to the frenzy-blitz variant with false.
    pub fn update_dice_decorations(game: &mut Game) {
        Self::update_dice_decorations_with_frenzy(game, false);
    }

    /// Java: ServerUtilBlock.updateDiceDecorations(GameState, boolean decorateForFrenzyBlitz)
    ///
    /// Clears dice decorations and rebuilds them based on the acting player's action
    /// and adjacent blockable opponents.
    ///
    /// Full implementation requires:
    /// - UtilPlayer::find_adjacent_blockable_players (TODO)
    /// - UtilPlayer::find_adjacent_prone_players (TODO)
    /// - UtilPlayer::find_blockable_players_two_squares_away (TODO)
    /// - NamedProperties::blocksDuringMove / canBlockSameTeamPlayer (TODO)
    /// - TurnMode::force_dice_decoration_update (TODO)
    /// - Self::find_nr_of_block_dice for per-player dice count (TODO: ServerUtilPlayer stub)
    pub fn update_dice_decorations_with_frenzy(game: &mut Game, decorate_for_frenzy_blitz: bool) {
        let player_action = game.acting_player.player_action;

        // Java lines 34-45: determine action flags
        let is_blitz = player_action == Some(PlayerAction::BlitzMove)
            || (player_action == Some(PlayerAction::Blitz) && decorate_for_frenzy_blitz);
        let is_carnage = player_action == Some(PlayerAction::MaximumCarnage);
        let is_putrid = player_action.map(|a| a.is_putrid()).unwrap_or(false);
        let is_block = player_action.map(|a| a.is_block_action()).unwrap_or(false);
        let is_multi_block = player_action == Some(PlayerAction::MultipleBlock);
        let kicks_downed = player_action.map(|a| a.is_kicking_downed()).unwrap_or(false);

        // Java lines 46-51: guard — only decorate when the acting player exists and the
        // action warrants decoration. blocksDuringMove / canBlockSameTeamPlayer properties
        // and actingPlayer.hasBlocked() are TODO (require NamedProperties + ActingPlayer).
        let should_decorate = is_carnage
            || is_putrid
            || is_blitz
            || is_block
            || is_multi_block
            || kicks_downed;

        if should_decorate {
            // Java line 52: game.getFieldModel().clearDiceDecorations()
            game.field_model.clear_dice_decorations();

            // Java line 53: coordinateAttacker = getPlayerCoordinate(actingPlayer)
            let acting_id = game.acting_player.player_id.clone();
            let attacker_coord = acting_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));

            if let Some(coord) = attacker_coord {
                // Java lines 55-68: find target players depending on action type.
                // Full implementation requires UtilPlayer helpers (TODO).
                // Here we fall back to scanning adjacent on-pitch opponents for basic Block/Blitz.
                let _ = coord; // used in TODO block below
                // TODO: replace with UtilPlayer::find_adjacent_blockable_players / find_adjacent_prone_players
                // TODO: add DiceDecoration per target via Self::add_dice_decorations
            }
        }
    }

    /// Java: ServerUtilBlock.removePlayerBlockStates(Game, PlayerState oldDefenderState)
    ///
    /// Iterates all players; any player whose base state is BLOCKED gets reset to
    /// STANDING (or restored to the old defender state if they were prone/stunned before).
    pub fn remove_player_block_states(game: &mut Game, old_defender_state: Option<PlayerState>) {
        let defender_id = game.defender_id.clone();

        // Collect all player ids + states that need changing to avoid borrow conflicts.
        let updates: Vec<(String, PlayerState)> = game.field_model.player_states
            .iter()
            .filter_map(|(id, &state)| {
                if state.base() == PS_BLOCKED {
                    let new_base = if let (Some(old_state), Some(def_id)) = (old_defender_state, &defender_id) {
                        if old_state.is_prone_or_stunned() && id == def_id {
                            old_state.base()
                        } else {
                            PS_STANDING
                        }
                    } else {
                        PS_STANDING
                    };
                    Some((id.clone(), state.change_base(new_base)))
                } else {
                    None
                }
            })
            .collect();

        for (id, new_state) in updates {
            game.field_model.set_player_state(&id, new_state);
        }
    }

    /// Java: ServerUtilBlock.findNrOfBlockDice(GameState, attacker, defender,
    ///        usingMultiBlock, successfulDauntless) -> int
    ///
    /// Returns the number of block dice (-3..-1, 1..3) for attacker vs defender.
    /// Full implementation requires `ServerUtilPlayer::find_block_strength` and
    /// `RollMechanic::get_total_attacker_strength` (TODO stubs).
    pub fn find_nr_of_block_dice(
        attacker_strength: i32,
        defender_strength: i32,
        same_team: bool,
        using_multi_block: bool,
        add_block_die: bool,
    ) -> i32 {
        // Java: multiBlockDefenderModifier() is +1 when usingMultiBlock.
        let effective_defender_str = if using_multi_block {
            defender_strength + 1
        } else {
            defender_strength
        };

        let mut nr_of_dice = 1i32;

        if attacker_strength > effective_defender_str {
            nr_of_dice = 2;
        }
        if attacker_strength > 2 * effective_defender_str {
            nr_of_dice = 3;
        }
        if attacker_strength < effective_defender_str {
            nr_of_dice = -2;
        }
        if 2 * attacker_strength < effective_defender_str {
            nr_of_dice = -3;
        }

        // Java lines 164-166: same-team block (Ball & Chain) always favours the attacker.
        if same_team {
            nr_of_dice = nr_of_dice.abs();
        }

        // Java lines 169-172: canAddBlockDie skill adds one die if currently 1 or 2.
        if add_block_die && (nr_of_dice == 1 || nr_of_dice == 2) {
            nr_of_dice += 1;
        }

        nr_of_dice
    }

    /// Java: private ServerUtilBlock.addDiceDecorations(GameState, Player[])
    ///
    /// For each target player computes the dice count and pushes a DiceDecoration.
    /// Requires TargetSelectionState, UtilPlayer, NamedProperties — stubbed here.
    /// The public interface is exposed for use by translated steps.
    pub fn add_dice_decoration_for_coord(
        game: &mut Game,
        coord: FieldCoordinate,
        nr_of_dice: i32,
        block_kind: Option<BlockKind>,
    ) {
        let kind = block_kind.unwrap_or(BlockKind::BLOCK);
        game.field_model.add_dice_decoration(
            DiceDecoration::new(coord, nr_of_dice, kind),
        );
    }
}

impl Default for ServerUtilBlock {
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;
    use ffb_model::enums::{Rules, PlayerState, PS_BLOCKED, PS_STANDING, PS_PRONE};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::model::block_kind::BlockKind;
    use ffb_model::model::dice_decoration::DiceDecoration;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    // ── update_dice_decorations ──────────────────────────────────────────────

    #[test]
    fn update_dice_decorations_calls_with_frenzy_false() {
        // Smoke test: calling update_dice_decorations does not panic.
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Block);
        ServerUtilBlock::update_dice_decorations(&mut game);
        // No decorations added because UtilPlayer helpers are TODO, but no panic.
        // The decorations list is cleared.
        assert!(game.field_model.dice_decorations.is_empty());
    }

    #[test]
    fn update_dice_decorations_does_not_decorate_for_move() {
        // Move action should not trigger decoration logic.
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Move);
        // Pre-populate a decoration to verify it is NOT cleared (guard not met).
        game.field_model.add_dice_decoration(
            DiceDecoration::new(FieldCoordinate::new(5, 7), 2, BlockKind::BLOCK),
        );
        ServerUtilBlock::update_dice_decorations(&mut game);
        // Guard not met — Move doesn't clear decorations.
        assert_eq!(game.field_model.dice_decorations.len(), 1);
    }

    #[test]
    fn update_dice_decorations_clears_on_block_action() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.field_model.add_dice_decoration(
            DiceDecoration::new(FieldCoordinate::new(5, 7), 2, BlockKind::BLOCK),
        );
        ServerUtilBlock::update_dice_decorations(&mut game);
        // Block action triggers the guard — decorations are cleared.
        assert!(game.field_model.dice_decorations.is_empty());
    }

    // ── remove_player_block_states ───────────────────────────────────────────

    #[test]
    fn remove_player_block_states_resets_blocked_to_standing() {
        let mut game = make_game();
        game.field_model.set_player_state("p1", PlayerState(PS_BLOCKED));
        game.field_model.set_player_state("p2", PlayerState(PS_STANDING));

        ServerUtilBlock::remove_player_block_states(&mut game, None);

        assert_eq!(game.field_model.player_state("p1").map(|s| s.base()), Some(PS_STANDING));
        assert_eq!(game.field_model.player_state("p2").map(|s| s.base()), Some(PS_STANDING));
    }

    #[test]
    fn remove_player_block_states_preserves_prone_defender() {
        let mut game = make_game();
        let old_state = PlayerState(PS_PRONE);
        game.field_model.set_player_state("def", PlayerState(PS_BLOCKED));
        game.defender_id = Some("def".to_string());

        ServerUtilBlock::remove_player_block_states(&mut game, Some(old_state));

        // Defender was prone before block — restore to prone.
        assert_eq!(game.field_model.player_state("def").map(|s| s.base()), Some(PS_PRONE));
    }

    #[test]
    fn remove_player_block_states_non_defender_always_standing() {
        let mut game = make_game();
        let old_state = PlayerState(PS_PRONE);
        game.field_model.set_player_state("p1", PlayerState(PS_BLOCKED));
        game.field_model.set_player_state("def", PlayerState(PS_BLOCKED));
        game.defender_id = Some("def".to_string());

        ServerUtilBlock::remove_player_block_states(&mut game, Some(old_state));

        // p1 is not the defender → always reset to STANDING.
        assert_eq!(game.field_model.player_state("p1").map(|s| s.base()), Some(PS_STANDING));
    }

    #[test]
    fn remove_player_block_states_non_blocked_untouched() {
        let mut game = make_game();
        game.field_model.set_player_state("p1", PlayerState(PS_PRONE));

        ServerUtilBlock::remove_player_block_states(&mut game, None);

        // Prone player was not BLOCKED — remains prone.
        assert_eq!(game.field_model.player_state("p1").map(|s| s.base()), Some(PS_PRONE));
    }

    // ── find_nr_of_block_dice ────────────────────────────────────────────────

    #[test]
    fn equal_strength_gives_one_die() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 3, false, false, false), 1);
    }

    #[test]
    fn double_attacker_strength_gives_two_dice() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(6, 3, false, false, false), 2);
    }

    #[test]
    fn triple_attacker_strength_gives_three_dice() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(9, 3, false, false, false), 3);
    }

    #[test]
    fn weaker_attacker_gives_minus_two() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 4, false, false, false), -2);
    }

    #[test]
    fn much_weaker_attacker_gives_minus_three() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 7, false, false, false), -3);
    }

    #[test]
    fn same_team_block_always_positive() {
        // Ball & Chain vs own team: -2 becomes +2.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 4, true, false, false), 2);
    }

    #[test]
    fn add_block_die_increments_one_die_to_two() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 3, false, false, true), 2);
    }

    #[test]
    fn add_block_die_increments_two_dice_to_three() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(6, 3, false, false, true), 3);
    }

    #[test]
    fn add_block_die_does_not_increment_three_dice() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(9, 3, false, false, true), 3);
    }

    #[test]
    fn multi_block_adds_one_to_defender_strength() {
        // With usingMultiBlock=true, effective defender str = 3+1 = 4. Attacker 5 > 4 → 2 dice.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(5, 3, false, true, false), 2);
        // Boundary: attacker 4 vs defender 3 normally → 2 dice; with multi → 4 vs 4 → 1 die.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(4, 3, false, true, false), 1);
    }

    // ── add_dice_decoration_for_coord ────────────────────────────────────────

    #[test]
    fn add_dice_decoration_for_coord_appends() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        ServerUtilBlock::add_dice_decoration_for_coord(&mut game, coord, 2, None);

        let decorations = game.field_model.get_dice_decorations();
        assert_eq!(decorations.len(), 1);
        assert_eq!(decorations[0].nr_of_dice, 2);
        assert_eq!(decorations[0].coordinate, Some(coord));
        assert_eq!(decorations[0].block_kind, Some(BlockKind::BLOCK));
    }

    #[test]
    fn add_dice_decoration_for_coord_uses_specified_block_kind() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        ServerUtilBlock::add_dice_decoration_for_coord(&mut game, coord, 1, Some(BlockKind::CHAINSAW));

        assert_eq!(game.field_model.dice_decorations[0].block_kind, Some(BlockKind::CHAINSAW));
    }
}
