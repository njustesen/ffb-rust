/// 1:1 translation of `com.fumbbl.ffb.server.step.UtilServerSteps` (BB2025).
///
/// Static utility methods used by step implementations.
use ffb_model::model::game::Game;
use ffb_model::enums::{PlayerAction, Rules};
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use crate::step::framework::StepId;

/// Java `UtilActingPlayer.changeActingPlayer`'s transient-state reset: when the acting player
/// changes (deselect / next activation), every BLOCKED player reverts to STANDING, and every MOVING
/// player except the current acting player and the thrower reverts to STANDING. Call this right after
/// the acting player is cleared/changed. A block DECLARED then CANCELLED before the block roll (e.g. a
/// failed Bone Head / Really Stupid) leaves the defender in the transient BLOCKED state that
/// StepInitBlocking sets before the ACTIVATION — only this reset clears it (human seed 16 i=11: an
/// Ogre's Bone-head-cancelled block left the defender BLOCKED in Rust vs STANDING in Java). A
/// normally-resolved block's defender is already Prone/Standing/Stunned by activation end, so this
/// only touches cancelled-block defenders and stray MOVING states.
pub fn reset_blocked_and_moving_players(game: &mut Game) {
    use ffb_model::enums::{PS_BLOCKED, PS_MOVING, PS_STANDING};
    let acting_id = game.acting_player.player_id.clone();
    let thrower_id = game.thrower_id.clone();
    let ids: Vec<String> = game.team_home.players.iter()
        .chain(game.team_away.players.iter())
        .map(|p| p.id.clone())
        .collect();
    for id in ids {
        if let Some(ps) = game.field_model.player_state(&id) {
            let base = ps.base();
            let reset = base == PS_BLOCKED
                || (base == PS_MOVING
                    && acting_id.as_deref() != Some(id.as_str())
                    && thrower_id.as_deref() != Some(id.as_str()));
            if reset {
                game.field_model.set_player_state(&id, ps.change_base(PS_STANDING));
            }
        }
    }
}

/// Java `validateStepId(IStep, StepId)`.
/// Panics if the step's id does not match `expected_id`.
pub fn validate_step_id(actual_id: StepId, expected_id: StepId) {
    if actual_id != expected_id {
        panic!("Wrong step id. Expected {:?} received {:?}", expected_id, actual_id);
    }
}

/// Java `checkCommandWithActingPlayer(GameState, ICommandWithActingPlayer)`.
/// Returns true if `acting_player_id` matches the current acting player.
pub fn check_command_with_acting_player(game: &Game, acting_player_id: &str) -> bool {
    !acting_player_id.is_empty()
        && game.acting_player.player_id.as_deref() == Some(acting_player_id)
}

/// Java `changePlayerAction(IStep, String, PlayerAction, boolean)`.
/// Updates the acting player in the game state, then recomputes move squares and
/// dice decorations (Java: UtilServerPlayerMove.updateMoveSquares +
/// ServerUtilBlock.updateDiceDecorations). Without the updateMoveSquares call the
/// first square of every move had no MoveSquare data, so StepInitMoving never set
/// actingPlayer.dodging and dodge rolls never fired.
pub fn change_player_action(game: &mut Game, player_id: &str, action: PlayerAction, jumping: bool) {
    if !player_id.is_empty() {
        // Java UtilActingPlayer.changeActingPlayer gates the used-skills / blocked-moving resets on
        // `if (changed)` — i.e. only when the acting player actually CHANGES. Capture that before
        // set_player overwrites the id: re-dispatching the SAME player (e.g. a Blitz whose block
        // sub-activation re-invokes changePlayerAction on the blitzer) must NOT reset its per-activation
        // skill usage, or a once-per-activation negatrait like Bloodlust re-rolls (vampire seed 2 i=11:
        // home_03's failed-Bloodlust blitz re-rolled Bloodlust in the block sub-activation → extra dice).
        let changed = game.acting_player.player_id.as_deref() != Some(player_id);
        // Java UtilActingPlayer.changeActingPlayer: standingUp = (oldState.base == PRONE). A prone
        // player being activated is "standing up" this activation, which lets StepStandUp resolve the
        // stand-up. set_player resets standing_up=false, so set it AFTER, from the pre-activation base.
        let pre_state = game.field_model.player_state(player_id);
        let was_prone = pre_state
            .map(|s| s.base() == ffb_model::enums::PS_PRONE).unwrap_or(false);
        game.acting_player.set_player(player_id.to_owned(), action);
        game.acting_player.standing_up = was_prone;
        game.acting_player.jumping = jumping;
        // Java UtilActingPlayer.changeActingPlayer: "// show acting player as moving" —
        // fieldModel.setPlayerState(newPlayer, oldState.changeBase(PlayerState.MOVING)).
        // The base stays MOVING for the whole activation (Java's `isStanding()` treats
        // MOVING as standing) and is restored by the NEXT changeActingPlayer / the
        // activation-end changePlayerAction(null). Without it, a pass-block window's
        // mid-activation state snapshot shows the suspended thrower as Standing while
        // Java shows Moving (amazon seed 11 i=106).
        if changed {
            if let Some(pre) = game.field_model.player_state(player_id) {
                game.field_model.set_player_state(
                    player_id,
                    pre.change_base(ffb_model::enums::PS_MOVING),
                );
            }
        }
        // Java UtilActingPlayer.changeActingPlayer: actingPlayer.setOldPlayerState(oldState) — the
        // player's pre-activation PlayerState, STICKY (set_player cleared it on a genuine change; a
        // same-player re-dispatch keeps the earlier value). TakeRootBehaviour uses this to only roll
        // Take Root when the Treeman STARTED the activation standing — a prone Treeman blitzing (wood_elf
        // seed 1 i=49) must NOT roll Take Root, but Rust was defaulting old_player_state=None→STANDING
        // and rolling it (twice), shifting the block dice.
        if game.acting_player.old_player_state.is_none() {
            game.acting_player.old_player_state = pre_state;
        }
        // Java clears per-activation skill re-roll usage when the acting player changes (legacy
        // engine.rs StepInitSelecting did `used_skills.clear()`). Rust stores skill-reroll "used"
        // flags on Player.used_skills (util_server_re_roll::use_reroll), so reset the activated
        // player's Regular-usage skills here — a Regular skill (Pass, Sure Hands, Catch, …) re-roll
        // is available again on the player's next activation. SkillUsageType::Regular is precisely
        // the set NOT tracked outside the player's own activation (track_outside_activation == false);
        // OncePer{Turn,Half,Drive,Game} usage skills reset at their own boundaries and stay intact.
        // (Human seed 98 i=124: home_03 used its Pass re-roll at i=92 turn 5; without this reset it
        // stayed marked used through turn 7, so find_skill_reroll_source returned None and the pass
        // fell to a declined TRR offer instead of Java's auto-used Pass re-roll.)
        if changed {
            if let Some(p) = game.team_home.player_mut(player_id)
                .or_else(|| game.team_away.player_mut(player_id))
            {
                p.reset_used_skills(ffb_model::enums::SkillUsageType::Regular);
            }
        }
        // Java UtilActingPlayer.changeActingPlayer (`if (changed)` block): resets transient
        // BLOCKED/MOVING states whenever the acting player changes. A block declared then cancelled
        // (e.g. failed Bone Head) leaves the defender BLOCKED until this runs (human seed 16 i=11).
        reset_blocked_and_moving_players(game);
        // Java: UtilServerPlayerMove.updateMoveSquares(pStep.getGameState(), actingPlayer.isJumping());
        crate::util::util_server_player_move::UtilServerPlayerMove::update_move_squares(game, jumping);
        // Java: ServerUtilBlock.updateDiceDecorations(pStep.getGameState());
        crate::util::server_util_block::ServerUtilBlock::update_dice_decorations(game);
    }
}

/// Java `UtilServerSteps.changePlayerAction(step, null, null, false)` — clear the acting
/// player. Mirrors `UtilActingPlayer.changeActingPlayer(game, null, null, false)`:
/// the OLD acting player (if any) leaves its transient MOVING base — a player who has
/// acted becomes STANDING **and inactive** (this is what retires a finished pass-block
/// mover so `checkNoPlayerActive` sees it used), a standing-up/was-prone player drops
/// back PRONE, anyone else becomes STANDING — then the acting player is reset.
/// (The Java bomb-action exception on the hasActed branch is not modelled: it only
/// matters for Bombardiers keeping their throw after moving.)
pub fn change_player_action_to_none(game: &mut Game) {
    use ffb_model::enums::{PS_MOVING, PS_PRONE, PS_STANDING};
    if let Some(old_id) = game.acting_player.player_id.clone() {
        if let Some(state) = game.field_model.player_state(&old_id) {
            if state.base() == PS_MOVING {
                // Java wasProne(): oldPlayerState base was PRONE.
                let was_prone = game.acting_player.old_player_state
                    .map(|s| s.base() == PS_PRONE)
                    .unwrap_or(false);
                let new_state = if game.acting_player.acted() {
                    state.change_base(PS_STANDING).change_active(false)
                } else if game.acting_player.standing_up || was_prone {
                    state.change_base(PS_PRONE)
                } else {
                    state.change_base(PS_STANDING)
                };
                game.field_model.set_player_state(&old_id, new_state);
            }
        }
        // Java UtilActingPlayer.changeActingPlayer `if (changed)` block: reset transient
        // BLOCKED/MOVING states (same as the non-null path in change_player_action).
        reset_blocked_and_moving_players(game);
    }
    // Java ActingPlayer.setPlayer(null): full field reset (mirrors set_player's reset list).
    let ap = &mut game.acting_player;
    ap.player_id = None;
    ap.player_action = None;
    ap.defender_id = None;
    ap.current_move = 0;
    ap.goes_for_it = false;
    ap.dodging = false;
    ap.has_blocked = false;
    ap.has_fouled = false;
    ap.has_passed = false;
    ap.has_moved = false;
    ap.has_fed = false;
    ap.has_acted = false;
    ap.standing_up = false;
    ap.old_player_state = None;
}

/// Java `checkTouchdown(GameState)`.
/// Returns true if there is a touchdown condition: a standing ball carrier is in the enemy
/// end zone and the ball is in play and not moving.
pub fn check_touchdown(game: &Game) -> bool {
    if !game.field_model.ball_in_play || game.field_model.ball_moving {
        return false;
    }
    let ball_pos = match game.field_model.ball_coordinate {
        Some(c) => c,
        None => return false,
    };
    let carrier_id = match game.field_model.player_at(ball_pos) {
        Some(id) => id,
        None => return false,
    };
    let carrier_state = match game.field_model.player_state(carrier_id) {
        Some(s) => s,
        None => return false,
    };
    if carrier_state.is_prone_or_stunned() {
        return false;
    }
    // Java: (ballCarrier != actingPlayer.getPlayer()) || !actingPlayer.isSufferingBloodLust() || !mechanic.allowMovementInEndZone()
    // In BB2020/BB2025 allowMovementInEndZone = false → the blood-lust guard never blocks touchdowns.
    // In BB2016 allowMovementInEndZone = true, so blood-lust could block, but we skip that
    // nuance here (blood-lust + movement in endzone is a rare edge case not relevant to parity).
    let allow_movement_in_endzone = matches!(game.rules, Rules::Bb2016);
    let acting_player_id = game.acting_player.player_id.as_deref();
    let is_acting_player = acting_player_id == Some(carrier_id);
    let suffering_blood_lust = game.acting_player.suffering_blood_lust;
    if is_acting_player && suffering_blood_lust && allow_movement_in_endzone {
        return false;
    }
    let home_has_carrier = game.team_home.players.iter().any(|p| p.id.as_str() == carrier_id);
    let away_has_carrier = game.team_away.players.iter().any(|p| p.id.as_str() == carrier_id);
    (home_has_carrier && FieldCoordinateBounds::ENDZONE_AWAY.is_in_bounds(ball_pos))
        || (away_has_carrier && FieldCoordinateBounds::ENDZONE_HOME.is_in_bounds(ball_pos))
}

/// Java `checkEndOfHalf(GameState)`.
/// Returns true when both teams have completed 8 turns (end of half).
pub fn check_end_of_half(game: &Game) -> bool {
    game.turn_data_home.turn_nr >= 8 && game.turn_data_away.turn_nr >= 8
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{PlayerAction, Rules, PS_STANDING, PS_PRONE};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn validate_step_id_same_is_ok() {
        validate_step_id(StepId::EndTurn, StepId::EndTurn); // no panic
    }

    #[test]
    #[should_panic(expected = "Wrong step id")]
    fn validate_step_id_mismatch_panics() {
        validate_step_id(StepId::EndTurn, StepId::InitStartGame);
    }

    #[test]
    fn check_command_with_acting_player_matches() {
        let mut game = make_game();
        game.acting_player.set_player("p01".into(), PlayerAction::Move);
        assert!(check_command_with_acting_player(&game, "p01"));
        assert!(!check_command_with_acting_player(&game, "p02"));
        assert!(!check_command_with_acting_player(&game, ""));
    }

    #[test]
    fn seed31_moving_blitzer_that_acted_ends_standing_not_prone() {
        // high_elf seed 31 i=14: a Dragon Prince (Steady Footing) starts its blitz PRONE, stands up,
        // moves, blocks (Both Down/Skull → FALLING), and Steady Footing saves it → MOVING. At end of
        // activation changeActingPlayer restores the MOVING base: Java uses the COMPUTED hasActed()
        // (has_moved||has_blocked||…), so an acted player becomes STANDING. Rust read the STALE stored
        // `has_acted` flag (false here) and, since the player was_prone, dropped it back to PRONE →
        // divergence vs Java's Standing. This asserts the computed `acted()` path keeps it STANDING.
        use ffb_model::enums::PS_MOVING;
        let mut game = make_game();
        game.acting_player.set_player("home_01".into(), PlayerAction::Blitz);
        game.acting_player.has_moved = true;   // moved during the blitz
        game.acting_player.has_blocked = true; // blocked during the blitz
        game.acting_player.has_acted = false;  // stale stored flag (the bug's trigger)
        game.acting_player.old_player_state = Some(ffb_model::enums::PlayerState::new(PS_PRONE)); // was_prone
        game.field_model.set_player_coordinate("home_01", FieldCoordinate::new(12, 7));
        game.field_model.set_player_state("home_01", ffb_model::enums::PlayerState::new(PS_MOVING));

        change_player_action_to_none(&mut game);

        assert_eq!(
            game.field_model.player_state("home_01").unwrap().base(),
            PS_STANDING,
            "a blitzer that moved+blocked has acted → MOVING must restore to STANDING, not PRONE"
        );
    }

    #[test]
    fn change_player_action_sets_fields() {
        let mut game = make_game();
        change_player_action(&mut game, "p01", PlayerAction::Blitz, true);
        assert_eq!(game.acting_player.player_id.as_deref(), Some("p01"));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::Blitz));
        assert!(game.acting_player.jumping);
    }

    #[test]
    fn change_player_action_empty_id_is_noop() {
        let mut game = make_game();
        change_player_action(&mut game, "", PlayerAction::Move, false);
        assert!(game.acting_player.player_id.is_none());
    }

    #[test]
    fn reset_blocked_and_moving_players_restores_blocked_and_stray_moving() {
        // Java UtilActingPlayer.changeActingPlayer: BLOCKED → STANDING (always), MOVING → STANDING
        // except the acting player and the thrower. This restores a defender left BLOCKED by a block
        // that was declared then cancelled before the block roll (human seed 16 i=11).
        use ffb_model::enums::{PS_BLOCKED, PS_MOVING, PlayerState, PlayerType, PlayerGender};
        use ffb_model::model::player::Player;
        let mut game = make_game();
        let mk = |id: &str| Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default()
        };
        for id in ["blk", "mov", "thr", "act"] { game.team_home.players.push(mk(id)); }
        game.field_model.set_player_state("blk", PlayerState::new(PS_BLOCKED));
        game.field_model.set_player_state("mov", PlayerState::new(PS_MOVING));
        game.field_model.set_player_state("thr", PlayerState::new(PS_MOVING));
        game.field_model.set_player_state("act", PlayerState::new(PS_MOVING));
        game.thrower_id = Some("thr".into());
        game.acting_player.set_player("act".into(), PlayerAction::Move);

        reset_blocked_and_moving_players(&mut game);

        assert_eq!(game.field_model.player_state("blk").unwrap().base(), PS_STANDING, "BLOCKED → STANDING");
        assert_eq!(game.field_model.player_state("mov").unwrap().base(), PS_STANDING, "stray MOVING → STANDING");
        assert_eq!(game.field_model.player_state("thr").unwrap().base(), PS_MOVING, "thrower MOVING preserved");
        assert_eq!(game.field_model.player_state("act").unwrap().base(), PS_MOVING, "acting-player MOVING preserved");
    }

    #[test]
    fn change_player_action_resets_regular_skill_reroll_but_keeps_once_per_game() {
        // Human seed 98 i=124: a Thrower used its Pass (Regular) re-roll on an earlier turn's pass;
        // the "used" flag stayed set, so its next pass fell to a declined TRR offer instead of the
        // auto-used Pass re-roll. Activating the player must clear Regular-usage skills (Pass) while
        // leaving OncePerGame usage intact.
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
        use ffb_model::model::player::Player;
        let mut game = make_game();
        let mut p = Player {
            id: "thr".into(), name: "thr".into(), nr: 1, position_id: "thrower".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8, ..Default::default()
        };
        p.used_skills.insert(SkillId::Pass);        // Regular  → should clear on activation
        p.used_skills.insert(SkillId::OldPro);      // OncePerGame → should persist
        game.team_home.players.push(p);

        change_player_action(&mut game, "thr", PlayerAction::Pass, false);

        let after = game.team_home.player("thr").unwrap();
        assert!(!after.used_skills.contains(&SkillId::Pass), "Regular Pass re-roll reset on activation");
        assert!(after.used_skills.contains(&SkillId::OldPro), "OncePerGame usage preserved");
    }

    #[test]
    fn check_end_of_half_false_when_turns_low() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 7;
        game.turn_data_away.turn_nr = 8;
        assert!(!check_end_of_half(&game));
    }

    #[test]
    fn check_end_of_half_true_when_both_eight() {
        let mut game = make_game();
        game.turn_data_home.turn_nr = 8;
        game.turn_data_away.turn_nr = 8;
        assert!(check_end_of_half(&game));
    }

    #[test]
    fn check_touchdown_false_when_ball_not_in_play() {
        let game = make_game();
        assert!(!check_touchdown(&game));
    }

    #[test]
    fn check_touchdown_false_when_ball_moving() {
        let mut game = make_game();
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        assert!(!check_touchdown(&game));
    }

    #[test]
    fn check_touchdown_false_when_no_carrier() {
        let mut game = make_game();
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(25, 7));
        assert!(!check_touchdown(&game));
    }

    fn add_player_to_home(game: &mut Game, id: &str, pos: FieldCoordinate, state: u32) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState};
        game.team_home.players.push(Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, PlayerState::new(state));
    }

    #[test]
    fn check_touchdown_home_player_in_away_endzone() {
        let mut game = make_game();
        let ball_pos = FieldCoordinate::new(25, 7);
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(ball_pos);
        add_player_to_home(&mut game, "p01", ball_pos, PS_STANDING);
        assert!(check_touchdown(&game));
    }

    #[test]
    fn check_touchdown_false_when_carrier_prone() {
        let mut game = make_game();
        let ball_pos = FieldCoordinate::new(25, 7);
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(ball_pos);
        add_player_to_home(&mut game, "p01", ball_pos, PS_PRONE);
        assert!(!check_touchdown(&game));
    }
}
