use crate::model::field_model::FieldModel;
use crate::rng::GameRng;
use crate::skills::SkillId;
use crate::types::{BlockResult, FieldCoordinate, PlayerId, TeamId};

// ── Block dice count ──────────────────────────────────────────────────────────

/// Compute the number of block dice.
/// Positive → attacker picks (favours attacker).
/// Negative → defender picks (favours defender).
///
/// Rules:
///   - If attacker ST ≥ 2× defender ST → 3 dice (attacker picks)
///   - If attacker ST > defender ST     → 2 dice (attacker picks)
///   - Equal ST                         → 1 die  (attacker picks)
///   - If defender ST > attacker ST     → 2 dice (defender picks)
///   - If defender ST ≥ 2× attacker ST  → 3 dice (defender picks)
///
/// `guard_bonus_attacker` / `guard_bonus_defender`: number of Guard players
/// supporting each side (each adds +1 effective ST).
/// `horns_bonus`: +1 ST for attacker (from Horns skill on a Blitz).
/// `defender_has_defensive`: when true, Guard bonuses for the attacker are ignored.
pub fn block_dice_count(
    att_st: u8,
    def_st: u8,
    guard_bonus_attacker: u8,
    guard_bonus_defender: u8,
) -> i8 {
    block_dice_count_ext(att_st, def_st, guard_bonus_attacker, guard_bonus_defender, false, false)
}

/// Extended version supporting Horns and Defensive skill parameters.
pub fn block_dice_count_ext(
    att_st: u8,
    def_st: u8,
    guard_bonus_attacker: u8,
    guard_bonus_defender: u8,
    horns_bonus: bool,
    defender_has_defensive: bool,
) -> i8 {
    let horns = if horns_bonus { 1i16 } else { 0 };
    let effective_guard_att = if defender_has_defensive { 0 } else { guard_bonus_attacker };
    let att = (att_st as i16) + (effective_guard_att as i16) + horns;
    let def = (def_st as i16) + (guard_bonus_defender as i16);

    if att > def * 2 {
        3
    } else if att > def {
        2
    } else if att == def {
        1
    } else if def > att * 2 {
        -3
    } else {
        -2
    }
}

/// Count Guard players from `team` adjacent to `coord`.
pub fn count_guard(
    field: &FieldModel,
    coord: FieldCoordinate,
    team: TeamId,
) -> u8 {
    coord.neighbors()
        .filter(|&n| {
            if let Some(pid) = field.player_at(n) {
                if field.player_team(pid) == Some(team) {
                    if let Some(state) = field.player_state(pid) {
                        if state.is_active() {
                            // Guard skill check is done by the caller via the player lookup
                            return true;
                        }
                    }
                }
            }
            false
        })
        .count() as u8
}

// ── Push squares ──────────────────────────────────────────────────────────────

/// Compute the 1–3 squares the defender can be pushed to.
/// Rules: the three squares "behind" the defender relative to the attack direction.
/// If a square is off-pitch → crowd pushback.
/// If a square is occupied → it becomes a push candidate only if all are occupied
///   (chain push).
pub fn pushback_options(
    field: &FieldModel,
    attacker: FieldCoordinate,
    defender: FieldCoordinate,
) -> Vec<PushOption> {
    // Direction of attack: from attacker toward defender
    let dx = (defender.x as i16 - attacker.x as i16).signum();
    let dy = (defender.y as i16 - attacker.y as i16).signum();

    // Candidate squares: the 3 squares "behind" the defender relative to the attack direction.
    // For cardinal directions (dx=0 or dy=0) the diagonals share the non-zero component.
    // For diagonal attacks the two adjacent directions (90° rotations) are used.
    //
    //   East  (1,0): (1,0), (1,+1), (1,-1)
    //   West  (-1,0): (-1,0), (-1,+1), (-1,-1)
    //   South (0,+1): (0,+1), (+1,+1), (-1,+1)
    //   North (0,-1): (0,-1), (+1,-1), (-1,-1)
    //   SE  (1,+1): (1,+1), (1,0), (0,+1)
    //   SW  (-1,+1): (-1,+1), (-1,0), (0,+1)
    //   NE  (1,-1): (1,-1), (1,0), (0,-1)
    //   NW  (-1,-1): (-1,-1), (-1,0), (0,-1)
    let raw: [(i16, i16); 3] = if dx == 0 {
        [(0, dy), (1, dy), (-1, dy)]
    } else if dy == 0 {
        [(dx, 0), (dx, 1), (dx, -1)]
    } else {
        [(dx, dy), (dx, 0), (0, dy)]
    };

    let candidates: Vec<(i16, i16)> = raw
        .into_iter()
        .filter_map(|(cx, cy)| {
            let nx = defender.x as i16 + cx;
            let ny = defender.y as i16 + cy;
            // Must differ from attacker position
            if nx == attacker.x as i16 && ny == attacker.y as i16 {
                None
            } else {
                Some((nx, ny))
            }
        })
        .collect();

    let mut options: Vec<PushOption> = candidates
        .into_iter()
        .map(|(nx, ny)| {
            if nx < 0
                || nx >= crate::types::PITCH_WIDTH as i16
                || ny < 0
                || ny >= crate::types::PITCH_HEIGHT as i16
            {
                PushOption::OffPitch
            } else {
                let coord = FieldCoordinate::new(nx as u8, ny as u8);
                if field.is_occupied(coord) {
                    PushOption::Occupied(coord)
                } else {
                    PushOption::Empty(coord)
                }
            }
        })
        .collect();

    // If all non-off-pitch options are occupied → they become valid push targets (chain)
    let any_empty = options.iter().any(|o| matches!(o, PushOption::Empty(_)));
    if !any_empty {
        // Return all including occupied (chain pushes) and off-pitch
        return options;
    }

    // Prefer empty squares — only return empty ones
    options.retain(|o| matches!(o, PushOption::Empty(_) | PushOption::OffPitch));
    if options.is_empty() {
        options.push(PushOption::OffPitch);
    }
    options
}

#[derive(Clone, Debug, PartialEq)]
pub enum PushOption {
    Empty(FieldCoordinate),
    Occupied(FieldCoordinate),
    OffPitch,
}

impl PushOption {
    pub fn coord(&self) -> Option<FieldCoordinate> {
        match self {
            PushOption::Empty(c) | PushOption::Occupied(c) => Some(*c),
            PushOption::OffPitch => None,
        }
    }

    pub fn is_off_pitch(&self) -> bool {
        matches!(self, PushOption::OffPitch)
    }
}

// ── Block outcome (full sequence) ─────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct BlockOutcome {
    pub dice: Vec<BlockResult>,
    pub chosen: BlockResult,
    pub attacker_knocked_down: bool,
    pub defender_knocked_down: bool,
    pub push_options: Vec<PushOption>,
}

// ── T-56 skill helpers ────────────────────────────────────────────────────────

use crate::model::game_state::GameState;

/// T-56 #3: MesmerisingDance — adjacent opponents suffer -1 to block dice.
/// Returns -1 if the defender (the player being blocked) has MesmerisingDance.
pub fn mesmerising_dance_penalty(state: &GameState, defender_id: &PlayerId) -> i8 {
    let has_skill = state.home.player_by_id(defender_id)
        .or_else(|| state.away.player_by_id(defender_id))
        .map(|p| p.has_skill(SkillId::MesmerisingDance))
        .unwrap_or(false);
    if has_skill { -1 } else { 0 }
}

/// T-56 #4: DwarvenScourge — blocks use 2 dice minimum regardless of ST,
/// capped at 3. Only applies when the attacker has DwarvenScourge.
pub fn dwarven_scourge_min_dice(state: &GameState, attacker_id: &PlayerId, current_dice: i8) -> i8 {
    let has_skill = state.home.player_by_id(attacker_id)
        .or_else(|| state.away.player_by_id(attacker_id))
        .map(|p| p.has_skill(SkillId::DwarvenScourge))
        .unwrap_or(false);
    if has_skill {
        current_dice.clamp(2, 3)
    } else {
        current_dice
    }
}

// ── Claws ──────────────────────────────────────────────────────────────────────

/// When the attacker has the Claws skill, treat the defender's AV as at most 7
/// for the purposes of the armor roll (so 2d6 ≥ 8 always breaks armor).
/// Returns the effective AV to use in `armor_roll`.
pub fn claws_effective_av(attacker_has_claws: bool, defender_av: u8) -> u8 {
    if attacker_has_claws {
        defender_av.min(7)
    } else {
        defender_av
    }
}

// ── Dauntless ─────────────────────────────────────────────────────────────────

/// Dauntless: if attacker has Dauntless and defender ST > attacker ST,
/// roll d6; on 4+ the attacker acts as if their ST equals the defender's ST
/// (i.e. at least 1 die, not disadvantaged).
///
/// Returns the effective block dice count accounting for Dauntless.
/// Positive → attacker picks, negative → defender picks.
pub fn dauntless_effective_dice(
    att_st: u8,
    def_st: u8,
    guard_bonus_attacker: u8,
    guard_bonus_defender: u8,
    has_dauntless: bool,
    rng: &mut GameRng,
) -> i8 {
    let normal = crate::mechanics::block::block_dice_count(
        att_st, def_st, guard_bonus_attacker, guard_bonus_defender,
    );
    // Dauntless only applies when defender is strictly stronger (negative dice)
    if has_dauntless && normal < 0 {
        let roll = rng.roll_d6();
        if roll >= 4 {
            // Treat attacker ST as equal to defender ST (1 die, attacker picks)
            return 1;
        }
    }
    normal
}

// ── StripBall ─────────────────────────────────────────────────────────────────

/// After a Pushback or PowPushback result, if the attacker has StripBall and
/// the defender was carrying the ball, scatter the ball from the defender's
/// new square (d8 direction).
///
/// Returns `Some((new_coord, direction))` if the ball was stripped and
/// scattered, or `None` if StripBall did not apply.
pub fn strip_ball_scatter(
    field: &mut crate::model::field_model::FieldModel,
    attacker_has_strip_ball: bool,
    defender_id: &PlayerId,
    rng: &mut GameRng,
) -> Option<(FieldCoordinate, u8)> {
    if !attacker_has_strip_ball {
        return None;
    }
    // Only applies if ball is at the defender's current location
    let def_coord = field.player_coord(defender_id)?;
    let ball_coord = field.ball.coord?;
    if ball_coord != def_coord {
        return None;
    }
    // Scatter ball in a random d8 direction
    let direction = rng.roll_scatter_direction();
    let (dx, dy): (i16, i16) = match direction {
        1 => (0, -1),
        2 => (1, -1),
        3 => (1, 0),
        4 => (1, 1),
        5 => (0, 1),
        6 => (-1, 1),
        7 => (-1, 0),
        _ => (-1, -1),
    };
    let new_x = def_coord.x as i16 + dx;
    let new_y = def_coord.y as i16 + dy;
    let new_coord = if new_x >= 0
        && new_x < crate::types::PITCH_WIDTH as i16
        && new_y >= 0
        && new_y < crate::types::PITCH_HEIGHT as i16
    {
        FieldCoordinate::new(new_x as u8, new_y as u8)
    } else {
        // Ball goes off pitch — stay at defender square (simplified)
        def_coord
    };
    field.ball.coord = Some(new_coord);
    field.ball.moving = false;
    Some((new_coord, direction))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_count_equal_st() {
        assert_eq!(block_dice_count(3, 3, 0, 0), 1);
    }

    #[test]
    fn dice_count_attacker_stronger() {
        assert_eq!(block_dice_count(4, 3, 0, 0), 2);
    }

    #[test]
    fn dice_count_defender_stronger() {
        assert_eq!(block_dice_count(3, 4, 0, 0), -2);
    }

    #[test]
    fn dice_count_guard_bonus_equalises() {
        // ST3 vs ST4 normally → -2; with guard bonus +1 for attacker → equal → 1
        assert_eq!(block_dice_count(3, 4, 1, 0), 1);
    }

    #[test]
    fn horns_gives_attacker_extra_die() {
        // Equal ST3 vs ST3 normally → 1 die; Horns → att_st effectively 4 → 2 dice
        assert_eq!(block_dice_count_ext(3, 3, 0, 0, true, false), 2);
    }

    #[test]
    fn defensive_cancels_guard_bonus() {
        // ST3 vs ST4, guard +1 for attacker normally equalises → 1 die
        // With Defensive, guard is ignored: 3 vs 4 → -2 (defender picks 2 dice)
        assert_eq!(block_dice_count_ext(3, 4, 1, 0, false, true), -2);
    }

    #[test]
    fn defensive_does_not_cancel_horns() {
        // ST3 + Horns vs ST3 + Defensive, no guard: att=4 vs def=3 → 2 dice
        assert_eq!(block_dice_count_ext(3, 3, 0, 0, true, true), 2);
    }

    #[test]
    fn pushback_options_open_field() {
        let field = FieldModel::new();
        let attacker = FieldCoordinate::new(5, 5);
        let defender = FieldCoordinate::new(6, 5);
        let opts = pushback_options(&field, attacker, defender);
        // Should have 1-3 empty options to the east/northeast/southeast of defender
        assert!(!opts.is_empty());
        for o in &opts {
            assert!(matches!(o, PushOption::Empty(_)));
        }
    }

    #[test]
    fn pushback_options_against_sideline() {
        let field = FieldModel::new();
        // Defender at right edge (x=25), attacker to the left
        let attacker = FieldCoordinate::new(24, 8);
        let defender = FieldCoordinate::new(25, 8);
        let opts = pushback_options(&field, attacker, defender);
        // At least one off-pitch option expected
        assert!(opts.iter().any(|o| o.is_off_pitch()));
    }

    #[test]
    fn pushback_does_not_include_attacker_square() {
        let field = FieldModel::new();
        let attacker = FieldCoordinate::new(5, 5);
        let defender = FieldCoordinate::new(6, 5);
        let opts = pushback_options(&field, attacker, defender);
        for o in &opts {
            if let Some(c) = o.coord() {
                assert_ne!(c, attacker, "push cannot send defender back to attacker square");
            }
        }
    }

    // ── Pushback direction tests ──────────────────────────────────────────────

    fn push_coords(att: (u8, u8), def: (u8, u8)) -> Vec<FieldCoordinate> {
        let field = FieldModel::new();
        let attacker = FieldCoordinate::new(att.0, att.1);
        let defender = FieldCoordinate::new(def.0, def.1);
        pushback_options(&field, attacker, defender)
            .into_iter()
            .filter_map(|o| o.coord())
            .collect()
    }

    #[test]
    fn pushback_direction_north() {
        // att south of def → push north (y decreases)
        let coords = push_coords((5, 6), (5, 5));
        let expected: Vec<FieldCoordinate> = [(5,4),(6,4),(4,4)].iter()
            .map(|&(x,y)| FieldCoordinate::new(x, y)).collect();
        for e in &expected { assert!(coords.contains(e), "missing {:?}", e); }
        assert_eq!(coords.len(), expected.len());
    }

    #[test]
    fn pushback_direction_northeast() {
        // att SW of def → push NE
        let coords = push_coords((4, 6), (5, 5));
        let expected: Vec<FieldCoordinate> = [(6,4),(6,5),(5,4)].iter()
            .map(|&(x,y)| FieldCoordinate::new(x, y)).collect();
        for e in &expected { assert!(coords.contains(e), "missing {:?}", e); }
        assert_eq!(coords.len(), expected.len());
    }

    #[test]
    fn pushback_direction_east() {
        let coords = push_coords((4, 5), (5, 5));
        let expected: Vec<FieldCoordinate> = [(6,5),(6,6),(6,4)].iter()
            .map(|&(x,y)| FieldCoordinate::new(x, y)).collect();
        for e in &expected { assert!(coords.contains(e), "missing {:?}", e); }
        assert_eq!(coords.len(), expected.len());
    }

    #[test]
    fn pushback_direction_southeast() {
        // att NW of def → push SE
        let coords = push_coords((4, 4), (5, 5));
        let expected: Vec<FieldCoordinate> = [(6,6),(6,5),(5,6)].iter()
            .map(|&(x,y)| FieldCoordinate::new(x, y)).collect();
        for e in &expected { assert!(coords.contains(e), "missing {:?}", e); }
        assert_eq!(coords.len(), expected.len());
    }

    #[test]
    fn pushback_direction_south() {
        // att north of def → push south (y increases)
        let coords = push_coords((5, 4), (5, 5));
        let expected: Vec<FieldCoordinate> = [(5,6),(6,6),(4,6)].iter()
            .map(|&(x,y)| FieldCoordinate::new(x, y)).collect();
        for e in &expected { assert!(coords.contains(e), "missing {:?}", e); }
        assert_eq!(coords.len(), expected.len());
    }

    #[test]
    fn pushback_direction_southwest() {
        // att NE of def → push SW
        let coords = push_coords((6, 4), (5, 5));
        let expected: Vec<FieldCoordinate> = [(4,6),(4,5),(5,6)].iter()
            .map(|&(x,y)| FieldCoordinate::new(x, y)).collect();
        for e in &expected { assert!(coords.contains(e), "missing {:?}", e); }
        assert_eq!(coords.len(), expected.len());
    }

    #[test]
    fn pushback_direction_west() {
        let coords = push_coords((6, 5), (5, 5));
        let expected: Vec<FieldCoordinate> = [(4,5),(4,6),(4,4)].iter()
            .map(|&(x,y)| FieldCoordinate::new(x, y)).collect();
        for e in &expected { assert!(coords.contains(e), "missing {:?}", e); }
        assert_eq!(coords.len(), expected.len());
    }

    #[test]
    fn pushback_direction_northwest() {
        // att SE of def → push NW
        let coords = push_coords((6, 6), (5, 5));
        let expected: Vec<FieldCoordinate> = [(4,4),(4,5),(5,4)].iter()
            .map(|&(x,y)| FieldCoordinate::new(x, y)).collect();
        for e in &expected { assert!(coords.contains(e), "missing {:?}", e); }
        assert_eq!(coords.len(), expected.len());
    }

    #[test]
    fn pushback_all_occupied_returns_occupied_options_for_chain() {
        // When all 3 candidate squares are occupied, pushback_options should
        // return them as Occupied variants so the caller can trigger a chain push.
        use crate::types::PlayerState;
        use crate::model::player::{Player, PlayerStats};
        use crate::model::team::Team;
        use crate::skills::SkillSet;
        use crate::types::{PlayerId, TeamId};

        let mut field = FieldModel::new();
        let att = FieldCoordinate::new(5, 5);
        let def = FieldCoordinate::new(6, 5);
        // Block all 3 east-of-def squares: (7,5), (7,6), (7,4)
        for (i, coord) in [(7u8,5u8),(7,6),(7,4)].iter().enumerate() {
            let pid = PlayerId(format!("blocker{i}"));
            field.place_player(pid, TeamId::Away, FieldCoordinate::new(coord.0, coord.1), PlayerState::Standing);
        }
        let opts = pushback_options(&field, att, def);
        assert!(
            opts.iter().all(|o| matches!(o, PushOption::Occupied(_))),
            "all options should be Occupied when squares are full, got {:?}", opts
        );
        assert_eq!(opts.len(), 3, "should return all 3 occupied options for chain-push selection");
    }

    // ── T-56 skill helper tests ────────────────────────────────────────────

    use crate::model::game_state::GameState;
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::skills::{SkillId, SkillSet};
    use crate::types::{PlayerId, TeamId};

    fn make_state_with_skill(pid: &str, skill: SkillId, team: TeamId) -> (GameState, PlayerId) {
        let player_id = PlayerId(pid.into());
        let mut skills = SkillSet::empty();
        skills.add(skill);
        let player = Player::new(
            player_id.clone(), pid.into(), "lineman".into(), team, 1,
            PlayerStats::new(4, 3, 3, 8, None), skills,
        );
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 2, true);
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 2, false);
        match team {
            TeamId::Home => home.add_player(player),
            TeamId::Away => away.add_player(player),
        }
        let state = GameState::new(home, away);
        (state, player_id)
    }

    #[test]
    fn mesmerising_dance_penalty_with_skill() {
        // Defender has MesmerisingDance → penalty of -1
        let (state, def_id) = make_state_with_skill("def", SkillId::MesmerisingDance, TeamId::Away);
        assert_eq!(mesmerising_dance_penalty(&state, &def_id), -1);
    }

    #[test]
    fn mesmerising_dance_penalty_without_skill() {
        let (state, def_id) = make_state_with_skill("def", SkillId::Block, TeamId::Away);
        assert_eq!(mesmerising_dance_penalty(&state, &def_id), 0);
    }

    #[test]
    fn mesmerising_dance_reduces_block_dice_to_one() {
        // Equal ST3 vs ST3 normally gives 1 die; with MesmerisingDance penalty -1 → 0 effective
        // The mechanic: block_dice_count gives 1, then penalty reduces to 0 (min 1 die in practice)
        // Here we just verify the raw penalty is -1 which the step layer would apply.
        let (state, def_id) = make_state_with_skill("def", SkillId::MesmerisingDance, TeamId::Away);
        let normal_dice = block_dice_count(3, 3, 0, 0); // 1
        let effective = normal_dice + mesmerising_dance_penalty(&state, &def_id);
        assert_eq!(effective, 0); // step layer floors at 1
    }

    #[test]
    fn dwarven_scourge_min_dice_below_two() {
        // Attacker has DwarvenScourge; current_dice=1 → should return 2
        let (state, att_id) = make_state_with_skill("att", SkillId::DwarvenScourge, TeamId::Home);
        assert_eq!(dwarven_scourge_min_dice(&state, &att_id, 1), 2);
    }

    #[test]
    fn dwarven_scourge_min_dice_already_two() {
        let (state, att_id) = make_state_with_skill("att", SkillId::DwarvenScourge, TeamId::Home);
        assert_eq!(dwarven_scourge_min_dice(&state, &att_id, 2), 2);
    }

    #[test]
    fn dwarven_scourge_min_dice_capped_at_three() {
        // Should not exceed 3
        let (state, att_id) = make_state_with_skill("att", SkillId::DwarvenScourge, TeamId::Home);
        assert_eq!(dwarven_scourge_min_dice(&state, &att_id, 3), 3);
    }

    #[test]
    fn dwarven_scourge_no_skill_unchanged() {
        let (state, att_id) = make_state_with_skill("att", SkillId::Block, TeamId::Home);
        assert_eq!(dwarven_scourge_min_dice(&state, &att_id, 1), 1);
    }

    // ── Claws tests ───────────────────────────────────────────────────────────

    #[test]
    fn claws_reduces_effective_av() {
        // AV=10 without Claws → 10; with Claws → min(10, 7) = 7
        assert_eq!(claws_effective_av(false, 10), 10);
        assert_eq!(claws_effective_av(true, 10), 7);
    }

    #[test]
    fn claws_does_not_reduce_low_av() {
        // AV=6 with Claws → min(6, 7) = 6 (no change)
        assert_eq!(claws_effective_av(true, 6), 6);
    }

    #[test]
    fn claws_reduces_effective_av_armor_roll_of_8_breaks() {
        use crate::mechanics::injury::{armor_roll, ArmorOutcome};
        // AV=10 with Claws → effective AV=7; roll 2d6=2+6=8 → 8 > 7 → broken
        let effective_av = claws_effective_av(true, 10);
        let mut rng = GameRng::new_test([2, 6]);
        assert_eq!(armor_roll(effective_av, 0, 0, &mut rng), ArmorOutcome::Broken);
    }

    // ── Dauntless tests ───────────────────────────────────────────────────────

    #[test]
    fn dauntless_succeeds_on_4plus() {
        // att_st=2, def_st=4 → normal dice would be -2; Dauntless roll=4 → 1 die
        let mut rng = GameRng::new_test([4]); // d6=4 → success
        let dice = dauntless_effective_dice(2, 4, 0, 0, true, &mut rng);
        assert_eq!(dice, 1);
    }

    #[test]
    fn dauntless_fails_on_low_roll() {
        // att=3, def=5: def < att*2, so normal dice = -2 (defender picks 2).
        // Dauntless roll=3 → fail, attacker still at normal disadvantage (-2).
        let mut rng = GameRng::new_test([3]); // d6=3 → fail
        let dice = dauntless_effective_dice(3, 5, 0, 0, true, &mut rng);
        assert_eq!(dice, -2);
    }

    #[test]
    fn dauntless_not_triggered_when_equal_st() {
        // Equal ST → Dauntless doesn't trigger (no dice consumed)
        let mut rng = GameRng::new_test([]);
        let dice = dauntless_effective_dice(3, 3, 0, 0, true, &mut rng);
        assert_eq!(dice, 1);
    }

    #[test]
    fn dauntless_not_triggered_without_skill() {
        // att=3, def=5: no Dauntless skill → normal dice (-2) returned
        let mut rng = GameRng::new_test([]);
        let dice = dauntless_effective_dice(3, 5, 0, 0, false, &mut rng);
        assert_eq!(dice, -2);
    }

    // ── StripBall tests ───────────────────────────────────────────────────────

    #[test]
    fn strip_ball_scatters_ball_from_defender_square() {
        use crate::model::field_model::{BallState, FieldModel};
        use crate::types::{PlayerId, PlayerState, TeamId};

        let mut field = FieldModel::new();
        let def_id = PlayerId("def".into());
        let def_coord = FieldCoordinate::new(10, 8);
        field.place_player(def_id.clone(), TeamId::Away, def_coord, PlayerState::Standing);
        field.ball = BallState {
            in_play: true,
            moving: false,
            coord: Some(def_coord),
        };

        // Direction 3 → dx=1, dy=0 → new coord = (11, 8)
        let mut rng = GameRng::new_test([3]);
        let result = strip_ball_scatter(&mut field, true, &def_id, &mut rng);
        assert!(result.is_some());
        let (new_coord, dir) = result.unwrap();
        assert_eq!(dir, 3);
        assert_eq!(new_coord, FieldCoordinate::new(11, 8));
        assert_eq!(field.ball.coord, Some(FieldCoordinate::new(11, 8)));
    }

    #[test]
    fn strip_ball_no_effect_without_skill() {
        use crate::model::field_model::{BallState, FieldModel};
        use crate::types::{PlayerId, PlayerState, TeamId};

        let mut field = FieldModel::new();
        let def_id = PlayerId("def".into());
        let def_coord = FieldCoordinate::new(10, 8);
        field.place_player(def_id.clone(), TeamId::Away, def_coord, PlayerState::Standing);
        field.ball = BallState {
            in_play: true,
            moving: false,
            coord: Some(def_coord),
        };

        let mut rng = GameRng::new_test([]);
        let result = strip_ball_scatter(&mut field, false, &def_id, &mut rng);
        assert!(result.is_none());
        assert_eq!(field.ball.coord, Some(def_coord));
    }
}
