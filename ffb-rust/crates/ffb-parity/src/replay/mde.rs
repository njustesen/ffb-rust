//! BACKLOG §O O3(b): the harness's move policy on the replay `Game`, 1:1 from
//! `ffb-java/ffb/ffb-ai/src/main/java/com/fumbbl/ffb/ai/` --
//! `MoveDecisionEngine.selectPlayer` / `selectMoveTarget` + helpers, `PathProbabilityFinder.findAllPaths`
//! (with a 1:1 `java.util.PriorityQueue`: equal-probability ties decide which node -- and so which
//! step count -- a square keeps, and that feeds the GFI probabilities further out), and
//! `ActionScore` / `PolicySampler.argmax`. Both records call these with `argmax = true`, so no
//! `java.util.Random` draw happens.
//!
//! Not the same algorithm as `ffb-mechanics/src/mechanics/path_probability.rs` (that one hand-rolls
//! the dodge modifiers); this one goes through the ruleset's `DodgeModifierFactory` /
//! `GoForItModifierFactory` / AgilityMechanic exactly like the Java.

use ffb_mechanics::modifiers::dodge_context::DodgeContext;
use ffb_mechanics::modifiers::dodge_modifier::DodgeModifier;
use ffb_mechanics::modifiers::dodge_modifier_factory::DodgeModifierFactory;
use ffb_mechanics::modifiers::go_for_it_context::GoForItContext;
use ffb_mechanics::modifiers::go_for_it_modifier_factory::GoForItModifierFactory;
use ffb_model::enums::{PlayerAction, Rules, PS_PRONE, PS_STANDING};
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::team::Team;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::util_player::UtilPlayer;
use std::collections::HashMap;

const MAX_GFI: i32 = 2;

const DIRS: [(i32, i32); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

// ── ActionScore ─────────────────────────────────────────────────────────────

/// `ActionScore`: probability x value x confidence, each clamped.
#[derive(Clone, Copy, Debug)]
pub struct ActionScore {
    pub probability: f64,
    pub value: f64,
    pub confidence: f64,
}

/// `Math.max(lo, Math.min(hi, v))`.
fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    java_max(lo, java_min(hi, v))
}

/// `Math.min(double, double)` (no NaN can reach these call sites; -0.0 < 0.0 as in Java).
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && a.is_sign_negative() {
        return a;
    }
    if a <= b { a } else { b }
}

/// `Math.max(double, double)`.
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && a.is_sign_positive() {
        return a;
    }
    if a >= b { a } else { b }
}

impl ActionScore {
    pub fn new(probability: f64, value: f64, confidence: f64) -> Self {
        ActionScore {
            probability: clamp(probability, 0.0, 1.0),
            value: clamp(value, -1.0, 1.0),
            confidence: clamp(confidence, 0.0, 1.0),
        }
    }

    pub fn score(&self) -> f64 {
        self.probability * self.value * self.confidence
    }

    pub fn softmax_score(&self) -> f64 {
        1.0 + self.score()
    }
}

/// `PolicySampler.argmax`: the first index of the maximum.
pub fn argmax(scores: &[f64]) -> usize {
    let mut best = 0;
    for i in 1..scores.len() {
        if scores[i] > scores[best] {
            best = i;
        }
    }
    best
}

// ── java.util.PriorityQueue<PathNode> ──────────────────────────────────────

#[derive(Clone, Copy)]
struct PathNode {
    coord: FieldCoordinate,
    cum_prob: f64,
    step_count: i32,
}

/// `PathNode.compareTo(other) = Double.compare(other.cumProb, this.cumProb)` (max-heap).
fn compare_to(this: &PathNode, other: &PathNode) -> std::cmp::Ordering {
    other.cum_prob.total_cmp(&this.cum_prob)
}

/// `java.util.PriorityQueue` with natural ordering: `offer` = siftUpComparable,
/// `poll` = siftDownComparable, element for element.
#[derive(Default)]
struct JavaPriorityQueue {
    queue: Vec<PathNode>,
}

impl JavaPriorityQueue {
    fn offer(&mut self, key: PathNode) {
        let mut k = self.queue.len();
        self.queue.push(key);
        while k > 0 {
            let parent = (k - 1) >> 1;
            let e = self.queue[parent];
            if compare_to(&key, &e) != std::cmp::Ordering::Less {
                break;
            }
            self.queue[k] = e;
            k = parent;
        }
        self.queue[k] = key;
    }

    fn poll(&mut self) -> Option<PathNode> {
        if self.queue.is_empty() {
            return None;
        }
        let result = self.queue[0];
        let n = self.queue.len() - 1;
        let x = self.queue[n];
        self.queue.truncate(n);
        if n > 0 {
            let mut k = 0;
            let half = n >> 1;
            while k < half {
                let mut child = 2 * k + 1;
                let mut c = self.queue[child];
                let right = child + 1;
                if right < n && compare_to(&c, &self.queue[right]) == std::cmp::Ordering::Greater {
                    child = right;
                    c = self.queue[child];
                }
                if compare_to(&x, &c) != std::cmp::Ordering::Greater {
                    break;
                }
                self.queue[k] = c;
                k = child;
            }
            self.queue[k] = x;
        }
        Some(result)
    }
}

// ── PathProbabilityFinder ───────────────────────────────────────────────────

fn has_prop(game: &Game, player: &Player, property: &str) -> bool {
    player.has_skill_property_in(game.rules, property)
}

/// `AgilityMechanic.minimumRollDodge(game, player, modifiers)` of the game's ruleset.
fn minimum_roll_dodge(rules: Rules, player: &Player, modifiers: &[&DodgeModifier]) -> i32 {
    DodgeModifierFactory::minimum_roll_edition(
        player.strength_with_modifiers(),
        player.agility_with_modifiers(),
        modifiers,
        rules,
    )
}

/// `PathProbabilityFinder.findAllPaths(game, actingPlayer)`: the best probability of every
/// reachable square (start excluded). The paths themselves are not needed by any record.
pub fn find_all_paths(game: &Game, acting: &ActingPlayer, player: &Player) -> Vec<(FieldCoordinate, f64)> {
    let Some(start) = game.field_model.player_coordinate(&player.id) else {
        return Vec::new();
    };
    let opposing_team = UtilPlayer::find_other_team(game, &player.id);
    let movement_allowance = player.movement_with_modifiers();
    let current_move = acting.current_move;
    let max_steps = movement_allowance - current_move + MAX_GFI;

    let dodge_factory = DodgeModifierFactory::for_rules(game.rules);
    let gfi_factory = GoForItModifierFactory::for_rules(game.rules);

    let ignores_tz = has_prop(game, player, NamedProperties::IGNORE_TACKLEZONES_WHEN_MOVING);

    // GoForItContext(game, player, Collections.emptySet())
    let gfi_ctx = GoForItContext::new(game, player);
    let gfi_applicable = gfi_factory.find_applicable(&gfi_ctx);
    let gfi_cards = gfi_factory.find_card_modifiers(&gfi_ctx);
    let gfi_skills = gfi_factory.find_skill_modifiers(&gfi_ctx);
    let mut gfi_mod_total = 0;
    for m in gfi_applicable.iter().copied().chain(gfi_cards.iter()).chain(gfi_skills.iter()) {
        gfi_mod_total += m.get_modifier();
    }
    let gfi_min_roll = 2.max(2 + gfi_mod_total);
    let gfi_prob = java_max(1.0 / 6.0, (7.0 - gfi_min_roll as f64) / 6.0);

    let mut queue = JavaPriorityQueue::default();
    let mut best_prob: HashMap<FieldCoordinate, f64> = HashMap::new();
    let mut best_node: HashMap<FieldCoordinate, PathNode> = HashMap::new();

    let seed = PathNode { coord: start, cum_prob: 1.0, step_count: 0 };
    queue.offer(seed);
    best_prob.insert(start, 1.0);
    best_node.insert(start, seed);

    while let Some(current) = queue.poll() {
        let recorded = best_prob.get(&current.coord).copied();
        match recorded {
            None => continue,
            Some(r) if current.cum_prob < r - 1e-12 => continue,
            _ => {}
        }
        if current.step_count >= max_steps {
            continue;
        }
        let needs_dodge = !ignores_tz
            && !UtilPlayer::find_adjacent_players_with_tacklezones(game, opposing_team, current.coord, false).is_empty();

        for (dx, dy) in DIRS {
            let to = FieldCoordinate::new(current.coord.x + dx, current.coord.y + dy);
            if !to.is_on_pitch() {
                continue;
            }
            if game.field_model.player_at(to).is_some() {
                continue;
            }
            let new_step_count = current.step_count + 1;
            let total_moves = current_move + new_step_count;
            let mut step_prob = 1.0;
            if needs_dodge {
                let ctx = DodgeContext::new(game, acting, current.coord, to);
                let applicable = dodge_factory.find_applicable(&ctx);
                let skill_mods = dodge_factory.find_skill_modifiers(&ctx);
                let all: Vec<&DodgeModifier> = applicable.iter().copied().chain(skill_mods.iter()).collect();
                let dodge_min_roll = minimum_roll_dodge(game.rules, player, &all);
                step_prob *= java_max(1.0 / 6.0, (7.0 - dodge_min_roll as f64) / 6.0);
            }
            if total_moves > movement_allowance {
                step_prob *= gfi_prob;
            }
            let new_cum_prob = current.cum_prob * step_prob;
            let prev_best = best_prob.get(&to).copied();
            if prev_best.is_none_or(|p| new_cum_prob > p + 1e-12) {
                best_prob.insert(to, new_cum_prob);
                let node = PathNode { coord: to, cum_prob: new_cum_prob, step_count: new_step_count };
                best_node.insert(to, node);
                queue.offer(node);
            }
        }
    }

    let mut result: Vec<(FieldCoordinate, f64)> = best_node
        .into_iter()
        .filter(|(c, _)| *c != start)
        .map(|(c, n)| (c, n.cum_prob))
        .collect();
    // Canonical (x, y) order -- the harness sorts the move_target candidates the same way, and the
    // other caller only takes a maximum.
    result.sort_by_key(|(c, _)| (c.x, c.y));
    result
}

/// `findAllPaths(game, player, action)`: a mock ActingPlayer (currentMove 0) for an unactivated player.
fn find_all_paths_for(game: &Game, player: &Player, action: PlayerAction) -> Vec<(FieldCoordinate, f64)> {
    let mock = ActingPlayer {
        player_id: Some(player.id.clone()),
        player_action: Some(action),
        ..ActingPlayer::default()
    };
    find_all_paths(game, &mock, player)
}

// ── MoveDecisionEngine ──────────────────────────────────────────────────────

/// `MoveDecisionEngine.PlayerSelection` (the parts the record reads). A `None` player = END TURN.
pub struct PlayerSelection {
    pub raw_scores: Vec<f64>,
    pub candidate_players: Vec<Option<String>>,
    pub candidate_actions: Vec<Option<PlayerAction>>,
}

/// `MoveDecisionEngine.MoveResult` (the parts the record reads).
pub struct MoveResult {
    pub candidates: Vec<FieldCoordinate>,
    pub raw_scores: Vec<f64>,
    pub has_end_option: bool,
}

fn chebyshev(a: FieldCoordinate, b: FieldCoordinate) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn advance_score(sq: FieldCoordinate, is_home: bool) -> f64 {
    if is_home { sq.x as f64 / 25.0 } else { (25.0 - sq.x as f64) / 25.0 }
}

fn endzone_distance(sq: FieldCoordinate, is_home: bool) -> i32 {
    if is_home { 25 - sq.x } else { sq.x }
}

fn role_dice_failure_cost(is_ball_carrier: bool, is_ball_retriever: bool) -> f64 {
    if is_ball_carrier {
        return 0.50;
    }
    if is_ball_retriever {
        return 0.40;
    }
    0.25
}

/// `Player.getStrengthWithModifiers(Game)`: minus this game's serious-injury ST decreases.
fn strength_with_modifiers_game(game: &Game, player: &Player) -> i32 {
    use ffb_model::enums::InjuryAttribute;
    let mut decreases = 0;
    // GameResult.getPlayerResult(player): the result of the player's own team.
    let team_result = if game.team_home.has_player(&player.id) { &game.game_result.home } else { &game.game_result.away };
    if let Some(pr) = team_result.player_result(&player.id) {
        if pr.serious_injury.and_then(|s| s.injury_attribute()) == Some(InjuryAttribute::ST) {
            decreases += 1;
        }
        if pr.serious_injury_decay.and_then(|s| s.injury_attribute()) == Some(InjuryAttribute::ST) {
            decreases += 1;
        }
    }
    player.strength_with_modifiers() - decreases
}

/// `Skill.canCancel(property)` over `player.getSkills()` (no temporary skills).
fn any_skill_can_cancel(game: &Game, player: &Player, property: &str) -> bool {
    let cancel = format!("cancels{}{}", property[..1].to_uppercase(), &property[1..]);
    player
        .starting_skills
        .iter()
        .chain(player.extra_skills.iter())
        .any(|sw| sw.skill_id.properties_for(game.rules).contains(&cancel.as_str()))
}

fn effective_guard(game: &Game, player: &Player, opposing_team: &Team) -> bool {
    if !has_prop(game, player, NamedProperties::ASSISTS_BLOCKS_IN_TACKLEZONES) {
        return false;
    }
    let Some(p_coord) = game.field_model.player_coordinate(&player.id) else {
        return false;
    };
    for opp_id in UtilPlayer::find_adjacent_players_with_tacklezones(game, opposing_team, p_coord, false) {
        if let Some(opp) = game.player(opp_id) {
            if any_skill_can_cancel(game, opp, NamedProperties::ASSISTS_BLOCKS_IN_TACKLEZONES) {
                return false;
            }
        }
    }
    true
}

/// `computeBlockProbability`.
pub fn compute_block_probability(game: &Game, attacker: &Player, defender: &Player, my_team: &Team, opponent_team: &Team) -> f64 {
    let atk_coord = game.field_model.player_coordinate(&attacker.id);
    let def_coord = game.field_model.player_coordinate(&defender.id);
    let (Some(atk_coord), Some(def_coord)) = (atk_coord, def_coord) else {
        return 0.33;
    };
    let atk_str = strength_with_modifiers_game(game, attacker);
    let def_str = strength_with_modifiers_game(game, defender);
    let no_assists = has_prop(game, attacker, NamedProperties::IGNORE_BLOCK_ASSISTS)
        || has_prop(game, defender, NamedProperties::IGNORE_BLOCK_ASSISTS);
    let (mut off_assists, mut def_assists) = (0, 0);
    if !no_assists {
        for pid in UtilPlayer::find_adjacent_players_with_tacklezones(game, my_team, def_coord, false) {
            if *pid == attacker.id {
                continue;
            }
            let Some(p) = game.player(pid) else { continue };
            if effective_guard(game, p, opponent_team) || UtilPlayer::find_tacklezones(game, pid) == 0 {
                off_assists += 1;
            }
        }
        for pid in UtilPlayer::find_adjacent_players_with_tacklezones(game, opponent_team, atk_coord, false) {
            if *pid == defender.id {
                continue;
            }
            let Some(p) = game.player(pid) else { continue };
            if effective_guard(game, p, my_team) || UtilPlayer::find_tacklezones(game, pid) == 0 {
                def_assists += 1;
            }
        }
    }
    let eff_atk = 1.max(atk_str + off_assists);
    let eff_def = 1.max(def_str + def_assists);
    if eff_atk >= 2 * eff_def {
        0.70
    } else if eff_atk > eff_def {
        0.56
    } else if eff_atk == eff_def {
        0.33
    } else if eff_atk * 2 >= eff_def {
        0.11
    } else {
        0.04
    }
}

fn near_friendly_ball_carrier(game: &Game, sq: FieldCoordinate, ball: Option<FieldCoordinate>, my_team: &Team) -> bool {
    let Some(ball) = ball else { return false };
    let Some(carrier) = game.field_model.player_at(ball) else { return false };
    if !my_team.has_player(carrier) {
        return false;
    }
    game.field_model.player_coordinate(carrier).is_some_and(|pos| chebyshev(sq, pos) <= 1)
}

fn near_opponent(game: &Game, sq: FieldCoordinate, opponent_team: &Team) -> bool {
    opponent_team
        .players
        .iter()
        .any(|opp| game.field_model.player_coordinate(&opp.id).is_some_and(|pos| chebyshev(sq, pos) <= 1))
}

/// `moveBaseActionScore`.
#[allow(clippy::too_many_arguments)]
fn move_base_action_score(
    game: &Game,
    coord: FieldCoordinate,
    player: &Player,
    current_move: i32,
    is_ball_carrier: bool,
    is_ball_retriever: bool,
    is_receiver: bool,
    ball: Option<FieldCoordinate>,
    player_coord: Option<FieldCoordinate>,
    my_team: &Team,
    opponent_team: &Team,
    is_home: bool,
) -> ActionScore {
    if is_ball_carrier {
        if endzone_distance(coord, is_home) == 0 {
            return ActionScore::new(1.0, 1.0, 1.0);
        }
        let advance = advance_score(coord, is_home);
        let advancing = player_coord.is_none_or(|pc| advance_score(coord, is_home) > advance_score(pc, is_home));
        return if advancing {
            ActionScore::new(1.0, advance * 0.6, 0.50)
        } else {
            ActionScore::new(1.0, advance * 0.3 - 0.1, 0.30)
        };
    }
    if is_ball_retriever {
        let Some(ball) = ball else { return ActionScore::new(1.0, 0.0, 0.30) };
        let dist = chebyshev(coord, ball);
        if dist == 0 {
            return ActionScore::new(1.0, 1.0, 0.95);
        }
        if dist == 1 {
            return ActionScore::new(1.0, 0.80, 0.80);
        }
        return ActionScore::new(1.0, java_max(0.05, 0.7 - 0.1 * dist as f64), 0.55);
    }
    if is_receiver {
        let moves_remaining = player.movement_with_modifiers() - current_move;
        let edz_dist = endzone_distance(coord, is_home);
        if edz_dist <= moves_remaining {
            return ActionScore::new(1.0, 0.70, 0.75);
        }
        return ActionScore::new(1.0, advance_score(coord, is_home) * 0.5, 0.50);
    }
    let adj_friendly = near_friendly_ball_carrier(game, coord, ball, my_team);
    let adj_opponent = near_opponent(game, coord, opponent_team);
    if adj_friendly {
        return ActionScore::new(1.0, 0.50, 0.60);
    }
    if adj_opponent {
        return ActionScore::new(1.0, 0.30, 0.50);
    }
    let current_edz = player_coord.map(|pc| endzone_distance(pc, is_home)).unwrap_or(25);
    let advancing = endzone_distance(coord, is_home) <= current_edz;
    ActionScore::new(1.0, if advancing { 0.10 } else { 0.0 }, 0.30)
}

/// `bestActivationSoftmaxScore`.
#[allow(clippy::too_many_arguments)]
fn best_activation_softmax_score(
    game: &Game,
    player: &Player,
    action: PlayerAction,
    ball: Option<FieldCoordinate>,
    player_coord: Option<FieldCoordinate>,
    is_ball_carrier: bool,
    is_ball_retriever: bool,
    is_receiver: bool,
    my_team: &Team,
    opponent_team: &Team,
    is_home: bool,
) -> f64 {
    let path_map = find_all_paths_for(game, player, action);
    if path_map.is_empty() {
        return 0.0;
    }
    let failure_cost = role_dice_failure_cost(is_ball_carrier, is_ball_retriever);
    let mut best_effective = 0.0;
    for (coord, prob) in path_map {
        let base = move_base_action_score(
            game, coord, player, 0, is_ball_carrier, is_ball_retriever, is_receiver, ball, player_coord,
            my_team, opponent_team, is_home,
        );
        let effective = prob * base.value * base.confidence - (1.0 - prob) * failure_cost;
        if effective > best_effective {
            best_effective = effective;
        }
    }
    1.0 + best_effective
}

/// `computeFoulActionScore`.
fn compute_foul_action_score(game: &Game, fouler: &Player, target: &Player, remaining_activations: i32) -> ActionScore {
    let has_dirty_player = has_prop(game, fouler, NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_FOUL);
    let has_chainsaw = has_prop(game, fouler, NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL);
    let assists = UtilPlayer::find_offensive_foul_assists(game, &fouler.id, &target.id)
        - UtilPlayer::find_defensive_foul_assists(game, &fouler.id, &target.id);
    let target_armour = target.armour_with_modifiers();
    let turn_nr = game.turn_data_home.turn_nr;
    let mut prob = 0.15
        + assists as f64 * 0.12
        + (if has_dirty_player { 0.25 } else { 0.0 })
        + (if has_chainsaw { 0.55 } else { 0.0 });
    prob = java_min(prob, 0.85);
    let mut value = java_max(0.10, (target_armour - 6) as f64 * 0.12 + (if turn_nr >= 7 { 0.15 } else { 0.0 }));
    if remaining_activations <= 2 {
        value = java_min(value * 1.4, 0.55);
    }
    value = java_min(value, 0.55);
    ActionScore::new(prob, value, 0.35)
}

/// `MoveDecisionEngine.selectPlayer(game, myTeam, opponentTeam, isHome, allowBlock=true, rng, argmax=true)`.
pub fn select_player(game: &Game, my_team: &Team, opponent_team: &Team, is_home: bool) -> PlayerSelection {
    let allow_block = true;
    let fm = &game.field_model;
    let ball = fm.ball_coordinate;
    let ball_carrier: Option<&String> = ball.and_then(|b| fm.player_at(b));
    let opponent_has_ball = ball_carrier.is_some_and(|c| opponent_team.has_player(c));
    let ball_is_loose = ball.is_some() && ball_carrier.is_none();

    let mut remaining_activations = 0;
    for p in &my_team.players {
        if let Some(ps) = fm.player_state(&p.id) {
            if ps.is_active() && ps.base() == PS_STANDING {
                remaining_activations += 1;
            }
        }
    }

    let mut cand_players: Vec<Option<String>> = Vec::new();
    let mut cand_actions: Vec<Option<PlayerAction>> = Vec::new();
    let mut cand_scores: Vec<f64> = Vec::new();
    let mut push = |p: &Player, a: PlayerAction, s: f64, players: &mut Vec<Option<String>>, actions: &mut Vec<Option<PlayerAction>>, scores: &mut Vec<f64>| {
        players.push(Some(p.id.clone()));
        actions.push(Some(a));
        scores.push(s);
    };

    for p in &my_team.players {
        let ps = fm.player_state(&p.id);
        let p_coord = fm.player_coordinate(&p.id);
        let (Some(ps), Some(p_coord)) = (ps, p_coord) else { continue };
        if !ps.is_active() {
            continue;
        }
        let standing = ps.base() == PS_STANDING;
        let prone = ps.base() == PS_PRONE;
        if !standing && !prone {
            continue;
        }

        if prone {
            let prone_has_ball = Some(p_coord) == ball;
            let prone_is_retriever = !prone_has_ball
                && ball_is_loose
                && ball.is_some_and(|b| chebyshev(p_coord, b) <= p.movement_with_modifiers());
            let raw = best_activation_softmax_score(
                game, p, PlayerAction::Move, ball, Some(p_coord), prone_has_ball, prone_is_retriever, false,
                my_team, opponent_team, is_home,
            );
            push(p, PlayerAction::Move, raw * 0.75, &mut cand_players, &mut cand_actions, &mut cand_scores);
            continue;
        }

        // 1. Ball carrier -> MOVE
        if Some(p_coord) == ball {
            let s = best_activation_softmax_score(
                game, p, PlayerAction::Move, ball, Some(p_coord), true, false, false, my_team, opponent_team, is_home,
            );
            push(p, PlayerAction::Move, s, &mut cand_players, &mut cand_actions, &mut cand_scores);
            continue;
        }

        if opponent_has_ball {
            let carrier_id = ball_carrier.expect("opponent_has_ball");
            if let (Some(bc_pos), Some(carrier)) = (fm.player_coordinate(carrier_id), game.player(carrier_id)) {
                let dist = chebyshev(p_coord, bc_pos);
                // 2. Blitz ball carrier
                if allow_block && dist > 1 && dist <= p.movement_with_modifiers() + 1 {
                    let bp = compute_block_probability(game, p, carrier, my_team, opponent_team);
                    push(p, PlayerAction::Blitz, ActionScore::new(bp, 0.80, 0.75).softmax_score(), &mut cand_players, &mut cand_actions, &mut cand_scores);
                }
                // 3. Block ball carrier (adjacent)
                if allow_block && dist == 1 && ps.has_tacklezones() {
                    let bp = compute_block_probability(game, p, carrier, my_team, opponent_team);
                    push(p, PlayerAction::Block, ActionScore::new(bp, 0.75, 0.70).softmax_score(), &mut cand_players, &mut cand_actions, &mut cand_scores);
                }
            }
        }

        // 4. Nearest mover to loose ball
        if ball_is_loose {
            if let Some(b) = ball {
                let dist = chebyshev(p_coord, b);
                if dist <= p.movement_with_modifiers() {
                    let s = best_activation_softmax_score(
                        game, p, PlayerAction::Move, ball, Some(p_coord), false, true, false, my_team, opponent_team, is_home,
                    );
                    push(p, PlayerAction::Move, s, &mut cand_players, &mut cand_actions, &mut cand_scores);
                    continue;
                }
            }
        }

        // 5. Block adjacent opponent
        if allow_block && ps.has_tacklezones() {
            let targets = UtilPlayer::find_adjacent_blockable_players(game, opponent_team, p_coord);
            if !targets.is_empty() {
                let mut best_block_prob = 0.0;
                for t in targets {
                    if let Some(target) = game.player(t) {
                        let bp = compute_block_probability(game, p, target, my_team, opponent_team);
                        if bp > best_block_prob {
                            best_block_prob = bp;
                        }
                    }
                }
                push(p, PlayerAction::Block, ActionScore::new(best_block_prob, 0.50, 0.65).softmax_score(), &mut cand_players, &mut cand_actions, &mut cand_scores);
            }
        }

        // 6. Receiver (Catch skill) -> MOVE
        if has_prop(game, p, NamedProperties::CAN_ATTEMPT_CATCH_IN_ADJACENT_SQUARES) {
            let s = best_activation_softmax_score(
                game, p, PlayerAction::Move, ball, Some(p_coord), false, false, true, my_team, opponent_team, is_home,
            );
            push(p, PlayerAction::Move, s, &mut cand_players, &mut cand_actions, &mut cand_scores);
            continue;
        }

        // 7. Support -> MOVE
        let s = best_activation_softmax_score(
            game, p, PlayerAction::Move, ball, Some(p_coord), false, false, false, my_team, opponent_team, is_home,
        );
        push(p, PlayerAction::Move, s, &mut cand_players, &mut cand_actions, &mut cand_scores);

        // 8. FOUL: adjacent prone opponent
        let foul_available = allow_block
            && (!game.turn_data().foul_used
                || my_team.players.iter().any(|pl| has_prop(game, pl, NamedProperties::ALLOWS_ADDITIONAL_FOUL)));
        if foul_available && ps.has_tacklezones() {
            let prone_targets = UtilPlayer::find_adjacent_prone_players(game, opponent_team, p_coord);
            if let Some(target) = prone_targets.first().and_then(|t| game.player(t)) {
                let foul = compute_foul_action_score(game, p, target, remaining_activations);
                push(p, PlayerAction::Foul, foul.softmax_score(), &mut cand_players, &mut cand_actions, &mut cand_scores);
            }
        }
    }

    // 9. End turn
    cand_players.push(None);
    cand_actions.push(None);
    let end_turn_value = -1.0 + 1.0 / (1.max(remaining_activations) as f64);
    cand_scores.push(ActionScore::new(1.0, end_turn_value, 0.30).softmax_score());

    PlayerSelection { raw_scores: cand_scores, candidate_players: cand_players, candidate_actions: cand_actions }
}

/// `MoveDecisionEngine.selectMoveTarget(game, actingPlayer, myTeam, opponentTeam, isHome, rng, argmax=true)`,
/// candidates in the harness's canonical (x, y) order.
pub fn select_move_target(game: &Game, acting: &ActingPlayer, player: &Player, my_team: &Team, opponent_team: &Team, is_home: bool) -> MoveResult {
    let path_map = find_all_paths(game, acting, player);
    let player_coord = game.field_model.player_coordinate(&player.id);
    let ball = game.field_model.ball_coordinate;

    let is_ball_carrier = player_coord.is_some() && player_coord == ball;
    let is_receiver = !is_ball_carrier && has_prop(game, player, NamedProperties::CAN_ATTEMPT_CATCH_IN_ADJACENT_SQUARES);
    let is_ball_retriever = !is_ball_carrier && !is_receiver && ball.is_some_and(|b| path_map.iter().any(|(c, _)| *c == b));

    let can_end_now = acting.current_move > 0
        || (acting.player_action != Some(PlayerAction::Move) && acting.player_action != Some(PlayerAction::Blitz));

    let candidates: Vec<FieldCoordinate> = path_map.iter().map(|(c, _)| *c).collect();
    let n = candidates.len();
    let total = if can_end_now { n + 1 } else { n };
    let mut scores = vec![0.0; total];
    let failure_cost = role_dice_failure_cost(is_ball_carrier, is_ball_retriever);
    for (i, (coord, prob)) in path_map.iter().enumerate() {
        let base = move_base_action_score(
            game, *coord, player, acting.current_move, is_ball_carrier, is_ball_retriever, is_receiver, ball,
            player_coord, my_team, opponent_team, is_home,
        );
        let effective = prob * base.value * base.confidence - (1.0 - prob) * failure_cost;
        scores[i] = 1.0 + effective;
    }
    if can_end_now {
        if let Some(pc) = player_coord.filter(|_| acting.current_move > 0) {
            let end_base = move_base_action_score(
                game, pc, player, acting.current_move, is_ball_carrier, is_ball_retriever, is_receiver, ball,
                player_coord, my_team, opponent_team, is_home,
            );
            scores[n] = 1.0 + end_base.value * end_base.confidence * 0.9;
        } else {
            scores[n] = 1.0 + (-0.05 * 0.50);
        }
    }
    if total == 0 {
        return MoveResult { candidates, raw_scores: Vec::new(), has_end_option: false };
    }
    MoveResult { candidates, raw_scores: scores, has_end_option: can_end_now }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(p: f64, step: i32) -> PathNode {
        PathNode { coord: FieldCoordinate::new(step, 0), cum_prob: p, step_count: step }
    }

    /// java.util.PriorityQueue order on ties, traced through the JDK's siftUpComparable /
    /// siftDownComparable: offers 1.0(#1) 1.0(#2) 0.5(#3) 1.0(#4) 1.0(#5) poll as #1 #5 #4 #2 #3
    /// (NOT insertion order among the 1.0s).
    #[test]
    fn priority_queue_ties_follow_java_heap_order() {
        let mut q = JavaPriorityQueue::default();
        for (p, s) in [(1.0, 1), (1.0, 2), (0.5, 3), (1.0, 4), (1.0, 5)] {
            q.offer(node(p, s));
        }
        let order: Vec<i32> = std::iter::from_fn(|| q.poll()).map(|n| n.step_count).collect();
        assert_eq!(order, vec![1, 5, 4, 2, 3]);
    }

    #[test]
    fn action_score_clamps_and_shifts() {
        assert_eq!(ActionScore::new(2.0, -3.0, 0.5).softmax_score(), 1.0 + 1.0 * -1.0 * 0.5);
        assert_eq!(argmax(&[1.0, 2.0, 2.0, 0.5]), 1);
    }
}
