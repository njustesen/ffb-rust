use std::collections::{HashSet, VecDeque};
use ffb_model::model::game::Game;
use ffb_model::model::player::PlayerId;
use ffb_model::types::FieldCoordinate;
use ffb_mechanics::skills::SkillId;
use ffb_mechanics::mechanics::STANDARD_GFI_SQUARES;
use ffb_model::enums::{Rules, PS_PRONE, PS_RESERVE, SkillCategory};
use crate::action::{Action, PlayerActionChoice};

/// The side (home or away) in the current game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamSide {
    Home,
    Away,
}

/// Returns all legal `ActivatePlayer` actions for the given side on their turn.
///
/// This lists which players can be activated and with which action type.
pub fn legal_activate_player_actions(game: &Game, side: TeamSide) -> Vec<Action> {
    let team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    let opponent_team = match side {
        TeamSide::Home => &game.team_away,
        TeamSide::Away => &game.team_home,
    };
    let turn_data = match side {
        TeamSide::Home => &game.turn_data_home,
        TeamSide::Away => &game.turn_data_away,
    };

    let acted = match side {
        TeamSide::Home => &game.turn_data_home.acted_player_ids,
        TeamSide::Away => &game.turn_data_away.acted_player_ids,
    };
    let ball_coord = game.field_model.ball_coordinate;

    let mut actions = Vec::new();

    for player in &team.players {
        let pid = player.id.clone();
        if acted.contains(&pid) { continue; } // already activated this turn

        let coord = game.field_model.player_coordinate(&pid);
        let state = game.field_model.player_state(&pid);

        let coord = match coord {
            Some(c) => c,
            None => continue, // off-pitch
        };
        // Java ParityRunner.computeEligiblePlayers: `onPitch = x in 0..=25 && y in 0..=14` — a
        // boxed player keeps a coordinate of (-1,-1) with a Standing-looking base (e.g. a banned
        // Secret Weapon Bombardier), so the Some(coord) check above is NOT enough. Including him
        // made Rust's turn-start snapshot one entry longer than Java's, shifting every pick's
        // modulo for the rest of the turn (goblin bb2025 seed 99 step 23).
        if coord.x < 0 || coord.x > 25 || coord.y < 0 || coord.y > 14 {
            continue;
        }
        let state = match state {
            Some(s) => s,
            None => continue,
        };

        let is_standing = state.is_standing();
        let is_prone = state.base() == PS_PRONE;

        if !is_standing && !is_prone {
            continue; // stunned / KO / dead
        }

        // Prone players activate with MOVE (which stands them up, spending MA) or BLITZ — matching
        // Java computeEligiblePlayers (offers MOVE + BLITZ for prone, not a distinct STAND_UP). The
        // move path stands the player up prone→standing; offering StandUp here diverged from Java's
        // MOVE label AND (in Rust) left the player prone.
        if is_prone {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::Move,
                block_defender_id: None,
            });
            if !turn_data.blitz_used {
                let adj_opponent = opponent_team.players.iter().any(|op| {
                    game.field_model.player_coordinate(&op.id)
                        .map(|oc| oc.is_adjacent(coord))
                        .unwrap_or(false)
                        && game.field_model.player_state(&op.id)
                            .map(|os| os.has_tacklezones())
                            .unwrap_or(false)
                });
                if adj_opponent {
                    actions.push(Action::ActivatePlayer {
                        player_id: pid.clone(),
                        // A prone player blitzing declares plain BLITZ (dispatched as BLITZ_MOVE), NOT
                        // STAND_UP_BLITZ — Java ParityRunner.computeEligiblePlayers offers a prone player
                        // [MOVE, BLITZ] and the stand-up is handled by the standing_up activation flow
                        // (mirrors the prone MOVE case above). Offering StandUpBlitz diverged from Java's
                        // BLITZ label and dispatched a different step sequence.
                        player_action: PlayerActionChoice::Blitz,
                        block_defender_id: None,
                    });
                }
            }
            continue;
        }

        // Standing player actions
        actions.push(Action::ActivatePlayer {
            player_id: pid.clone(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        });

        // Block / Blitz: require adjacent standing opponent; Block + Blitz both use blitz slot.
        // Mirrors Java computeEligiblePlayers: only offered when hasAdjacentBlockTarget().
        // No standalone move-into-contact Blitz — matches Java parity contract exactly.
        if !turn_data.blitz_used {
            let adj_standing_opponent = opponent_team.players.iter().any(|op| {
                game.field_model.player_coordinate(&op.id)
                    .map(|oc| oc.is_adjacent(coord))
                    .unwrap_or(false)
                    && game.field_model.player_state(&op.id)
                        .map(|os| os.has_tacklezones())
                        .unwrap_or(false)
            });
            if adj_standing_opponent {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Block,
                    block_defender_id: None,
                });
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Blitz,
                    block_defender_id: None,
                });
            }
        }

        // Pass: only the ball-carrier can pass, and only if no skill forbids a regular pass.
        // Java gates this on the PROPERTY, not on a list of skill ids:
        //   `!p.hasSkillProperty(NamedProperties.preventRegularPassAction)`
        // (ParityRunner.computeEligiblePlayers, mirroring the engine). Rust hardcoded
        // `!MyBall && !NoBall`, which misses every OTHER skill that registers the property —
        // `bb2016/NoHands` and `bb2016/BallAndChain` both do. The goblin Fanatic carries both, so a
        // Fanatic standing on the ball got a 5-action snapshot here against Java's 4, and the shared
        // actionRng modulo picked a different action: goblin bb2016 seed 19 i=3, Java
        // `live=[MOVE,BLOCK,BLITZ,HAND_OVER] idx=2 → BLITZ` vs Rust
        // `[Move,Block,Blitz,Pass,HandOver] idx=0 → Move`.
        let prevents_regular_pass = player.has_skill_property(
            ffb_model::model::property::named_properties::NamedProperties::PREVENT_REGULAR_PASS_ACTION,
        );
        if !turn_data.pass_used && !prevents_regular_pass && ball_coord == Some(coord) {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::Pass,
                block_defender_id: None,
            });
        }

        // Hail Mary Pass: a canPassToAnySquare carrier declares HAIL_MARY_PASS as its own action
        // (StepDispatchPassing routes on it). Offered DIRECTLY AFTER Pass — ParityRunner inserts
        // it at the same slot, and the two turn-start snapshots must match in length and order.
        if !turn_data.pass_used && ball_coord == Some(coord)
            && player.has_skill_property(
                ffb_model::model::property::named_properties::NamedProperties::CAN_PASS_TO_ANY_SQUARE,
            )
        {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::HailMaryPass,
                block_defender_id: None,
            });
        }

        // HandOff: ball-carrier can hand off to an adjacent teammate
        if !turn_data.hand_over_used && ball_coord == Some(coord) {
            let teammate_adjacent = team.players.iter().any(|tp| {
                tp.id != pid
                    && game.field_model.player_coordinate(&tp.id)
                        .map(|tc| tc.is_adjacent(coord))
                        .unwrap_or(false)
            });
            if teammate_adjacent {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::HandOff,
                    block_defender_id: None,
                });
            }
        }

        // Foul: requires adjacent prone/stunned opponent
        if !turn_data.foul_used {
            let foul_target_exists = opponent_team.players.iter().any(|op| {
                game.field_model.player_coordinate(&op.id)
                    .map(|oc| oc.is_adjacent(coord))
                    .unwrap_or(false)
                    && game.field_model.player_state(&op.id)
                        .map(|os| !os.has_tacklezones() && (os.base() == PS_PRONE || os.base() == ffb_model::enums::PS_STUNNED))
                        .unwrap_or(false)
            });
            if foul_target_exists {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Foul,
                    block_defender_id: None,
                });
            }
        }

        // ThrowBomb: Bombardier skill, uses pass-action slot
        if !turn_data.pass_used && player.has_skill(SkillId::Bombardier) {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::ThrowBomb,
                block_defender_id: None,
            });
        }

        // ThrowTeamMate: player must have ThrowTeamMate skill and an adjacent teammate.
        // Availability follows the edition's `TtmMechanic.isTtmAvailable`: BB2025 tests its own
        // `ttmUsed` flag, while BB2016 and BB2020 spend the team's once-per-turn PASS on the throw
        // and test `!isPassUsed()`. Offering a second BB2020 throw after the pass was gone got the
        // command refused by StepInitSelecting, which advanced nothing and spun the turn.
        let ttm_available = if game.rules == ffb_model::enums::Rules::Bb2025 {
            !turn_data.ttm_used
        } else {
            !turn_data.ttm_used && !turn_data.pass_used
        };
        if ttm_available && player.has_skill(SkillId::ThrowTeamMate) {
            let adjacent_teammate = team.players.iter().any(|tp| {
                tp.id != pid
                    && game.field_model.player_coordinate(&tp.id)
                        .map(|tc| tc.is_adjacent(coord))
                        .unwrap_or(false)
            });
            if adjacent_teammate {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::ThrowTeamMate,
                    block_defender_id: None,
                });
            }
        }

        // KickTeamMate: player must have the Kick Team-Mate skill and an adjacent teammate.
        // Availability follows the edition's `TtmMechanic.isKtmAvailable`: BB2020 and BB2025 test
        // their own `ktmUsed` flag, while BB2016 spends the team's BLITZ on the kick
        // (`bb2016/TtmMechanic.isKtmAvailable` is `!turnData.isBlitzUsed()`). This was gated to
        // BB2025 alone, which left the whole mechanic unreachable in BB2020 even though
        // `skill/mixed/KickTeamMate` is registered for BB2020 and the BB2020 ogre roster carries it.
        let ktm_available = if game.rules == Rules::Bb2016 {
            !turn_data.blitz_used
        } else {
            !turn_data.ktm_used
        };
        if ktm_available && player.has_skill(SkillId::KickTeamMate) {
            let adjacent_teammate = team.players.iter().any(|tp| {
                tp.id != pid
                    && game.field_model.player_coordinate(&tp.id)
                        .map(|tc| tc.is_adjacent(coord))
                        .unwrap_or(false)
            });
            if adjacent_teammate {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::KickTeamMate,
                    block_defender_id: None,
                });
            }
        }

        // Multiple Block (bb2020/bb2025): the client's isMultiBlockActionAvailable rule —
        // canBlockTwoAtOnce (uncancelled) + standing + MORE THAN ONE adjacent blockable
        // opponent. bb2016's multi-block is a different mechanism (MULTI_BLOCK_DEFENDER_ID on
        // ordinary Block sequences) and never dispatches the multiblock step family, so the
        // offer is bb2020/bb2025-only. Declaring it is two-phase like Java: ActingPlayer
        // (MULTIPLE_BLOCK) then CLIENT_SYNCHRONOUS_MULTI_BLOCK with both targets.
        if game.rules != Rules::Bb2016
            && player.has_skill_property(
                ffb_model::model::property::named_properties::NamedProperties::CAN_BLOCK_TWO_AT_ONCE,
            )
            && !player.has_skill_property(
                ffb_model::model::property::named_properties::NamedProperties::PREVENT_REGULAR_BLOCK_ACTION,
            )
        {
            let blockable = ffb_model::util::util_player::UtilPlayer::find_adjacent_blockable_players(
                game, opponent_team, coord,
            );
            if blockable.len() > 1 {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::MultipleBlock,
                    block_defender_id: None,
                });
            }
        }

        // Treacherous (bb2020+ star special): the client's isTreacherousAvailable rule —
        // unused canStabTeamMateForBall skill + an adjacent BLOCKABLE teammate carrying the
        // ball (bb2025 SelectLogicModule.isTreacherousAvailable: UtilCards.hasUnusedSkill…
        // && findAdjacentBlockablePlayers(actingTeam, coord).anyMatch(hasBall)). Declaring it
        // maps to the client's ActingPlayer(PASS_MOVE)+UseSkill(treacherous) command pair.
        // Offer sits DIRECTLY AFTER KickTeamMate — ParityRunner inserts it at the same slot.
        if player.has_skill(SkillId::Treacherous)
            && !player.used_skills.contains(&SkillId::Treacherous)
        {
            let carrier_adjacent =
                ffb_model::util::util_player::UtilPlayer::find_adjacent_blockable_players(
                    game, team, coord,
                )
                .into_iter()
                .any(|tp_id| ffb_model::util::util_player::UtilPlayer::has_ball(game, tp_id));
            if carrier_adjacent {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::Treacherous,
                    block_defender_id: None,
                });
            }
        }

        // Raiding Party (bb2025 star special): the client's isRaidingPartyAvailable rule
        // (LogicModule :200-237) — unused canMoveOpenTeamMate + an acting-team mate that is
        // STANDING, within 5 steps, OPEN (no adjacent opponents with tacklezones) and has an
        // adjacent EMPTY in-field square that itself neighbours an opponent. Offer sits
        // DIRECTLY AFTER Treacherous; ParityRunner mirrors the same position.
        if player.has_skill(SkillId::RaidingParty)
            && !player.used_skills.contains(&SkillId::RaidingParty)
        {
            let own_team = if game.team_home.has_player(pid.as_str()) { &game.team_home } else { &game.team_away };
            let eligible_mate = own_team.players.iter().any(|tm| {
                let Some(tc) = game.field_model.player_coordinate(&tm.id) else { return false };
                let Some(ts) = game.field_model.player_state(&tm.id) else { return false };
                if !ts.is_standing() { return false; }
                if tc.distance_in_steps(coord) > 5 { return false; }
                let open = !opponent_team.players.iter().any(|op| {
                    game.field_model.player_coordinate(&op.id)
                        .map(|oc| oc.is_adjacent(tc)
                            && game.field_model.player_state(&op.id)
                                .map(|os| os.has_tacklezones()).unwrap_or(false))
                        .unwrap_or(false)
                });
                if !open { return false; }
                // an adjacent empty in-field square that neighbours an opponent
                let mut sq = Vec::new();
                for dx in -1i32..=1 { for dy in -1i32..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    let n = ffb_model::types::FieldCoordinate::new(tc.x + dx, tc.y + dy);
                    if ffb_model::types::FieldCoordinateBounds::FIELD.is_in_bounds(n) { sq.push(n); }
                }}
                sq.into_iter().any(|s| {
                    game.field_model.player_at(s).is_none() && {
                        let mut adj = Vec::new();
                        for dx in -1i32..=1 { for dy in -1i32..=1 {
                            if dx == 0 && dy == 0 { continue; }
                            let n = ffb_model::types::FieldCoordinate::new(s.x + dx, s.y + dy);
                            if ffb_model::types::FieldCoordinateBounds::FIELD.is_in_bounds(n) { adj.push(n); }
                        }}
                        adj.into_iter().any(|a| {
                            game.field_model.player_at(a)
                                .map(|oid| opponent_team.has_player(oid))
                                .unwrap_or(false)
                        })
                    }
                })
            });
            if eligible_mate {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::RaidingParty,
                    block_defender_id: None,
                });
            }
        }

        // Look Into My Eyes (bb2025 star special): the client's isLookIntoMyEyesAvailable rule
        // — unused canStealBallFromOpponent + an adjacent BLOCKABLE opponent carrying the ball.
        // Offer sits DIRECTLY AFTER Raiding Party; ParityRunner mirrors the same position.
        if player.has_skill(SkillId::LookIntoMyEyes)
            && !player.used_skills.contains(&SkillId::LookIntoMyEyes)
        {
            let carrier_adjacent = ffb_model::util::util_player::UtilPlayer::find_adjacent_blockable_players(
                    game, opponent_team, coord,
                )
                .into_iter()
                .any(|op_id| ffb_model::util::util_player::UtilPlayer::has_ball(game, op_id));
            if carrier_adjacent {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::LookIntoMyEyes,
                    block_defender_id: None,
                });
            }
        }

        // Baleful Hex (bb2025 star special): the client's isBalefulHexAvailable rule —
        // unused canMakeOpponentMissTurn + any opponent within 5 Chebyshev steps.
        // Offer sits DIRECTLY AFTER Look Into My Eyes; ParityRunner mirrors the position.
        if player.has_skill(SkillId::BalefulHex)
            && !player.used_skills.contains(&SkillId::BalefulHex)
        {
            let opponent_near = opponent_team.players.iter().any(|op| {
                game.field_model.player_coordinate(&op.id)
                    .map(|c| c.distance_in_steps(coord) <= 5)
                    .unwrap_or(false)
            });
            if opponent_near {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::BalefulHex,
                    block_defender_id: None,
                });
            }
        }

        // Black Ink (bb2020+ star special): the client's isBlackInkAvailable rule — unused
        // canGazeAutomatically skill + an adjacent standing-or-prone, NOT-distracted opponent
        // (LogicModule.isBlackInkAvailable). Declaring it maps to the client's
        // ActingPlayer(MOVE)+UseSkill(blackInk) pair; the player continues the move after the
        // gaze. Offer sits DIRECTLY AFTER Treacherous.
        if player.has_skill(SkillId::BlackInk)
            && !player.used_skills.contains(&SkillId::BlackInk)
        {
            let opponent_adjacent = opponent_team.players.iter().any(|op| {
                game.field_model.player_coordinate(&op.id)
                    .map(|oc| oc.is_adjacent(coord))
                    .unwrap_or(false)
                    && game.field_model.player_state(&op.id)
                        .map(|os| (os.is_standing() || os.base() == PS_PRONE) && !os.is_distracted())
                        .unwrap_or(false)
            });
            if opponent_adjacent {
                actions.push(Action::ActivatePlayer {
                    player_id: pid.clone(),
                    player_action: PlayerActionChoice::BlackInk,
                    block_defender_id: None,
                });
            }
        }

        // Punt (BB2025): ball-carrier with Punt skill; once per drive
        if game.rules == Rules::Bb2025
            && !turn_data.punt_used
            && player.has_skill(SkillId::Punt)
            && game.field_model.ball_in_play
            && ball_coord == Some(coord)
        {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::Punt,
                block_defender_id: None,
            });
        }

        // SecureTheBall (BB2025): player at ball square can secure with 2+ roll; once per turn.
        if game.rules == Rules::Bb2025
            && game.field_model.ball_in_play
            && game.field_model.ball_moving
            && ball_coord == Some(coord)
            && !turn_data.secure_the_ball_used
            && !player.has_skill(SkillId::Unsteady)
        {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::SecureTheBall,
                block_defender_id: None,
            });
        }

        // HypnoticGaze — LAST in the list, matching ParityRunner.computeEligiblePlayers:
        //     if (p.hasSkillProperty(NamedProperties.canGazeDuringMove)) actions.add(PlayerAction.GAZE);
        // Offered to any standing player with the property, with no target/adjacency condition.
        // Rust never offered it, so a Vampire's snapshot was one action SHORT of the harness's and
        // the shared actionRng modulo picked a different action every time
        // (vampire bb2016 seed 1 i=1: Java `[MOVE,BLOCK,BLITZ,GAZE] idx=1 -> BLOCK` vs Rust
        // `[Move,Block,Blitz] idx=2 -> Blitz`). GAZE itself is one of the actions
        // `sendConcreteAction` does NOT handle, so both agents immediately DESELECT it (ITER80) —
        // it only has to be PRESENT so the two lists have the same length and order.
        // GATED TO BB2016: `canGazeDuringMove` is registered ONLY by `skill/bb2016/HypnoticGaze` —
        // bb2020's and bb2025's HypnoticGaze do not register it, so Java offers GAZE in bb2016 alone.
        // Rust's `SkillId::HypnoticGaze.properties()` is edition-agnostic and always returns the
        // property, so an ungated check added GAZE to every edition and regressed vampire bb2025 from
        // 0 to 100 fails. (Same trap as Decay's BB2016-only `requiresSecondCasualtyRoll`.)
        if game.rules == Rules::Bb2016
            && player.has_skill_property(
                ffb_model::model::property::named_properties::NamedProperties::CAN_GAZE_DURING_MOVE,
            )
        {
            actions.push(Action::ActivatePlayer {
                player_id: pid.clone(),
                player_action: PlayerActionChoice::HypnoticGaze,
                block_defender_id: None,
            });
        }
    }

    actions
}

/// The `(player, actions)` eligibility list for the `ActivatePlayer` prompt — the same
/// per-player action enumeration as `legal_activate_player_actions` (which Java
/// `ParityRunner.computeEligiblePlayers` mirrors), grouped per player and mapped into model
/// `PlayerAction`s. This is what `StepInitSelecting` should prompt with; a bare
/// "everyone with a coordinate can Move" list never shrinks as players act and never offers
/// Block/Blitz/Pass/HandOver/Foul at all.
pub fn eligible_players_for_activation(game: &Game) -> Vec<(PlayerId, Vec<ffb_model::enums::PlayerAction>)> {
    use ffb_model::enums::PlayerAction as PA;
    use crate::action::PlayerActionChoice as PAC;
    let side = if game.home_playing { TeamSide::Home } else { TeamSide::Away };
    let mut out: Vec<(PlayerId, Vec<PA>)> = Vec::new();
    for action in legal_activate_player_actions(game, side) {
        let Action::ActivatePlayer { player_id, player_action, .. } = action else { continue };
        let pa = match player_action {
            PAC::Move => PA::Move,
            PAC::Block => PA::Block,
            PAC::Blitz => PA::Blitz,
            PAC::StandUp => PA::StandUp,
            PAC::StandUpBlitz => PA::StandUpBlitz,
            PAC::Foul => PA::Foul,
            PAC::Pass => PA::Pass,
            PAC::HandOff => PA::HandOver,
            PAC::ThrowTeamMate => PA::ThrowTeamMate,
            PAC::KickTeamMate => PA::KickTeamMate,
            PAC::ThrowBomb => PA::ThrowBomb,
            PAC::HailMaryPass => PA::HailMaryPass,
            PAC::MultipleBlock => PA::MultipleBlock,
            PAC::RaidingParty => PA::RaidingParty,
            PAC::LookIntoMyEyes => PA::LookIntoMyEyes,
            PAC::BalefulHex => PA::BalefulHex,
            PAC::Treacherous => PA::Treacherous,
            PAC::BlackInk => PA::BlackInk,
            PAC::Punt => PA::Punt,
            PAC::SecureTheBall => PA::SecureTheBall,
            // ParityRunner adds PlayerAction.GAZE for any canGazeDuringMove player; the two
            // eligibility lists must match in LENGTH and ORDER or idx%N diverges.
            PAC::HypnoticGaze => PA::Gaze,
            _ => continue,
        };
        match out.iter_mut().find(|(pid, _)| *pid == player_id) {
            Some((_, acts)) => acts.push(pa),
            None => out.push((player_id, vec![pa])),
        }
    }
    out
}

/// Returns all squares the player can legally move to (BFS, within MA moves, empty squares only).
/// Excludes the player's current square. Sorted by (x, y).
/// Returns the 8 adjacent squares of the player that are on-pitch and unoccupied,
/// sorted by (x, y). Mirrors Java ParityRunner.sendMoveAction: 1-step neighbours only.
/// Returns empty when the player has exhausted MA + GFI allowance (mirrors Java StepGoForIt cap).
pub fn legal_move_targets(game: &Game, player_id: &str) -> Vec<FieldCoordinate> {
    let start = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    // Cap: no move offered once current_move has reached MA + GFI (Java: max MA + 2 rush squares).
    let ma = find_player_ma(game, player_id);
    let cur = game.acting_player.current_move;
    if cur >= ma + STANDARD_GFI_SQUARES {
        return vec![];
    }
    let mut targets: Vec<FieldCoordinate> = start.neighbours().into_iter()
        .filter(|n| n.is_on_pitch() && game.field_model.player_at(*n).is_none())
        .collect();
    targets.sort_by_key(|c| (c.x, c.y));
    targets
}

/// Adjacent empty on-pitch squares of `player_id`, sorted by (x, y) — like `legal_move_targets`
/// but WITHOUT the MA+GFI cap. The agent's phase-2 move-target PRE-DRAW (prone/rooted movers) runs
/// while `acting_player` is still the PREVIOUS activator, so `acting_player.current_move` (read by
/// `legal_move_targets`'s cap) is STALE and can wrongly zero the list (wood_elf seed 25 i=47: prone
/// home_01's list was capped to 0 because home_03 had current_move=4). A fresh activation hasn't spent
/// any MA, so the cap doesn't apply — mirror Java ParityRunner.sendMoveAction, which lists the 1-step
/// neighbours regardless of MA.
pub fn adjacent_empty_move_targets(game: &Game, player_id: &str) -> Vec<FieldCoordinate> {
    let start = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    let mut targets: Vec<FieldCoordinate> = start.neighbours().into_iter()
        .filter(|n| n.is_on_pitch() && game.field_model.player_at(*n).is_none())
        .collect();
    targets.sort_by_key(|c| (c.x, c.y));
    targets
}

/// Returns all squares the blitzing player can legally move to for a BLITZ action:
/// empty squares within MA moves that are adjacent to `defender_id`, plus the player's
/// current square if it is already adjacent to the defender (0-move BLITZ).
/// Sorted by (x, y).
pub fn legal_blitz_move_targets(game: &Game, player_id: &str, defender_id: &str) -> Vec<FieldCoordinate> {
    let start = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    let def_coord = match game.field_model.player_coordinate(defender_id) {
        Some(c) => c,
        None => return vec![],
    };
    let ma = find_player_ma(game, player_id);
    let reachable = bfs_reachable_empty(game, player_id, start, ma);

    let mut targets: Vec<FieldCoordinate> = reachable.into_iter()
        .filter(|c| c.is_adjacent(def_coord))
        .collect();

    // Include current position if already adjacent to defender (0-move BLITZ)
    if start.is_adjacent(def_coord) && !targets.contains(&start) {
        targets.push(start);
    }

    targets.sort_by_key(|c| (c.x, c.y));
    targets
}

/// BFS from `start` over empty squares (excluding `player_id`'s own square if it would
/// block a revisit). Returns up to `ma` moves deep, sorted by (x, y).
fn bfs_reachable_empty(game: &Game, player_id: &str, start: FieldCoordinate, ma: i32) -> Vec<FieldCoordinate> {
    let mut visited: HashSet<FieldCoordinate> = HashSet::new();
    let mut queue: VecDeque<(FieldCoordinate, i32)> = VecDeque::new();
    let mut result: Vec<FieldCoordinate> = Vec::new();

    visited.insert(start);
    queue.push_back((start, 0));

    while let Some((coord, dist)) = queue.pop_front() {
        for n in coord.neighbours() {
            if !n.is_on_pitch() { continue; }
            if visited.contains(&n) { continue; }
            // Occupied by any player (including the mover's own square is already visited)
            if let Some(occ) = game.field_model.player_at(n) {
                if occ != player_id {
                    visited.insert(n); // blocked, don't revisit
                    continue;
                }
            }
            visited.insert(n);
            result.push(n);
            if dist + 1 < ma {
                queue.push_back((n, dist + 1));
            }
        }
    }

    result.sort_by_key(|c| (c.x, c.y));
    result
}

/// Compute the shortest path from `src` to `dest` for `player_id` using BFS.
/// Returns each square visited, excluding `src`, including `dest`.
/// Returns an empty vec if `dest` is unreachable.
pub fn bfs_path(game: &Game, player_id: &str, src: FieldCoordinate, dest: FieldCoordinate) -> Vec<FieldCoordinate> {
    if src == dest { return vec![]; }
    let mut visited: HashSet<FieldCoordinate> = HashSet::new();
    let mut queue: VecDeque<(FieldCoordinate, Vec<FieldCoordinate>)> = VecDeque::new();
    visited.insert(src);
    queue.push_back((src, vec![]));
    while let Some((coord, path)) = queue.pop_front() {
        // Sort neighbours by (x, y) ascending so BFS path ties break the same way as Java's
        // smallest-coordinate-first traversal — without this, neighbour enum order gives a
        // different path than Java (SW before W), shifting dodge-roll positions.
        let mut ns: Vec<FieldCoordinate> = coord.neighbours().into_iter()
            .filter(|n| n.is_on_pitch() && !visited.contains(n))
            .collect();
        ns.sort_by_key(|n| (n.x, n.y));
        for n in ns {
            let occupied = game.field_model.player_at(n).map(|occ| occ != player_id).unwrap_or(false);
            if occupied && n != dest { visited.insert(n); continue; }
            visited.insert(n);
            let mut new_path = path.clone();
            new_path.push(n);
            if n == dest { return new_path; }
            queue.push_back((n, new_path));
        }
    }
    vec![]
}

fn find_player_ma(game: &Game, player_id: &str) -> i32 {
    game.team_home.players.iter()
        .chain(game.team_away.players.iter())
        .find(|p| p.id == player_id)
        .map(|p| p.movement_with_modifiers())
        .unwrap_or(6)
}

/// Returns all opponent players adjacent to the given player (valid block targets).
pub fn legal_block_targets(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };

    let opponent_team = match side {
        TeamSide::Home => &game.team_away,
        TeamSide::Away => &game.team_home,
    };

    let mut targets: Vec<PlayerId> = opponent_team.players.iter()
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
        })
        .filter(|p| {
            game.field_model.player_state(&p.id)
                .map(|s| s.can_be_blocked())
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    // Java pickBlockTarget sorts by (x, y) before picking — match that order.
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns all adjacent friendly players (valid hand-off receivers), sorted by (x, y).
/// Mirrors Java ParityRunner.sendHandOverAction which sorts players by coordinate.
pub fn legal_handoff_receivers(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    let team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    let mut targets: Vec<PlayerId> = team.players.iter()
        .filter(|p| p.id != player_id)
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns adjacent teammates the acting player may throw (Throw Team-Mate): standing teammates
/// carrying the `canBeThrown` property (Right Stuff), coordinate-sorted. Mirrors the reference
/// harness ParityRunner.sendThrowTeamMateAction's candidate set so both agents pick identically.
pub fn legal_throw_team_mate_targets(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    use ffb_model::enums::PS_STANDING;
    use ffb_model::model::property::named_properties::NamedProperties;
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };
    let team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    let mut targets: Vec<PlayerId> = team.players.iter()
        .filter(|p| p.id != player_id)
        // Java ParityRunner.sendThrowTeamMateAction filters on `tp.canBeThrown()` — the ENGINE's own
        // predicate (`Player.canBeThrown`, the one every edition's `TtmMechanic` uses), not the raw
        // `canBeThrown` property.
        //
        // The distinction is the whole reason BB2020 had no Throw Team-Mate coverage. Right Stuff is
        // registered per edition: bb2016 and bb2025 grant `canBeThrown`, bb2020 grants
        // `canBeThrownIfStrengthIs3orLess`. Both harnesses used to test the raw property, so in
        // BB2020 the candidate list was ALWAYS empty — the action was declared 7,235 times per 8,700
        // games and `StepThrowTeamMate` was dispatched exactly zero times. Testing the engine
        // predicate instead lets BB2020 throw the players it is supposed to (ST<=3) and keeps
        // refusing the ones it is not, which the raw-union alternative did not.
        .filter(|p| p.can_be_thrown(game.rules))
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
                && game.field_model.player_state(&p.id)
                    .map(|s| s.base() == PS_STANDING)
                    .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns all on-pitch teammates for the pass-target list (Java ParityRunner.sendPassAction).
/// Matches Java: all teammates with a valid on-field coordinate, sorted by (x, y), 1 actionRng pick.
pub fn legal_pass_receivers(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    let team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    let mut targets: Vec<PlayerId> = team.players.iter()
        .filter(|p| p.id != player_id)
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.x >= 0 && c.x <= 25 && c.y >= 0 && c.y <= 14)
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns all prone/stunned opponent players adjacent to the given player (valid foul targets).
pub fn legal_foul_targets(game: &Game, player_id: &str, side: TeamSide) -> Vec<PlayerId> {
    let coord = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return vec![],
    };

    let opponent_team = match side {
        TeamSide::Home => &game.team_away,
        TeamSide::Away => &game.team_home,
    };

    let mut targets: Vec<PlayerId> = opponent_team.players.iter()
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
        })
        .filter(|p| {
            game.field_model.player_state(&p.id)
                .map(|s| s.can_be_fouled())
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    // Java sortPlayersByCoordinate: sort by (x, y)
    targets.sort_by_key(|id| {
        game.field_model.player_coordinate(id)
            .map(|c| (c.x, c.y))
            .unwrap_or((i32::MAX, i32::MAX))
    });
    targets
}

/// Returns legal kickoff target coordinates (the opponent's half of the field).
///
/// The kicking team can kick to any square in the opponent's half.
pub fn legal_kickoff_targets(_game: &Game, side: TeamSide) -> Vec<FieldCoordinate> {
    // The opponent half: if home is kicking (home_playing = false), the away half is x in 0..=12
    // Field is 26 columns wide, home half is columns 0-12, away half 13-25
    let mut targets = Vec::new();
    let (x_start, x_end) = match side {
        TeamSide::Home => (13i32, 25i32), // home kicks to away half
        TeamSide::Away => (0i32, 12i32),  // away kicks to home half
    };
    for x in x_start..=x_end {
        for y in 1i32..=14i32 {
            let coord = FieldCoordinate::new(x, y);
            if coord.is_on_pitch() {
                targets.push(coord);
            }
        }
    }
    targets
}

/// Enumerates every legal inducement purchase for the `BuyInducements`/`BuyPrayersAndInducements`
/// prompts: every subset of `available` (each item bought 0 or 1 times — the prompt's `available`
/// list does not carry per-item unit prices for larger quantities) whose total cost is within
/// `budget`.
///
/// Design choice: full combinatorial power-set enumeration, not a single greedy/random-walk
/// sample. "Uniformly among all truly legal actions" only holds if every affordable subset is a
/// genuine candidate — a greedy-cheapest-first sampler would systematically bias toward small,
/// cheap purchases and never surface e.g. "skip the cheap apothecary, buy the expensive star
/// player instead." Realistic inducement catalogs are short (rulebook-bounded, well under 20
/// entries), so 2^n stays tractable. As a defensive fallback for a pathologically large
/// `available` list (currently never hit in practice), we cap full enumeration at 20 items and
/// fall back to a single cheapest-first greedy-affordable subset beyond that, to avoid a 2^n
/// blowup — documented here as an accepted approximation for that (currently theoretical) case.
pub fn legal_inducement_purchases(available: &[(String, i32)], budget: i32) -> Vec<Vec<(String, i32)>> {
    const MAX_FULL_ENUMERATION: usize = 20;
    if available.len() > MAX_FULL_ENUMERATION {
        let mut sorted: Vec<&(String, i32)> = available.iter().collect();
        sorted.sort_by_key(|(_, cost)| *cost);
        let mut remaining = budget;
        let mut subset = Vec::new();
        for (name, cost) in sorted {
            if *cost >= 0 && *cost <= remaining {
                subset.push((name.clone(), *cost));
                remaining -= *cost;
            }
        }
        return vec![subset];
    }

    let n = available.len();
    let mut results = Vec::new();
    for mask in 0u32..(1u32 << n) {
        let mut total = 0i32;
        let mut subset = Vec::new();
        for (i, item) in available.iter().enumerate() {
            if mask & (1 << i) != 0 {
                total += item.1;
                subset.push(item.clone());
            }
        }
        if total <= budget {
            results.push(subset);
        }
    }
    results
}

/// Flattens the `SelectSkill` prompt's `available: Vec<(SkillCategory, Vec<u16>)>` into the
/// concrete set of legal skill ids the coach may pick from (union across all offered
/// categories) — so a real choice can be made instead of the informational-only fallback.
pub fn legal_skill_choices(available: &[(SkillCategory, Vec<u16>)]) -> Vec<u16> {
    available.iter().flat_map(|(_, ids)| ids.iter().copied()).collect()
}

/// The canonical LOS squares for team setup (home-side coordinates; away is mirrored `x -> 25-x`).
/// 1:1 with Java `ParityRunner.placeReserves()`'s `losSquares` and Rust's (orphaned, pre-`driver.rs`)
/// `engine.rs::place_team_canonical`'s `los` array.
const SETUP_LOS_SQUARES: &[(i32, i32)] = &[(12, 7), (12, 6), (12, 8), (12, 5), (12, 9), (12, 4), (12, 10)];
/// The canonical overflow squares (behind the LOS) for team setup, same provenance as
/// `SETUP_LOS_SQUARES`.
const SETUP_OVERFLOW_SQUARES: &[(i32, i32)] = &[
    (5, 5), (5, 7), (5, 9), (6, 6), (6, 8), (4, 6), (4, 8), (3, 6), (3, 8), (2, 5), (2, 9), (1, 7),
];

/// The next legal action in response to `AgentPrompt::TeamSetup { team_id, .. }`: one
/// `Action::PlacePlayer` for the next still-in-RESERVE player of the canonical formation, or
/// `Action::ConfirmSetup` once no reserve remains to place. The engine re-emits `TeamSetup`
/// after every `PlacePlayer` until `ConfirmSetup` is sent, so agents invoke this once per
/// prompt — no internal state needed.
///
/// 1:1 with Java `ParityRunner.placeReserves()`: players in jersey-number order (top 11),
/// **base == RESERVE only** (KO'd/injured players are never placed; a player placed by an
/// earlier prompt is STANDING and simply drops out of the candidate list), first three fill
/// the LOS squares, the rest the overflow squares — and, exactly like Java, each square is
/// **skipped when already occupied** (checked on the mirrored coordinate for the away side;
/// the raw home-side coordinate is what goes into `Action::PlacePlayer`, since
/// `UtilServerSetup::setup_player` transforms it internally for away teams — mirroring here
/// too would double-transform). Without the occupancy skip, one leftover player on a target
/// square deadlocks the whole setup: the same rejected `PlacePlayer` repeats forever.
pub fn canonical_setup_action(game: &Game, team_id: &str) -> Action {
    let home = team_id == game.team_home.id;
    let team = if home { &game.team_home } else { &game.team_away };

    // Cap at 11 fielded players. The cap counts players ON PITCH, not "first 11 jerseys":
    // with 12+-man drafted teams (data/teams/), KO'd/injured players among jerseys 1-11 must
    // be replaced from the reserves (jersey 12+). Taking the first 11 jerseys and THEN
    // filtering out the unavailable ones fielded fewer than min(11, available) players, which
    // Java's setup validation rejects (endless SETUP_ERROR at the half-2 setup, human seed 2).
    // Java ParityRunner.placeReserves applies the same available-first rule.
    let on_pitch = team.players.iter()
        .filter(|p| game.field_model.player_coordinate(&p.id)
            .map(|c| c.is_on_pitch())
            .unwrap_or(false))
        .count();
    if on_pitch >= 11 {
        return Action::ConfirmSetup;
    }
    let mut players: Vec<(PlayerId, i32)> = team.players.iter().map(|p| (p.id.clone(), p.nr)).collect();
    players.sort_by_key(|&(_, nr)| nr);
    let candidates: Vec<PlayerId> = players.into_iter()
        .filter(|(pid, _)| game.field_model.player_state(pid)
            .map(|s| s.base() == PS_RESERVE)
            // At the very first setup no player_state entries exist yet — those players are
            // reserves in all but name (Java's fresh game starts everyone as RESERVE).
            .unwrap_or(true))
        .map(|(pid, _)| pid)
        .collect();
    let Some(next_pid) = candidates.first().cloned() else { return Action::ConfirmSetup };

    let occupied = |raw: FieldCoordinate| {
        let stored = if home { raw } else { raw.transform() };
        game.field_model.player_at(stored).is_some()
    };
    // Java's losNeeded = min(3, available): incrementally, keep filling the LOS until our
    // team holds min(3, held + remaining) of the LOS squares.
    let los_held = SETUP_LOS_SQUARES.iter().filter(|&&(x, y)| {
        let stored = if home { FieldCoordinate::new(x, y) } else { FieldCoordinate::new(x, y).transform() };
        game.field_model.player_at(stored)
            .map(|pid| team.players.iter().any(|p| p.id == *pid))
            .unwrap_or(false)
    }).count();
    let need_los = los_held < (los_held + candidates.len()).min(3);

    let square_lists: &[&[(i32, i32)]] = if need_los {
        &[SETUP_LOS_SQUARES, SETUP_OVERFLOW_SQUARES]
    } else {
        &[SETUP_OVERFLOW_SQUARES]
    };
    for list in square_lists {
        for &(x, y) in *list {
            let raw = FieldCoordinate::new(x, y);
            if !occupied(raw) {
                return Action::PlacePlayer { player_id: next_pid, coord: raw };
            }
        }
    }
    // Every canonical square is taken — nothing sensible left to place.
    Action::ConfirmSetup
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerState, PlayerType, PlayerGender, Rules, PS_PRONE, PS_STANDING, PS_STUNNED, PS_KNOCKED_OUT};

    fn make_game(rules: Rules) -> Game {
        Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), rules)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, state_base: u32, skills: Vec<SkillId>) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skills.into_iter().map(ffb_model::model::SkillWithValue::new).collect(),
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(state_base));
    }

    fn c(x: i32, y: i32) -> FieldCoordinate { FieldCoordinate::new(x, y) }

    fn has_action(actions: &[Action], player_id: &str, choice: PlayerActionChoice) -> bool {
        actions.iter().any(|a| matches!(a, Action::ActivatePlayer { player_id: pid, player_action, .. }
            if pid == player_id && *player_action == choice))
    }

    // ── legal_activate_player_actions ─────────────────────────────────────────

    /// Throw Team-Mate availability follows the edition's `TtmMechanic.isTtmAvailable`: BB2025 tests
    /// its own `ttmUsed` flag, while BB2016 and BB2020 spend the team's once-per-turn PASS on the
    /// throw and test `!isPassUsed()`. Offering a BB2020 throw after the pass was gone got the
    /// command refused by StepInitSelecting, which advanced nothing and spun the turn until the
    /// harness abandoned the game (9 of 10 ogre bb2020 seeds).
    #[test]
    fn ttm_availability_follows_the_edition() {
        for (rules, offered_after_pass) in
            [(Rules::Bb2025, true), (Rules::Bb2020, false), (Rules::Bb2016, false)]
        {
            let mut game = make_game(rules);
            add_player(&mut game, true, "thrower", c(5, 5), PS_STANDING, vec![SkillId::ThrowTeamMate]);
            add_player(&mut game, true, "mate", c(5, 6), PS_STANDING, vec![SkillId::RightStuff]);
            // Before anything is spent, every edition offers the throw.
            let actions = legal_activate_player_actions(&game, TeamSide::Home);
            assert!(has_action(&actions, "thrower", PlayerActionChoice::ThrowTeamMate),
                "{rules:?}: a fresh turn must offer Throw Team-Mate");

            game.turn_data_home.pass_used = true;
            let actions = legal_activate_player_actions(&game, TeamSide::Home);
            assert_eq!(has_action(&actions, "thrower", PlayerActionChoice::ThrowTeamMate),
                offered_after_pass,
                "{rules:?}: Throw Team-Mate availability after the pass is spent");
        }
    }

    #[test]
    fn seed14_my_ball_carrier_not_offered_pass() {
        // high_elf seed 14: the Dragon Prince has My Ball (preventRegularPassAction +
        // preventRegularHandOverAction), so a My Ball ball-carrier must NOT be offered PASS — the
        // engine forbids a regular pass. This is the eligible-list contract the Java ParityRunner
        // harness (computeEligiblePlayers) was fixed to match; a bare ball-carrier IS offered Pass.
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "carrier", c(5, 5), PS_STANDING, vec![SkillId::MyBall]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        game.field_model.ball_in_play = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "carrier", PlayerActionChoice::Pass),
            "a My Ball carrier (preventRegularPassAction) must NOT be offered Pass");

        // Control: a plain carrier (no My Ball) IS offered Pass.
        let mut g2 = make_game(Rules::Bb2025);
        add_player(&mut g2, true, "plain", c(5, 5), PS_STANDING, vec![]);
        g2.field_model.ball_coordinate = Some(c(5, 5));
        g2.field_model.ball_in_play = true;
        let a2 = legal_activate_player_actions(&g2, TeamSide::Home);
        assert!(has_action(&a2, "plain", PlayerActionChoice::Pass),
            "a plain ball-carrier must be offered Pass");
    }

    #[test]
    fn already_acted_player_excluded() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.turn_data_home.acted_player_ids.push("p1".into());
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(actions.is_empty());
    }

    #[test]
    fn off_pitch_player_excluded() {
        let mut game = make_game(Rules::Bb2025);
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        });
        // Never placed on the field: no coordinate/state set.
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(actions.is_empty());
    }

    #[test]
    fn stunned_player_excluded() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STUNNED, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(actions.is_empty());
    }

    #[test]
    fn knocked_out_player_excluded() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_KNOCKED_OUT, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(actions.is_empty());
    }

    #[test]
    fn standing_player_offered_move() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Move));
    }

    #[test]
    fn prone_player_offered_move_without_adjacent_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_PRONE, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        // Java offers a prone player MOVE (= stand up and move), no distinct StandUp; no adjacent
        // opponent means no Blitz.
        assert!(has_action(&actions, "p1", PlayerActionChoice::Move));
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Blitz));
        assert!(!has_action(&actions, "p1", PlayerActionChoice::StandUpBlitz));
    }

    #[test]
    fn prone_player_offered_blitz_with_adjacent_standing_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_PRONE, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        // A prone player blitzing declares plain BLITZ (dispatched BLITZ_MOVE); Java ParityRunner
        // offers [MOVE, BLITZ], not STAND_UP_BLITZ.
        assert!(has_action(&actions, "p1", PlayerActionChoice::Blitz));
        assert!(!has_action(&actions, "p1", PlayerActionChoice::StandUpBlitz));
    }

    #[test]
    fn prone_player_blitz_omitted_when_blitz_used() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_PRONE, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        game.turn_data_home.blitz_used = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Blitz));
    }

    #[test]
    fn block_and_blitz_offered_with_adjacent_standing_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Block));
        assert!(has_action(&actions, "p1", PlayerActionChoice::Blitz));
    }

    #[test]
    fn block_and_blitz_omitted_without_adjacent_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(20, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Block));
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Blitz));
    }

    #[test]
    fn block_omitted_against_prone_adjacent_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_PRONE, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Block));
    }

    #[test]
    fn pass_offered_only_to_ball_carrier() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Pass));
    }

    #[test]
    fn pass_omitted_without_ball() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Pass));
    }

    #[test]
    fn pass_omitted_with_no_ball_skill() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::NoBall]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Pass));
    }

    #[test]
    fn hand_off_offered_with_ball_and_adjacent_teammate() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, true, "p2", c(6, 5), PS_STANDING, vec![]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::HandOff));
    }

    #[test]
    fn hand_off_omitted_without_adjacent_teammate() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::HandOff));
    }

    #[test]
    fn throw_team_mate_targets_are_adjacent_standing_right_stuff_teammates() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "ogre", c(5, 5), PS_STANDING, vec![SkillId::ThrowTeamMate]);
        // Adjacent Right Stuff teammate → a target.
        add_player(&mut game, true, "snot_ok", c(6, 5), PS_STANDING, vec![SkillId::RightStuff]);
        // Right Stuff but NOT adjacent → excluded.
        add_player(&mut game, true, "snot_far", c(9, 9), PS_STANDING, vec![SkillId::RightStuff]);
        // Adjacent Right Stuff but PRONE → excluded (can't be picked up).
        add_player(&mut game, true, "snot_prone", c(4, 5), PS_PRONE, vec![SkillId::RightStuff]);
        // Adjacent teammate WITHOUT Right Stuff → excluded (not throwable).
        add_player(&mut game, true, "lineman", c(5, 6), PS_STANDING, vec![]);
        let targets = legal_throw_team_mate_targets(&game, "ogre", TeamSide::Home);
        assert_eq!(targets, vec!["snot_ok".to_string()]);
    }

    #[test]
    fn throw_team_mate_targets_empty_without_right_stuff_teammate() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "ogre", c(5, 5), PS_STANDING, vec![SkillId::ThrowTeamMate]);
        add_player(&mut game, true, "lineman", c(6, 5), PS_STANDING, vec![]);
        assert!(legal_throw_team_mate_targets(&game, "ogre", TeamSide::Home).is_empty());
    }

    #[test]
    fn foul_offered_against_adjacent_prone_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_PRONE, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Foul));
    }

    #[test]
    fn foul_omitted_against_standing_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::Foul));
    }

    /// §9: a canPassToAnySquare carrier is offered HAIL_MARY_PASS as its own declared action,
    /// DIRECTLY AFTER Pass — ParityRunner inserts it at the same slot, and the two turn-start
    /// snapshots must match in length and order or idx % N pairs different actions.
    #[test]
    fn hail_mary_pass_offered_to_carrier_right_after_pass() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            let mut game = make_game(rules);
            add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::HailMaryPass]);
            game.field_model.ball_coordinate = Some(c(5, 5));
            game.field_model.ball_in_play = true;
            let actions = legal_activate_player_actions(&game, TeamSide::Home);
            let p1_actions: Vec<_> = actions.iter().filter_map(|a| match a {
                Action::ActivatePlayer { player_id, player_action, .. } if player_id == "p1" =>
                    Some(*player_action),
                _ => None,
            }).collect();
            let pass_pos = p1_actions.iter().position(|a| *a == PlayerActionChoice::Pass)
                .unwrap_or_else(|| panic!("{rules:?}: Pass must be offered"));
            assert_eq!(p1_actions.get(pass_pos + 1), Some(&PlayerActionChoice::HailMaryPass),
                "{rules:?}: HailMaryPass must sit directly after Pass, got {p1_actions:?}");
            // No HMP without the ball
            let mut g2 = make_game(rules);
            add_player(&mut g2, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::HailMaryPass]);
            let a2 = legal_activate_player_actions(&g2, TeamSide::Home);
            assert!(!has_action(&a2, "p1", PlayerActionChoice::HailMaryPass),
                "{rules:?}: no HMP offer without the ball");
        }
    }

    /// §9: Treacherous is offered under the CLIENT's rule (bb2025 SelectLogicModule
    /// isTreacherousAvailable): unused skill + an adjacent blockable teammate CARRYING the
    /// Raiding Party (seed-1 i=27 shape): the actor is marked but a STANDING team-mate one
    /// square away is OPEN and can step into an empty square beside an opponent.
    #[test]
    fn raiding_party_offered_with_open_movable_team_mate() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "ivar", c(12, 6), PS_STANDING, vec![SkillId::RaidingParty]);
        add_player(&mut game, true, "mate", c(11, 7), PS_STANDING, vec![]);
        add_player(&mut game, false, "opp1", c(13, 7), PS_STANDING, vec![]);
        add_player(&mut game, false, "opp2", c(13, 8), PS_STANDING, vec![]);
        let a = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&a, "ivar", PlayerActionChoice::RaidingParty),
            "open mate at (11,7) can step to (12,8) beside opp2 — must be offered");
        // Used skill withdraws the offer.
        game.team_home.players.iter_mut().find(|p| p.id == "ivar").unwrap()
            .used_skills.insert(SkillId::RaidingParty);
        let a2 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a2, "ivar", PlayerActionChoice::RaidingParty));
    }

    /// ball. No adjacent carrier → no offer; used skill → no offer.
    #[test]
    fn treacherous_offered_only_with_adjacent_ball_carrier() {
        let mut game = make_game(Rules::Bb2020);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::Treacherous]);
        add_player(&mut game, true, "p2", c(6, 5), PS_STANDING, vec![]);
        // teammate adjacent but NOT carrying → no offer
        let a0 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a0, "p1", PlayerActionChoice::Treacherous));
        // teammate carries the ball → offered
        game.field_model.ball_coordinate = Some(c(6, 5));
        game.field_model.ball_in_play = true;
        let a1 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&a1, "p1", PlayerActionChoice::Treacherous),
            "unused Treacherous + adjacent carrier must be offered");
        // used skill withdraws it
        if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == "p1") {
            p.used_skills.insert(SkillId::Treacherous);
        }
        let a2 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a2, "p1", PlayerActionChoice::Treacherous));
    }

    /// §11: Look Into My Eyes is offered under the CLIENT's rule
    /// (LogicModule.isLookIntoMyEyesAvailable): unused canStealBallFromOpponent + an
    /// adjacent BLOCKABLE opponent carrying the ball.
    #[test]
    fn look_into_my_eyes_offered_only_with_adjacent_opponent_carrier() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "boa", c(5, 5), PS_STANDING, vec![SkillId::LookIntoMyEyes]);
        add_player(&mut game, false, "opp", c(6, 5), PS_STANDING, vec![]);
        // adjacent opponent NOT carrying → no offer
        let a0 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a0, "boa", PlayerActionChoice::LookIntoMyEyes));
        // opponent carries the ball → offered
        game.field_model.ball_coordinate = Some(c(6, 5));
        game.field_model.ball_in_play = true;
        let a1 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&a1, "boa", PlayerActionChoice::LookIntoMyEyes),
            "unused Look Into My Eyes + adjacent opponent carrier must be offered");
        // used skill withdraws it
        if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == "boa") {
            p.used_skills.insert(SkillId::LookIntoMyEyes);
        }
        let a2 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a2, "boa", PlayerActionChoice::LookIntoMyEyes));
    }

    /// §11: Baleful Hex is offered under the CLIENT's rule (LogicModule.isBalefulHexAvailable):
    /// unused canMakeOpponentMissTurn + any opponent within 5 Chebyshev steps.
    #[test]
    fn baleful_hex_offered_only_with_opponent_within_five() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "estelle", c(5, 5), PS_STANDING, vec![SkillId::BalefulHex]);
        add_player(&mut game, false, "opp", c(12, 5), PS_STANDING, vec![]);
        // opponent 7 steps away → no offer
        let a0 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a0, "estelle", PlayerActionChoice::BalefulHex));
        // opponent within 5 → offered
        if let Some(op) = game.team_away.players.iter().map(|p| p.id.clone()).next() {
            game.field_model.set_player_coordinate(&op, c(10, 5));
        }
        let a1 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&a1, "estelle", PlayerActionChoice::BalefulHex),
            "unused Baleful Hex + opponent within 5 steps must be offered");
        // used skill withdraws it
        if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == "estelle") {
            p.used_skills.insert(SkillId::BalefulHex);
        }
        let a2 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a2, "estelle", PlayerActionChoice::BalefulHex));
    }

    /// §9: Black Ink is offered under the CLIENT's rule (LogicModule.isBlackInkAvailable):
    /// unused canGazeAutomatically skill + adjacent standing-or-prone NOT-distracted opponent.
    #[test]
    fn black_ink_offered_only_with_adjacent_undistracted_opponent() {
        let mut game = make_game(Rules::Bb2020);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::BlackInk]);
        // no adjacent opponent → no offer
        let a0 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a0, "p1", PlayerActionChoice::BlackInk));
        add_player(&mut game, false, "o1", c(6, 5), PS_STANDING, vec![]);
        let a1 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&a1, "p1", PlayerActionChoice::BlackInk),
            "unused Black Ink + adjacent standing opponent must be offered");
        // used skill withdraws it
        if let Some(p) = game.team_home.players.iter_mut().find(|p| p.id == "p1") {
            p.used_skills.insert(SkillId::BlackInk);
        }
        let a2 = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&a2, "p1", PlayerActionChoice::BlackInk));
    }

    #[test]
    fn throw_bomb_offered_for_bombardier_skill() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::Bombardier]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::ThrowBomb));
    }

    #[test]
    fn throw_team_mate_offered_with_skill_and_adjacent_teammate() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::ThrowTeamMate]);
        add_player(&mut game, true, "p2", c(6, 5), PS_STANDING, vec![]);
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::ThrowTeamMate));
    }

    /// Kick Team-Mate is available in EVERY edition — `skill/mixed/KickTeamMate` is registered for
    /// BB2020 and BB2025, `skill/bb2016/KickTeamMate` for BB2016, and all three TtmMechanics
    /// implement isKtmAvailable. What differs is WHICH once-per-turn slot the kick spends: BB2016
    /// spends the BLITZ, BB2020 and BB2025 use their own ktmUsed flag. This was gated to BB2025
    /// alone, which left the mechanic unreachable in BB2020 even though the BB2020 ogre roster
    /// carries the skill.
    #[test]
    fn kick_team_mate_availability_follows_the_edition() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            let mut game = make_game(rules);
            add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::KickTeamMate]);
            add_player(&mut game, true, "p2", c(6, 5), PS_STANDING, vec![]);
            let actions = legal_activate_player_actions(&game, TeamSide::Home);
            assert!(has_action(&actions, "p1", PlayerActionChoice::KickTeamMate),
                "{rules:?}: a fresh turn must offer Kick Team-Mate");

            // BB2016 spends the blitz; BB2020/BB2025 spend ktm_used.
            if rules == Rules::Bb2016 {
                game.turn_data_home.blitz_used = true;
            } else {
                game.turn_data_home.ktm_used = true;
            }
            let actions = legal_activate_player_actions(&game, TeamSide::Home);
            assert!(!has_action(&actions, "p1", PlayerActionChoice::KickTeamMate),
                "{rules:?}: Kick Team-Mate must be gone once its slot is spent");
        }
    }

    #[test]
    fn punt_only_offered_in_bb2025_with_ball_in_play() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::Punt]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        game.field_model.ball_in_play = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::Punt));
    }

    #[test]
    fn secure_the_ball_offered_when_ball_moving_at_coord() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(has_action(&actions, "p1", PlayerActionChoice::SecureTheBall));
    }

    #[test]
    fn secure_the_ball_omitted_for_unsteady_player() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![SkillId::Unsteady]);
        game.field_model.ball_coordinate = Some(c(5, 5));
        game.field_model.ball_in_play = true;
        game.field_model.ball_moving = true;
        let actions = legal_activate_player_actions(&game, TeamSide::Home);
        assert!(!has_action(&actions, "p1", PlayerActionChoice::SecureTheBall));
    }

    // ── legal_move_targets / legal_blitz_move_targets ─────────────────────────

    #[test]
    fn legal_move_targets_returns_adjacent_empty_squares() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        let targets = legal_move_targets(&game, "p1");
        assert!(targets.contains(&c(6, 5)));
        assert!(!targets.is_empty());
    }

    #[test]
    fn legal_move_targets_excludes_occupied_squares() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let targets = legal_move_targets(&game, "p1");
        assert!(!targets.contains(&c(6, 5)));
    }

    #[test]
    fn legal_move_targets_empty_when_ma_and_gfi_exhausted() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        game.acting_player.current_move = 8; // movement 6 + STANDARD_GFI_SQUARES 2
        let targets = legal_move_targets(&game, "p1");
        assert!(targets.is_empty());
    }

    #[test]
    fn legal_move_targets_unknown_player_returns_empty() {
        let game = make_game(Rules::Bb2025);
        let targets = legal_move_targets(&game, "ghost");
        assert!(targets.is_empty());
    }

    #[test]
    fn legal_blitz_move_targets_includes_zero_move_when_already_adjacent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STANDING, vec![]);
        let targets = legal_blitz_move_targets(&game, "p1", "op1");
        assert!(targets.contains(&c(5, 5)));
    }

    #[test]
    fn legal_blitz_move_targets_reachable_and_adjacent_to_defender() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(8, 5), PS_STANDING, vec![]);
        let targets = legal_blitz_move_targets(&game, "p1", "op1");
        assert!(targets.iter().all(|t| t.is_adjacent(c(8, 5))));
        assert!(!targets.is_empty());
    }

    // ── bfs_path ───────────────────────────────────────────────────────────────

    #[test]
    fn bfs_path_same_square_returns_empty() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        assert!(bfs_path(&game, "p1", c(5, 5), c(5, 5)).is_empty());
    }

    #[test]
    fn bfs_path_unreachable_returns_empty() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        // Fully surround p1's destination-adjacent square isn't necessary; use an
        // out-of-pitch destination to force unreachability.
        let dest = c(-1, -1);
        assert!(bfs_path(&game, "p1", c(5, 5), dest).is_empty());
    }

    #[test]
    fn bfs_path_finds_short_path() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        let path = bfs_path(&game, "p1", c(5, 5), c(7, 5));
        assert_eq!(path.last(), Some(&c(7, 5)));
        assert!(!path.is_empty());
    }

    // ── legal_block_targets / legal_foul_targets / legal_handoff_receivers / legal_pass_receivers ──

    #[test]
    fn legal_block_targets_returns_adjacent_blockable_opponents_sorted() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op_far", c(6, 4), PS_STANDING, vec![]);
        add_player(&mut game, false, "op_near", c(4, 5), PS_STANDING, vec![]);
        let targets = legal_block_targets(&game, "p1", TeamSide::Home);
        assert_eq!(targets, vec!["op_near".to_string(), "op_far".to_string()]);
    }

    #[test]
    fn legal_block_targets_excludes_prone_opponent() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_PRONE, vec![]);
        let targets = legal_block_targets(&game, "p1", TeamSide::Home);
        assert!(targets.is_empty());
    }

    #[test]
    fn legal_foul_targets_returns_adjacent_prone_or_stunned_opponents() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, false, "op1", c(6, 5), PS_STUNNED, vec![]);
        let targets = legal_foul_targets(&game, "p1", TeamSide::Home);
        assert_eq!(targets, vec!["op1".to_string()]);
    }

    #[test]
    fn legal_handoff_receivers_returns_adjacent_teammates_sorted() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, true, "p2", c(6, 5), PS_STANDING, vec![]);
        add_player(&mut game, true, "p3", c(4, 5), PS_STANDING, vec![]);
        let targets = legal_handoff_receivers(&game, "p1", TeamSide::Home);
        assert_eq!(targets, vec!["p3".to_string(), "p2".to_string()]);
    }

    #[test]
    fn eligible_players_excludes_boxed_standing_player() {
        // A banned/boxed player keeps a Standing-looking base at (-1,-1); ParityRunner's
        // computeEligiblePlayers has an explicit on-pitch guard, and without it Rust's turn-start
        // snapshot was one entry longer, shifting every pick modulo (goblin bb2025 seed 99).
        let mut game = make_game(Rules::Bb2025);
        game.home_playing = true;
        add_player(&mut game, true, "boxed", FieldCoordinate::new(-1, -1), PS_STANDING, vec![]);
        add_player(&mut game, true, "onpitch", FieldCoordinate::new(5, 5), PS_STANDING, vec![]);
        let eligible = eligible_players_for_activation(&game);
        assert!(!eligible.iter().any(|(id, _)| id == "boxed"),
            "an off-pitch (boxed) player must not appear in the activation snapshot");
        assert!(eligible.iter().any(|(id, _)| id == "onpitch"));
    }

    #[test]
    fn legal_pass_receivers_excludes_self_includes_all_onfield_teammates() {
        let mut game = make_game(Rules::Bb2025);
        add_player(&mut game, true, "p1", c(5, 5), PS_STANDING, vec![]);
        add_player(&mut game, true, "p2", c(20, 10), PS_STANDING, vec![]);
        let targets = legal_pass_receivers(&game, "p1", TeamSide::Home);
        assert_eq!(targets, vec!["p2".to_string()]);
    }

    // ── legal_kickoff_targets ──────────────────────────────────────────────────

    #[test]
    fn legal_kickoff_targets_home_kicks_to_away_half() {
        let game = make_game(Rules::Bb2025);
        let targets = legal_kickoff_targets(&game, TeamSide::Home);
        assert!(targets.iter().all(|t| t.x >= 13 && t.x <= 25));
        assert!(!targets.is_empty());
    }

    #[test]
    fn legal_kickoff_targets_away_kicks_to_home_half() {
        let game = make_game(Rules::Bb2025);
        let targets = legal_kickoff_targets(&game, TeamSide::Away);
        assert!(targets.iter().all(|t| t.x >= 0 && t.x <= 12));
        assert!(!targets.is_empty());
    }

    // ── legal_inducement_purchases ─────────────────────────────────────────────

    #[test]
    fn legal_inducement_purchases_includes_empty_subset() {
        let available = vec![("Wizard".to_string(), 150)];
        let subsets = legal_inducement_purchases(&available, 100);
        assert!(subsets.contains(&vec![]));
    }

    #[test]
    fn legal_inducement_purchases_excludes_over_budget_subsets() {
        let available = vec![("Wizard".to_string(), 150), ("Bribe".to_string(), 50)];
        let subsets = legal_inducement_purchases(&available, 50);
        for subset in &subsets {
            let total: i32 = subset.iter().map(|(_, c)| c).sum();
            assert!(total <= 50, "subset {:?} exceeds budget", subset);
        }
        assert!(subsets.contains(&vec![("Bribe".to_string(), 50)]));
        assert!(!subsets.iter().any(|s| s.iter().any(|(n, _)| n == "Wizard")));
    }

    #[test]
    fn legal_inducement_purchases_enumerates_all_affordable_subsets() {
        let available = vec![("A".to_string(), 10), ("B".to_string(), 20)];
        let subsets = legal_inducement_purchases(&available, 30);
        // All 4 subsets (empty, {A}, {B}, {A,B}) are affordable within budget 30.
        assert_eq!(subsets.len(), 4);
    }

    #[test]
    fn legal_inducement_purchases_large_catalog_falls_back_to_greedy() {
        let available: Vec<(String, i32)> = (0..25).map(|i| (format!("item{i}"), 10)).collect();
        let subsets = legal_inducement_purchases(&available, 55);
        // Fallback returns exactly one greedy-affordable subset (cheapest-first; all equal cost
        // here, so up to floor(55/10) = 5 items fit).
        assert_eq!(subsets.len(), 1);
        let total: i32 = subsets[0].iter().map(|(_, c)| c).sum();
        assert!(total <= 55);
        assert_eq!(subsets[0].len(), 5);
    }

    // ── legal_skill_choices ─────────────────────────────────────────────────────

    #[test]
    fn legal_skill_choices_flattens_across_categories() {
        let available = vec![
            (SkillCategory::General, vec![1u16, 2, 3]),
            (SkillCategory::Strength, vec![4, 5]),
        ];
        let ids = legal_skill_choices(&available);
        assert_eq!(ids, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn legal_skill_choices_empty_when_no_categories() {
        let available: Vec<(SkillCategory, Vec<u16>)> = vec![];
        assert!(legal_skill_choices(&available).is_empty());
    }

    /// Java gates the PASS action on the PROPERTY, not on a list of skill ids:
    /// `!p.hasSkillProperty(NamedProperties.preventRegularPassAction)`. Rust hardcoded
    /// `!MyBall && !NoBall`, which misses `bb2016/NoHands` and `bb2016/BallAndChain` — both register
    /// the same property. The goblin Fanatic carries both, so a Fanatic standing on the ball got a
    /// 5-action snapshot against the harness's 4 and the shared actionRng modulo picked a different
    /// action (goblin bb2016 seed 19 i=3: Java `[MOVE,BLOCK,BLITZ,HAND_OVER] idx=2 -> BLITZ` vs Rust
    /// `[Move,Block,Blitz,Pass,HandOver] idx=0 -> Move`).
    #[test]
    fn every_prevent_regular_pass_skill_removes_pass_not_just_my_ball_and_no_ball() {
        for skill in [SkillId::MyBall, SkillId::NoBall, SkillId::NoHands, SkillId::BallAndChain] {
            let mut game = make_game(Rules::Bb2016);
            add_player(&mut game, true, "carrier", c(5, 5), PS_STANDING, vec![skill]);
            game.field_model.ball_coordinate = Some(c(5, 5));
            game.field_model.ball_in_play = true;
            let actions = legal_activate_player_actions(&game, TeamSide::Home);
            assert!(!has_action(&actions, "carrier", PlayerActionChoice::Pass),
                "{skill:?} registers preventRegularPassAction, so Pass must not be offered");
        }

        // Control: a plain carrier IS offered Pass.
        let mut g = make_game(Rules::Bb2016);
        add_player(&mut g, true, "plain", c(5, 5), PS_STANDING, vec![]);
        g.field_model.ball_coordinate = Some(c(5, 5));
        g.field_model.ball_in_play = true;
        assert!(has_action(&legal_activate_player_actions(&g, TeamSide::Home), "plain", PlayerActionChoice::Pass),
            "a plain ball-carrier must be offered Pass");
    }

    /// `ParityRunner.computeEligiblePlayers` adds `PlayerAction.GAZE` for any player with
    /// `canGazeDuringMove`, LAST in the list. Rust never offered it, so a Vampire's snapshot was one
    /// action short of the harness's and the shared actionRng modulo picked a different action
    /// (vampire bb2016 seed 1 i=1: Java `[MOVE,BLOCK,BLITZ,GAZE] idx=1 -> BLOCK` vs Rust
    /// `[Move,Block,Blitz] idx=2 -> Blitz`).
    ///
    /// It is BB2016-ONLY: `canGazeDuringMove` is registered by `skill/bb2016/HypnoticGaze` alone,
    /// while Rust's `SkillId::HypnoticGaze.properties()` is edition-agnostic. An ungated check
    /// regressed vampire bb2025 from 0 to 100 fails.
    #[test]
    fn hypnotic_gaze_is_offered_last_and_only_in_bb2016() {
        let build = |rules: Rules| {
            let mut game = make_game(rules);
            add_player(&mut game, true, "vamp", c(5, 5), PS_STANDING, vec![SkillId::HypnoticGaze]);
            legal_activate_player_actions(&game, TeamSide::Home)
        };

        let bb2016 = build(Rules::Bb2016);
        assert!(has_action(&bb2016, "vamp", PlayerActionChoice::HypnoticGaze),
            "bb2016 offers GAZE for a canGazeDuringMove player");
        // ...and LAST, so the two lists line up index-for-index.
        let last = bb2016.iter().rev().find_map(|a| match a {
            Action::ActivatePlayer { player_id, player_action, .. } if player_id == "vamp" => Some(*player_action),
            _ => None,
        });
        assert_eq!(last, Some(PlayerActionChoice::HypnoticGaze), "GAZE must be last in the list");

        for rules in [Rules::Bb2020, Rules::Bb2025] {
            assert!(!has_action(&build(rules), "vamp", PlayerActionChoice::HypnoticGaze),
                "{rules:?} HypnoticGaze does not register canGazeDuringMove, so no GAZE action");
        }
    }
}
