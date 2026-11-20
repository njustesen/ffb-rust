/// 1:1 translation of com.fumbbl.ffb.server.util.ServerUtilBlock.
use ffb_model::model::game::Game;
use ffb_model::enums::{PlayerState, PlayerAction, PS_BLOCKED, PS_STANDING};
use ffb_model::model::dice_decoration::DiceDecoration;
use ffb_model::model::block_kind::BlockKind;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::util_player::UtilPlayer;
use crate::util::server_util_player::ServerUtilPlayer;

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
    /// and adjacent blockable opponents. Uses ServerUtilPlayer::find_block_strength for
    /// per-target assist counting (Guard skill not yet handled — assistsBlocksInTacklezones
    /// branch headless).
    ///
    /// client-only: TurnMode::force_dice_decoration_update — dice decorations are client-side display only;
    /// headless always uses the conservative path (decorations not rebuilt).
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

        // Also check acting player skill properties
        let acting_player_id = game.acting_player.player_id.clone();
        let blocks_during_move = acting_player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_DURING_MOVE))
            .unwrap_or(false);
        let can_block_same_team = acting_player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_BLOCK_SAME_TEAM_PLAYER))
            .unwrap_or(false);
        let has_blocked = game.acting_player.has_blocked;

        let guard = blocks_during_move
            || ((!has_blocked || decorate_for_frenzy_blitz) && (is_blitz || is_block || is_multi_block || kicks_downed))
            || is_carnage || is_putrid;

        if guard && acting_player_id.is_some() {
            game.field_model.clear_dice_decorations();

            let attacker_coord = acting_player_id.as_deref()
                .and_then(|id| game.field_model.player_coordinate(id));

            if let Some(coord) = attacker_coord {
                let acting_id_str = acting_player_id.as_deref().unwrap_or("");
                let attacker_on_home = game.team_home.has_player(acting_id_str);

                // Collect target IDs before any mutable borrow
                let target_ids: Vec<String> = {
                    let other_team = if attacker_on_home { &game.team_away } else { &game.team_home };
                    if kicks_downed {
                        UtilPlayer::find_adjacent_prone_players(game, other_team, coord)
                            .into_iter().map(|id| id.clone()).collect()
                    } else if player_action == Some(PlayerAction::ViciousVines) {
                        UtilPlayer::find_blockable_players_two_squares_away(game, other_team, coord)
                            .into_iter().map(|id| id.clone()).collect()
                    } else {
                        UtilPlayer::find_adjacent_blockable_players(game, other_team, coord)
                            .into_iter().map(|id| id.clone()).collect()
                    }
                };

                let same_team_ids: Vec<String> = if can_block_same_team {
                    let same_team = if attacker_on_home { &game.team_home } else { &game.team_away };
                    UtilPlayer::find_adjacent_blockable_players(game, same_team, coord)
                        .into_iter().map(|id| id.clone()).collect()
                } else {
                    vec![]
                };

                // Determine block kind for each target
                let block_kind = if player_action.map(|a| a.is_putrid_block()).unwrap_or(false) {
                    BlockKind::VOMIT
                } else if kicks_downed {
                    BlockKind::CHAINSAW
                } else {
                    BlockKind::BLOCK
                };

                // Attacker base strength (used for every target decoration).
                let attacker_base_str = acting_player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.strength_with_modifiers())
                    .unwrap_or(1);
                // Bug (fixed, #12): the multi-block strength modifier is applied inside
                // find_nr_of_block_dice per edition (attacker/defender), not as a spurious
                // attacker `+1` here.

                // Java addDiceDecorations :111 -- `addBlockDie = targetSelectionState != null &&
                // targetSelectionState.getUsedSkills().stream().anyMatch(canAddBlockDie)`: once a
                // Gored by the Bull use sits in the selection state the decoration shows the extra die.
                let add_block_die = Self::selection_state_add_block_die_skill(game).is_some();
                // Java addDiceDecorations: for putrid/chainsaw we skip the dice-count path.
                // For normal blocks we compute per-target strengths via ServerUtilPlayer.findBlockStrength.
                for target_id in &target_ids {
                    if let Some(target_coord) = game.field_model.player_coordinate(target_id) {
                        let nr_of_dice = if block_kind == BlockKind::BLOCK {
                            let att_str = ServerUtilPlayer::find_block_strength(
                                game, coord, attacker_base_str, target_coord,
                            );
                            let def_base_str = game.player(target_id)
                                .map(|p| p.strength_with_modifiers())
                                .unwrap_or(1);
                            let def_str = ServerUtilPlayer::find_block_strength(
                                game, target_coord, def_base_str, coord,
                            );
                            Self::find_nr_of_block_dice(att_str, def_str, false, is_multi_block, game.rules, add_block_die)
                        } else {
                            0
                        };
                        game.field_model.add_dice_decoration(DiceDecoration::new(target_coord, nr_of_dice, block_kind));
                    }
                }
                for target_id in &same_team_ids {
                    if let Some(target_coord) = game.field_model.player_coordinate(target_id) {
                        let att_str = ServerUtilPlayer::find_block_strength(
                            game, coord, attacker_base_str, target_coord,
                        );
                        let def_base_str = game.player(target_id)
                            .map(|p| p.strength_with_modifiers())
                            .unwrap_or(1);
                        let def_str = ServerUtilPlayer::find_block_strength(
                            game, target_coord, def_base_str, coord,
                        );
                        let nr_of_dice = Self::find_nr_of_block_dice(att_str, def_str, true, is_multi_block, game.rules, false);
                        game.field_model.add_dice_decoration(DiceDecoration::new(target_coord, nr_of_dice, BlockKind::BLOCK));
                    }
                }
            }
        }
    }

    /// The `canAddBlockDie` skill (Gored by the Bull) recorded in the target selection state, if
    /// any -- Java `targetSelectionState.getUsedSkills().stream().filter(skill ->
    /// skill.hasSkillProperty(NamedProperties.canAddBlockDie)).findFirst()` (bb2020/bb2025
    /// StepBlockRoll, ServerUtilBlock.addDiceDecorations).
    pub fn selection_state_add_block_die_skill(game: &Game) -> Option<ffb_model::enums::SkillId> {
        game.field_model.target_selection_state.as_ref().and_then(|t| {
            t.get_used_skills().iter().copied()
                .find(|s| s.properties_for(game.rules).contains(&NamedProperties::CAN_ADD_BLOCK_DIE))
        })
    }

    /// The block-dice count of the acting player against `defender_id`: Java
    /// `ServerUtilBlock.findNrOfBlockDice(gameState, attacker, defender, false, successfulDauntless,
    /// doubleTargetStrength, false).getLeft()` with `RollMechanic.getTotalAttackerStrength` folded in
    /// (base strength, successful Dauntless, Horns on a blitz, the defender's weakenOpposingBlitzer,
    /// both sides' assists, the Cheering Fans additional assist). Extracted unchanged from
    /// `StepBlockRoll.handle_initial_roll_and_reroll_with_explicit_selection`, which still uses it, so
    /// the Gored by the Bull decoration check (BACKLOG §M 2b(e)) counts dice exactly as the roll does.
    pub fn total_block_dice(
        game: &Game,
        defender_id: Option<&str>,
        successful_dauntless: bool,
        double_target_strength: bool,
    ) -> i32 {
        let attacker_str = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.strength_with_modifiers())
            .unwrap_or(3);
        let defender_str = defender_id
            .and_then(|id| game.player(id))
            .map(|p| p.strength_with_modifiers())
            .unwrap_or(3);
        // Java RollMechanic.getTotalAttackerStrength: a SUCCESSFUL Dauntless roll raises the
        // attacker's block strength to at least the defender's base strength (doubled when
        // doubleTargetStrength), BEFORE the Horns +1 and before assists.
        let attacker_str = if successful_dauntless {
            let target = if double_target_strength { 2 * defender_str } else { defender_str };
            attacker_str.max(target)
        } else {
            attacker_str
        };
        // Horns (`addStrengthOnBlitz`) +1 on BLITZ / BLITZ_MOVE; the defender's
        // `weakenOpposingBlitzer` -1 once the blitzer has moved.
        let attacker_str = {
            let is_blitz = matches!(game.acting_player.player_action,
                Some(PlayerAction::Blitz) | Some(PlayerAction::BlitzMove));
            let mut s = attacker_str;
            if is_blitz {
                let atk_horns = game.acting_player.player_id.as_deref()
                    .and_then(|id| game.player(id))
                    .map(|p| p.has_skill_property(NamedProperties::ADD_STRENGTH_ON_BLITZ))
                    .unwrap_or(false);
                if atk_horns { s += 1; }
                if game.acting_player.has_moved {
                    let def_weakens = defender_id
                        .and_then(|id| game.player(id))
                        .map(|p| p.has_skill_property(NamedProperties::WEAKEN_OPPOSING_BLITZER))
                        .unwrap_or(false);
                    if def_weakens { s -= 1; }
                }
            }
            s
        };
        // Both sides add their assists (ServerUtilPlayer.findBlockStrength).
        let coords = game.acting_player.player_id.as_deref()
            .and_then(|aid| game.field_model.player_coordinate(aid))
            .zip(defender_id.and_then(|did| game.field_model.player_coordinate(did)));
        let (attacker_str, defender_str) = if let Some((ac, dc)) = coords {
            (
                ServerUtilPlayer::find_block_strength(game, ac, attacker_str, dc),
                ServerUtilPlayer::find_block_strength(game, dc, defender_str, ac),
            )
        } else {
            (attacker_str, defender_str)
        };
        // bb2025 RollMechanic.getTotalAttackerStrength: the Cheering Fans additional assists.
        let additional_assists = if game.home_playing {
            game.home_additional_assists
        } else {
            game.away_additional_assists
        };
        ffb_mechanics::mechanics::block_dice_count(attacker_str + additional_assists, defender_str)
    }

    /// The dice decoration Java's `ServerUtilBlock.updateDiceDecorations` leaves on `target_id`
    /// (`game.getFieldModel().getDiceDecoration(targetCoordinate).getNrOfDice()`), or `None` when it
    /// leaves none. Rust's decorations are display-only and simplified, so the `canAddBlockDie`
    /// arms read this instead: the guard of `updateDiceDecorations(gameState)` (no frenzy
    /// decoration), the target among `findAdjacentBlockablePlayers(otherTeam, attacker)`, the
    /// `addDiceDecorations` skips (a bystander during a blitz, `game.getLastDefenderId()`), 0 dice
    /// for `needsNoDiceDecorations`, else findNrOfBlockDice with the selection state's addBlockDie.
    pub fn dice_decoration_nr_of_dice(game: &Game, target_id: &str) -> Option<i32> {
        let player_action = game.acting_player.player_action?;
        let attacker_id = game.acting_player.player_id.as_deref()?;
        let attacker = game.player(attacker_id)?;
        let is_blitz = player_action == PlayerAction::BlitzMove;
        let is_block = player_action.is_block_action();
        let is_multi_block = player_action == PlayerAction::MultipleBlock;
        let kicks_downed = player_action.is_kicking_downed();
        let guard = attacker.has_skill_property(NamedProperties::BLOCKS_DURING_MOVE)
            || (!game.acting_player.has_blocked && (is_blitz || is_block || is_multi_block || kicks_downed))
            || player_action == PlayerAction::MaximumCarnage
            || player_action.is_putrid();
        if !guard || kicks_downed || player_action == PlayerAction::ViciousVines
            || player_action.is_putrid_block()
        {
            return None;
        }
        let coord = game.field_model.player_coordinate(attacker_id)?;
        let other_team = if game.team_home.has_player(attacker_id) { &game.team_away } else { &game.team_home };
        let adjacent = UtilPlayer::find_adjacent_blockable_players(game, other_team, coord)
            .into_iter().any(|id| id.as_str() == target_id);
        if !adjacent {
            return None;
        }
        let selected = game.field_model.target_selection_state.as_ref()
            .filter(|t| t.is_selected())
            .and_then(|t| t.get_selected_player_id().cloned());
        if selected.as_deref().map(|s| s != target_id).unwrap_or(false)
            || game.last_defender_id.as_deref() == Some(target_id)
        {
            return None;
        }
        if attacker.has_skill_property(NamedProperties::NEEDS_NO_DICE_DECORATIONS) {
            return Some(0);
        }
        let mut nr_of_dice = Self::total_block_dice(game, Some(target_id), false, false);
        if Self::selection_state_add_block_die_skill(game).is_some() && (nr_of_dice == 1 || nr_of_dice == 2) {
            nr_of_dice += 1;
        }
        Some(nr_of_dice)
    }

    /// The condition every `canAddBlockDie` arm (Gored by the Bull) applies before recording the
    /// skill -- bb2025 StepInitSelecting :378-385, StepInitMoving :285-296, StepEndBlocking :98-104
    /// (bb2020 StepInitSelecting :358-365 the same):
    ///
    /// ```java
    /// if (targetSelectionState != null && UtilCards.hasUnusedSkill(actingPlayer, skill)) {
    ///   ... DiceDecoration diceDecoration = game.getFieldModel().getDiceDecoration(targetCoordinate);
    ///   boolean opponentCanMove = UtilCards.hasUnusedSkillWithProperty(defender, canMoveBeforeBeingBlocked);
    ///   if (diceDecoration != null && (nrOfDice == 1 || nrOfDice == 2 || (nrOfDice == 3 && opponentCanMove))
    ///       && targetCoordinate.isAdjacent(playerCoordinate)) { addUsedSkill ... }
    /// ```
    /// Returns `(applies_outer, adds_die)`: the outer `targetSelectionState != null && unused` gate
    /// (which decides whether the arm dispatches at all) and the inner decoration gate.
    pub fn add_block_die_check(game: &Game, skill: ffb_model::enums::SkillId) -> (bool, bool) {
        let Some(tss) = game.field_model.target_selection_state.as_ref() else { return (false, false) };
        let Some(attacker_id) = game.acting_player.player_id.as_deref() else { return (false, false) };
        let unused = game.player(attacker_id)
            .map(|p| p.has_skill(skill) && !p.used_skills.contains(&skill))
            .unwrap_or(false)
            && !game.acting_player.used_skills.contains(&skill);
        if !unused {
            return (false, false);
        }
        let Some(target_id) = tss.get_selected_player_id().cloned() else { return (true, false) };
        let target_coord = game.field_model.player_coordinate(&target_id);
        let player_coord = game.field_model.player_coordinate(attacker_id);
        let opponent_can_move = game.player(&target_id)
            .map(|d| ffb_model::util::util_cards::UtilCards::has_unused_skill_with_property_for(
                d, NamedProperties::CAN_MOVE_BEFORE_BEING_BLOCKED, game.rules))
            .unwrap_or(false);
        let dice_ok = match Self::dice_decoration_nr_of_dice(game, &target_id) {
            Some(n) => n == 1 || n == 2 || (n == 3 && opponent_can_move),
            None => false,
        };
        let adjacent = matches!((target_coord, player_coord), (Some(t), Some(p)) if t.is_adjacent(p));
        (true, dice_ok && adjacent)
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
    /// Returns the number of block dice (-3..-1, 1..3) for pre-computed attacker and
    /// defender strength totals (caller must call ServerUtilPlayer::find_block_strength
    /// to resolve assists before calling this).
    pub fn find_nr_of_block_dice(
        attacker_strength: i32,
        defender_strength: i32,
        same_team: bool,
        using_multi_block: bool,
        rules: ffb_model::enums::Rules,
        add_block_die: bool,
    ) -> i32 {
        // Bug (fixed, #12): the multi-block adjustment was hardcoded to `defender + 1`, which is
        // wrong for EVERY edition. Java applies edition-specific modifiers instead — the attacker
        // gets `RollMechanic.multiBlockAttackerModifier()` (bb2020/bb2025 = -2, bb2016 = 0) via
        // getAttackerBaseStrength, and the defender gets `multiBlockDefenderModifier()`
        // (bb2016 = 2, else 0). Both are applied to base strength before assists; since assists
        // are strength-independent the order does not change the final sum, so applying them here
        // to the already-assist-resolved totals is equivalent.
        use ffb_mechanics::mechanics::{multi_block_attacker_modifier, multi_block_defender_modifier};
        let (effective_attacker_str, effective_defender_str) = if using_multi_block {
            (
                (attacker_strength + multi_block_attacker_modifier(rules)).max(1),
                defender_strength + multi_block_defender_modifier(rules),
            )
        } else {
            (attacker_strength, defender_strength)
        };
        let attacker_strength = effective_attacker_str;

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
            vampire_lord: false,
            necromancer: false,
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
    fn update_dice_decorations_no_clear_without_acting_player() {
        // Java guard: actingPlayer.getPlayer() != null — without player, nothing is cleared.
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Block);
        game.field_model.add_dice_decoration(
            DiceDecoration::new(FieldCoordinate::new(5, 7), 2, BlockKind::BLOCK),
        );
        ServerUtilBlock::update_dice_decorations(&mut game);
        // No acting player set — decorations are not touched.
        assert_eq!(game.field_model.dice_decorations.len(), 1);
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
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 3, false, false, Rules::Bb2025, false), 1);
    }

    #[test]
    fn double_attacker_strength_gives_two_dice() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(6, 3, false, false, Rules::Bb2025, false), 2);
    }

    #[test]
    fn triple_attacker_strength_gives_three_dice() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(9, 3, false, false, Rules::Bb2025, false), 3);
    }

    #[test]
    fn weaker_attacker_gives_minus_two() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 4, false, false, Rules::Bb2025, false), -2);
    }

    #[test]
    fn much_weaker_attacker_gives_minus_three() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 7, false, false, Rules::Bb2025, false), -3);
    }

    #[test]
    fn same_team_block_always_positive() {
        // Ball & Chain vs own team: -2 becomes +2.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 4, true, false, Rules::Bb2025, false), 2);
    }

    #[test]
    fn add_block_die_increments_one_die_to_two() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(3, 3, false, false, Rules::Bb2025, true), 2);
    }

    #[test]
    fn add_block_die_increments_two_dice_to_three() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(6, 3, false, false, Rules::Bb2025, true), 3);
    }

    #[test]
    fn add_block_die_does_not_increment_three_dice() {
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(9, 3, false, false, Rules::Bb2025, true), 3);
    }

    #[test]
    // Bug (fixed, #12): multi-block modifiers are edition-specific. bb2016 gives the DEFENDER +2;
    // bb2020/bb2025 give the ATTACKER -2 (both net a -2 swing). The old test asserted a
    // hardcoded defender +1 that matched no edition.
    fn multi_block_applies_edition_modifiers() {
        // bb2016: defender 3 + 2 = 5; attacker 5 == 5 -> 1 die.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(5, 3, false, true, Rules::Bb2016, false), 1);
        // bb2016: attacker 6 vs defender 5 -> 2 dice.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(6, 3, false, true, Rules::Bb2016, false), 2);
        // bb2025: attacker 5 - 2 = 3 == defender 3 -> 1 die.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(5, 3, false, true, Rules::Bb2025, false), 1);
        // bb2025: attacker 6 - 2 = 4 vs defender 3 -> 2 dice.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(6, 3, false, true, Rules::Bb2025, false), 2);
        // bb2020 matches bb2025 (attacker -2): attacker 4 - 2 = 2 < defender 3 -> -2 dice.
        assert_eq!(ServerUtilBlock::find_nr_of_block_dice(4, 3, false, true, Rules::Bb2020, false), -2);
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

    // ── update_dice_decorations with real dice counts ────────────────────────

    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, strength: i32) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState(PS_STANDING | 0x100)); // active+standing
    }

    #[test]
    fn update_dice_decorations_equal_strength_gives_one_die() {
        // Attacker str=3, defender str=3 → 1 die.
        let mut game = make_game();
        let att_coord = FieldCoordinate::new(5, 7);
        let def_coord = FieldCoordinate::new(6, 7);
        add_player(&mut game, true,  "att", att_coord, 3);
        add_player(&mut game, false, "def", def_coord, 3);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);

        ServerUtilBlock::update_dice_decorations(&mut game);

        assert_eq!(game.field_model.dice_decorations.len(), 1);
        assert_eq!(game.field_model.dice_decorations[0].nr_of_dice, 1);
    }

    #[test]
    fn update_dice_decorations_double_strength_gives_two_dice() {
        // Attacker str=6, defender str=3 → 2 dice.
        let mut game = make_game();
        let att_coord = FieldCoordinate::new(5, 7);
        let def_coord = FieldCoordinate::new(6, 7);
        add_player(&mut game, true,  "att", att_coord, 6);
        add_player(&mut game, false, "def", def_coord, 3);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);

        ServerUtilBlock::update_dice_decorations(&mut game);

        assert_eq!(game.field_model.dice_decorations[0].nr_of_dice, 2);
    }

    #[test]
    fn update_dice_decorations_assist_raises_dice_count() {
        // Attacker str=3, one free assist → effective 4 vs defender str=3 → 2 dice.
        let mut game = make_game();
        let att_coord = FieldCoordinate::new(5, 7);
        let def_coord = FieldCoordinate::new(6, 7);
        let assist_coord = FieldCoordinate::new(6, 8); // adjacent to def, not adjacent to other opponents
        add_player(&mut game, true,  "att",    att_coord,    3);
        add_player(&mut game, false, "def",    def_coord,    3);
        add_player(&mut game, true,  "assist", assist_coord, 3);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);

        ServerUtilBlock::update_dice_decorations(&mut game);

        // assist → att effective str = 4, def str = 3 → 2 dice
        assert_eq!(game.field_model.dice_decorations.len(), 1);
        assert_eq!(game.field_model.dice_decorations[0].nr_of_dice, 2);
    }

    #[test]
    fn update_dice_decorations_hindered_assist_no_change() {
        // Attacker str=3, one hindered assist (opposing TZ adjacent to it) → still 1 die.
        let mut game = make_game();
        let att_coord    = FieldCoordinate::new(5, 7);
        let def_coord    = FieldCoordinate::new(6, 7);
        let assist_coord = FieldCoordinate::new(6, 8);
        let hinder_coord = FieldCoordinate::new(7, 8); // adjacent to assist → blocks the assist
        add_player(&mut game, true,  "att",     att_coord,    3);
        add_player(&mut game, false, "def",     def_coord,    3);
        add_player(&mut game, true,  "assist",  assist_coord, 3);
        add_player(&mut game, false, "hinder",  hinder_coord, 3);
        game.acting_player.player_id = Some("att".into());
        game.acting_player.player_action = Some(PlayerAction::Block);

        ServerUtilBlock::update_dice_decorations(&mut game);

        // Assist is hindered → att effective str stays 3 → 1 die
        assert_eq!(game.field_model.dice_decorations[0].nr_of_dice, 1);
    }
}
