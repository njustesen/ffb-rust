use crate::rng::GameRng;
use crate::skills::SkillId;
use crate::types::{FieldCoordinate, PassRange, PlayerState};
use crate::model::game_state::GameState;
use crate::types::PlayerId;

// ── Pass result ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum PassResult {
    /// Pass landed accurately.
    Accurate,
    /// Pass was inaccurate — scatter from target.
    Inaccurate,
    /// Passer fumbled — ball scatters from passer square.
    Fumble,
}

// ── TheBallista skill ─────────────────────────────────────────────────────────

/// Extends pass range by one category (TheBallista skill).
/// HandOff → Short, Short → Long, Long → LongBomb, LongBomb → LongBomb.
pub fn extended_pass_range(base_range: PassRange) -> PassRange {
    match base_range {
        PassRange::HandOff => PassRange::Short,
        PassRange::Short => PassRange::Long,
        PassRange::Long => PassRange::LongBomb,
        PassRange::LongBomb => PassRange::LongBomb,
    }
}

// ── KrumpAndSmash skill ───────────────────────────────────────────────────────

/// Returns 1 (AV penalty) if the attacker has KrumpAndSmash, else 0.
pub fn krump_av_penalty(state: &GameState, attacker_id: &PlayerId) -> u8 {
    use crate::skills::SkillId;
    state.field.player_team(attacker_id)
        .and_then(|team| state.team(team).player_by_id(attacker_id))
        .map(|p| if p.has_skill(SkillId::KrumpAndSmash) { 1 } else { 0 })
        .unwrap_or(0)
}

// ── WisdomOfTheWhiteDwarf skill ───────────────────────────────────────────────

/// Returns the AG of the WisdomOfTheWhiteDwarf player if they are adjacent
/// to `adjacent_player_id`, else None.
pub fn wisdom_ag_bonus(state: &GameState, wisdom_player_id: &PlayerId, adjacent_player_id: &PlayerId) -> Option<u8> {
    use crate::skills::SkillId;
    let wisdom_team = state.field.player_team(wisdom_player_id)?;
    let wisdom_player = state.team(wisdom_team).player_by_id(wisdom_player_id)?;
    if !wisdom_player.has_skill(SkillId::WisdomOfTheWhiteDwarf) {
        return None;
    }
    let wisdom_coord = state.field.player_coord(wisdom_player_id)?;
    let adjacent_coord = state.field.player_coord(adjacent_player_id)?;
    let is_adjacent = wisdom_coord.neighbors().any(|n| n == adjacent_coord);
    if is_adjacent {
        Some(wisdom_player.effective_ag())
    } else {
        None
    }
}

// ── Pass range ────────────────────────────────────────────────────────────────

/// Classify pass range from passer to target.
/// Chebyshev (king-move) distance determines range:
///   ≤1 = HandOff, 2–3 = Short, 4–6 = Long, >6 = LongBomb
pub fn pass_range_from_coords(from: FieldCoordinate, to: FieldCoordinate) -> PassRange {
    let dx = (from.x as i16 - to.x as i16).unsigned_abs();
    let dy = (from.y as i16 - to.y as i16).unsigned_abs();
    let dist = dx.max(dy) as u8;
    PassRange::for_distance(dist)
}

// ── Pass minimum roll ─────────────────────────────────────────────────────────

/// Minimum d6 roll needed to complete a pass.
/// PA=None → always fails (fumble on any non-6).
/// Base: `max(2, 4 - (pa - 1))` = `max(2, 5 - pa)`.
/// Range modifier: Quick=0, Short=+1, Long=+2, LongBomb=+3, HandOff=0.
/// `tz_penalty`: number of opposing tackle zones on the passer's square (each adds +1).
/// `nerves_of_steel`: if true, the tz_penalty is ignored.
/// Result clamped to 2–6 (roll of 1 is always a fumble regardless).
pub fn pass_min_roll(pa: Option<u8>, range: PassRange) -> u8 {
    pass_min_roll_with_tz(pa, range, 0, false)
}

/// Like `pass_min_roll`, but with an explicit TZ penalty and NervesOfSteel flag.
pub fn pass_min_roll_with_tz(pa: Option<u8>, range: PassRange, tz_penalty: u8, nerves_of_steel: bool) -> u8 {
    let pa = match pa {
        Some(v) => v,
        None => return 7, // impossible — PA-less players always fumble
    };
    let base = (5u8).saturating_sub(pa).max(2);
    let modifier: u8 = match range {
        PassRange::HandOff => 0,
        PassRange::Short => 1,
        PassRange::Long => 2,
        PassRange::LongBomb => 3,
    };
    let tz_mod = if nerves_of_steel { 0 } else { tz_penalty };
    (base + modifier + tz_mod).clamp(2, 6)
}

// ── Pass execution ────────────────────────────────────────────────────────────

/// Roll for a pass attempt.
/// Roll of 1 → Fumble regardless of modifiers.
/// Roll ≥ min_roll → Accurate.
/// Otherwise → Inaccurate.
pub fn pass_roll(pa: Option<u8>, range: PassRange, rng: &mut GameRng) -> PassResult {
    pass_roll_with_tz(pa, range, 0, false, rng)
}

/// Roll for a pass attempt with TZ penalty and NervesOfSteel support.
/// `tz_penalty`: opposing TZs on the passer's square.
/// `nerves_of_steel`: if true, tz_penalty is ignored.
pub fn pass_roll_with_tz(pa: Option<u8>, range: PassRange, tz_penalty: u8, nerves_of_steel: bool, rng: &mut GameRng) -> PassResult {
    let roll = rng.roll_d6();
    if roll == 1 {
        return PassResult::Fumble;
    }
    let min_roll = pass_min_roll_with_tz(pa, range, tz_penalty, nerves_of_steel);
    if min_roll > 6 {
        // PA=None players: roll 1 = fumble (handled above), 2-5 = inaccurate, 6 = accurate
        if roll == 6 {
            PassResult::Accurate
        } else {
            PassResult::Inaccurate
        }
    } else if roll >= min_roll {
        PassResult::Accurate
    } else {
        PassResult::Inaccurate
    }
}

// ── Scatter calculation ───────────────────────────────────────────────────────

/// Compute scatter landing square for an inaccurate pass, fumble, or ball bounce.
/// Implements BB2025 crowd throw-in when the ball exits the pitch.
pub fn pass_scatter_coord(from: FieldCoordinate, rng: &mut GameRng) -> FieldCoordinate {
    use crate::pathfinding::scatter_with_crowd_throw_in;
    let direction = rng.roll_scatter_direction();
    let distance = rng.roll_scatter_distance();
    scatter_with_crowd_throw_in(from, direction, distance, rng)
}

// ── Bombardier — throw_bomb ───────────────────────────────────────────────────

/// Result of a bomb throw.
#[derive(Clone, Debug, PartialEq)]
pub struct BombResult {
    /// Players knocked prone by the bomb (landing square + Chebyshev-1 radius).
    pub hit_players: Vec<PlayerId>,
    /// Square where the bomb landed.
    pub landing_square: FieldCoordinate,
}

/// Throw a bomb from `thrower_id` at `target`.
/// The bomb always scatters (d8 direction, d6 distance clamped 0–2 via `% 3`).
/// On landing, all players within Chebyshev distance 1 are knocked prone.
/// No armor roll from the bomb itself — just prone.
pub fn throw_bomb(
    state: &mut GameState,
    _thrower_id: &PlayerId,
    target: FieldCoordinate,
    rng: &mut GameRng,
) -> BombResult {
    // Bomb always scatters from target
    let landing = pass_scatter_coord(target, rng);

    // Knock down all players in Chebyshev-1 radius of the landing square
    let mut hit_players = Vec::new();
    let affected: Vec<FieldCoordinate> = {
        let mut squares = vec![landing];
        for n in landing.neighbors() {
            squares.push(n);
        }
        squares
    };

    for sq in &affected {
        if let Some(pid) = state.field.player_at(*sq).cloned() {
            if state.field.player_state(&pid).map(|s| s.is_on_pitch()).unwrap_or(false) {
                state.field.set_player_state(&pid, PlayerState::Prone);
                hit_players.push(pid);
            }
        }
    }

    BombResult { hit_players, landing_square: landing }
}

// ── PileDriver ────────────────────────────────────────────────────────────────

/// Returns true if the attacker has PileDriver and there is at least one Prone
/// opponent adjacent to them (i.e., they just knocked someone down).
pub fn pile_driver_available(state: &GameState, attacker_id: &PlayerId) -> bool {
    let att_team = match state.field.player_team(attacker_id) {
        Some(t) => t,
        None => return false,
    };
    let has_pile_driver = state.team(att_team)
        .player_by_id(attacker_id)
        .map(|p| p.has_skill(SkillId::PileDriver))
        .unwrap_or(false);
    if !has_pile_driver {
        return false;
    }
    let att_coord = match state.field.player_coord(attacker_id) {
        Some(c) => c,
        None => return false,
    };
    let opp_team = att_team.opponent();
    att_coord.neighbors().any(|n| {
        if let Some(pid) = state.field.player_at(n) {
            if state.field.player_team(pid) == Some(opp_team) {
                return state.field.player_state(pid) == Some(PlayerState::Prone);
            }
        }
        false
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::GameRng;
    use crate::types::PassRange;

    #[test]
    fn pass_min_roll_short_pa3() {
        // PA3: base = max(2, 5-3) = 2; Short +1 → 3
        assert_eq!(pass_min_roll(Some(3), PassRange::Short), 3);
    }

    #[test]
    fn pass_min_roll_long_pa4() {
        // PA4: base = max(2, 5-4) = 2; Long +2 → 4
        assert_eq!(pass_min_roll(Some(4), PassRange::Long), 4);
    }

    #[test]
    fn pass_min_roll_pa_none_impossible() {
        assert_eq!(pass_min_roll(None, PassRange::Short), 7);
    }

    #[test]
    fn pass_min_roll_clamped_to_6() {
        // PA1: base = max(2, 5-1)=4; LongBomb +3 → 7 clamped to 6
        assert_eq!(pass_min_roll(Some(1), PassRange::LongBomb), 6);
    }

    #[test]
    fn pass_roll_accurate_on_high() {
        // PA3, Short → min 3; roll 5 → Accurate
        let mut rng = GameRng::new_test([5]);
        assert_eq!(pass_roll(Some(3), PassRange::Short, &mut rng), PassResult::Accurate);
    }

    #[test]
    fn pass_roll_fumble_on_1() {
        let mut rng = GameRng::new_test([1]);
        assert_eq!(pass_roll(Some(3), PassRange::Short, &mut rng), PassResult::Fumble);
    }

    #[test]
    fn pass_roll_inaccurate_on_low() {
        // PA3, Short → min 3; roll 2 → Inaccurate
        let mut rng = GameRng::new_test([2]);
        assert_eq!(pass_roll(Some(3), PassRange::Short, &mut rng), PassResult::Inaccurate);
    }

    #[test]
    fn scatter_stays_in_bounds() {
        // Corner (0,0) + NW direction → exits pitch → crowd throw-in triggered
        // Provide extra rolls for throw-in: direction d6=4 (S straight), distance d6=2 (1 sq)
        let corner = FieldCoordinate::new(0, 0);
        let mut rng = GameRng::new_test([1, 3, 4, 2]); // scatter dir=1, dist=3, throw_in_dir=4, throw_in_dist=2
        let result = pass_scatter_coord(corner, &mut rng);
        assert!(result.is_valid());
    }

    #[test]
    fn pass_range_classification() {
        let a = FieldCoordinate::new(5, 5);
        assert_eq!(pass_range_from_coords(a, FieldCoordinate::new(5, 6)), PassRange::HandOff);
        assert_eq!(pass_range_from_coords(a, FieldCoordinate::new(5, 8)), PassRange::Short);
        assert_eq!(pass_range_from_coords(a, FieldCoordinate::new(5, 11)), PassRange::Long);
        assert_eq!(pass_range_from_coords(a, FieldCoordinate::new(5, 16)), PassRange::LongBomb);
    }

    // ── TheBallista ───────────────────────────────────────────────────────────

    #[test]
    fn ballista_extends_handoff_to_short() {
        assert_eq!(extended_pass_range(PassRange::HandOff), PassRange::Short);
    }

    #[test]
    fn ballista_extends_short_to_long() {
        assert_eq!(extended_pass_range(PassRange::Short), PassRange::Long);
    }

    #[test]
    fn ballista_extends_long_to_longbomb() {
        assert_eq!(extended_pass_range(PassRange::Long), PassRange::LongBomb);
    }

    #[test]
    fn ballista_longbomb_stays_longbomb() {
        assert_eq!(extended_pass_range(PassRange::LongBomb), PassRange::LongBomb);
    }

    // ── KrumpAndSmash ─────────────────────────────────────────────────────────

    #[test]
    fn krump_av_penalty_with_skill() {
        use crate::model::game_state::GameState;
        use crate::model::player::{Player, PlayerStats};
        use crate::model::team::Team;
        use crate::skills::{SkillId, SkillSet};
        use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

        let pid = PlayerId("krump_p".into());
        let skills: SkillSet = [SkillId::KrumpAndSmash].into_iter().collect();
        let mut home = Team::new("h".into(), "Home".into(), "Orc".into(), 3, false);
        home.add_player(Player::new(
            pid.clone(), "Krumper".into(), "blitzer".into(), TeamId::Home, 1,
            PlayerStats::new(6, 4, 3, 9, None), skills,
        ));
        let away = Team::new("a".into(), "Away".into(), "Human".into(), 3, true);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        assert_eq!(krump_av_penalty(&state, &pid), 1);
    }

    #[test]
    fn krump_av_penalty_without_skill() {
        use crate::model::game_state::GameState;
        use crate::model::player::{Player, PlayerStats};
        use crate::model::team::Team;
        use crate::skills::SkillSet;
        use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

        let pid = PlayerId("normal_p".into());
        let mut home = Team::new("h".into(), "Home".into(), "Orc".into(), 3, false);
        home.add_player(Player::new(
            pid.clone(), "Normal".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        ));
        let away = Team::new("a".into(), "Away".into(), "Human".into(), 3, true);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        assert_eq!(krump_av_penalty(&state, &pid), 0);
    }

    // ── WisdomOfTheWhiteDwarf ─────────────────────────────────────────────────

    #[test]
    fn wisdom_ag_bonus_returns_ag_when_adjacent() {
        use crate::model::game_state::GameState;
        use crate::model::player::{Player, PlayerStats};
        use crate::model::team::Team;
        use crate::skills::{SkillId, SkillSet};
        use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

        let wisdom_pid = PlayerId("wisdom_p".into());
        let adj_pid = PlayerId("adj_p".into());
        let wisdom_skills: SkillSet = [SkillId::WisdomOfTheWhiteDwarf].into_iter().collect();

        let mut home = Team::new("h".into(), "Home".into(), "Dwarf".into(), 3, true);
        home.add_player(Player::new(
            wisdom_pid.clone(), "Wise Dwarf".into(), "captain".into(), TeamId::Home, 1,
            PlayerStats::new(5, 3, 4, 9, None), wisdom_skills,
        ));
        home.add_player(Player::new(
            adj_pid.clone(), "Ally".into(), "lineman".into(), TeamId::Home, 2,
            PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        ));

        let away = Team::new("a".into(), "Away".into(), "Human".into(), 3, true);
        let mut state = GameState::new(home, away);
        let wisdom_coord = FieldCoordinate::new(5, 5);
        let adj_coord = FieldCoordinate::new(5, 6); // directly adjacent
        state.field.place_player(wisdom_pid.clone(), TeamId::Home, wisdom_coord, PlayerState::Standing);
        state.field.place_player(adj_pid.clone(), TeamId::Home, adj_coord, PlayerState::Standing);

        // Wisdom player has AG4; adjacent_player should get Some(4)
        assert_eq!(wisdom_ag_bonus(&state, &wisdom_pid, &adj_pid), Some(4));
    }

    #[test]
    fn wisdom_ag_bonus_none_when_not_adjacent() {
        use crate::model::game_state::GameState;
        use crate::model::player::{Player, PlayerStats};
        use crate::model::team::Team;
        use crate::skills::{SkillId, SkillSet};
        use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

        let wisdom_pid = PlayerId("wisdom_p2".into());
        let far_pid = PlayerId("far_p".into());
        let wisdom_skills: SkillSet = [SkillId::WisdomOfTheWhiteDwarf].into_iter().collect();

        let mut home = Team::new("h".into(), "Home".into(), "Dwarf".into(), 3, true);
        home.add_player(Player::new(
            wisdom_pid.clone(), "Wise Dwarf".into(), "captain".into(), TeamId::Home, 1,
            PlayerStats::new(5, 3, 4, 9, None), wisdom_skills,
        ));
        home.add_player(Player::new(
            far_pid.clone(), "Far Away".into(), "lineman".into(), TeamId::Home, 2,
            PlayerStats::new(6, 3, 3, 8, None), SkillSet::empty(),
        ));

        let away = Team::new("a".into(), "Away".into(), "Human".into(), 3, true);
        let mut state = GameState::new(home, away);
        state.field.place_player(wisdom_pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(far_pid.clone(), TeamId::Home, FieldCoordinate::new(10, 10), PlayerState::Standing);

        assert_eq!(wisdom_ag_bonus(&state, &wisdom_pid, &far_pid), None);
    }

    // ── throw_bomb (Bombardier) ───────────────────────────────────────────────

    fn make_two_player_state(
        att_pid_s: &str, att_skills: crate::skills::SkillSet,
        def_pid_s: &str, def_skills: crate::skills::SkillSet,
        att_coord: FieldCoordinate, def_coord: FieldCoordinate,
    ) -> (GameState, PlayerId, PlayerId) {
        use crate::model::player::{Player, PlayerStats};
        use crate::model::team::Team;
        use crate::types::TeamId;

        let att_pid = PlayerId(att_pid_s.into());
        let def_pid = PlayerId(def_pid_s.into());

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            att_pid.clone(), att_pid_s.into(), "bombardier".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 3, 8, Some(4)), att_skills,
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            def_pid.clone(), def_pid_s.into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 3, 3, 9, None), def_skills,
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(att_pid.clone(), TeamId::Home, att_coord, PlayerState::Standing);
        state.field.place_player(def_pid.clone(), TeamId::Away, def_coord, PlayerState::Standing);
        state.home_is_active = true;
        (state, att_pid, def_pid)
    }

    #[test]
    fn throw_bomb_knocks_down_adjacent_player() {
        // Thrower at (5,5), target at (10,5), defender at (10,5).
        // Scatter: direction=5 (east +1x), distance=1 → lands at (11,5).
        // Defender at (10,5) is within Chebyshev-1 of (11,5) → knocked prone.
        let att = FieldCoordinate::new(5, 5);
        let target = FieldCoordinate::new(10, 5);
        let def_coord = FieldCoordinate::new(10, 5); // same as target

        // We need scatter to land near the defender.
        // pass_scatter_coord(target, rng) rolls d8 direction then d6 distance.
        // Direction 5 (SE) with distance 1: x+1, y+1 → (11,6)
        // Let's choose a scatter that lands ON (10,5) = direction 5 (south = y+1), dist = 0
        // Actually roll_scatter_distance rolls d6 (1-6), so distance is always ≥1.
        // Direction 1 (NW): dx=-1 dy=-1; distance=1 → from (10,5) → (9,4)
        // Defender at (10,5) — Chebyshev dist from (9,4): max(1,1)=1 → hit!
        let att_skills: crate::skills::SkillSet = [SkillId::Bombardier].into_iter().collect();
        let def_skills = crate::skills::SkillSet::empty();
        let (mut state, att_pid, def_pid) = make_two_player_state(
            "att", att_skills, "def", def_skills, att, def_coord,
        );

        // direction=1, distance=1 → scatter from (10,5) to (9,4)
        let mut rng = GameRng::new_test([1, 1]);
        let result = throw_bomb(&mut state, &att_pid, target, &mut rng);

        // Defender at (10,5) should be within Chebyshev-1 of (9,4): dx=1, dy=1 → max=1
        assert!(result.hit_players.contains(&def_pid),
            "defender should be hit; hit: {:?}, landing: {:?}", result.hit_players, result.landing_square);
        assert_eq!(state.field.player_state(&def_pid), Some(PlayerState::Prone));
    }

    #[test]
    fn throw_bomb_landing_square_is_valid() {
        let att_skills: crate::skills::SkillSet = [SkillId::Bombardier].into_iter().collect();
        let def_skills = crate::skills::SkillSet::empty();
        let (mut state, att_pid, _def_pid) = make_two_player_state(
            "att2", att_skills, "def2", def_skills,
            FieldCoordinate::new(5, 5), FieldCoordinate::new(8, 8),
        );
        let target = FieldCoordinate::new(8, 8);
        let mut rng = GameRng::new_test([3, 2]);
        let result = throw_bomb(&mut state, &att_pid, target, &mut rng);
        assert!(result.landing_square.is_valid());
    }

    // ── pile_driver_available ─────────────────────────────────────────────────

    #[test]
    fn pile_driver_available_when_adjacent_prone_opponent() {
        let att_skills: crate::skills::SkillSet = [SkillId::PileDriver].into_iter().collect();
        let def_skills = crate::skills::SkillSet::empty();
        let (mut state, att_pid, def_pid) = make_two_player_state(
            "att3", att_skills, "def3", def_skills,
            FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5),
        );
        // Make defender prone
        state.field.set_player_state(&def_pid, PlayerState::Prone);
        assert!(pile_driver_available(&state, &att_pid));
    }

    #[test]
    fn pile_driver_not_available_when_opponent_standing() {
        let att_skills: crate::skills::SkillSet = [SkillId::PileDriver].into_iter().collect();
        let def_skills = crate::skills::SkillSet::empty();
        let (state, att_pid, _def_pid) = make_two_player_state(
            "att4", att_skills, "def4", def_skills,
            FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5),
        );
        // Defender is still standing
        assert!(!pile_driver_available(&state, &att_pid));
    }

    #[test]
    fn pile_driver_not_available_without_skill() {
        let att_skills = crate::skills::SkillSet::empty();
        let def_skills = crate::skills::SkillSet::empty();
        let (mut state, att_pid, def_pid) = make_two_player_state(
            "att5", att_skills, "def5", def_skills,
            FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5),
        );
        state.field.set_player_state(&def_pid, PlayerState::Prone);
        assert!(!pile_driver_available(&state, &att_pid));
    }
}
