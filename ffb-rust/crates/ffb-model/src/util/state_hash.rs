use crate::model::game::Game;
use crate::enums::{
    PS_STANDING, PS_MOVING, PS_PRONE, PS_STUNNED,
    PS_KNOCKED_OUT, PS_BADLY_HURT, PS_SERIOUS_INJURY, PS_RIP,
};

/// FNV-1a 64-bit hash — matches Java's ParityRunner.fnv1a64().
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

/// The raw canonical state string before hashing (for diagnostics/testing).
pub fn state_string(game: &Game) -> String {
    let half = game.half.max(1);
    let turn_home = game.turn_data_home.turn_nr;
    let turn_away = game.turn_data_away.turn_nr;
    let active = if game.home_playing { "home" } else { "away" };
    let score_home = game.game_result.home.score;
    let score_away = game.game_result.away.score;

    let fm = &game.field_model;
    let (bx, by, in_play) = match fm.ball_coordinate {
        Some(c) => (c.x as i32, c.y as i32, fm.ball_in_play),
        None => (-1, -1, false),
    };

    let mut player_parts: Vec<String> = Vec::new();
    collect_player_parts(game, &mut player_parts, true);
    collect_player_parts(game, &mut player_parts, false);
    player_parts.sort();

    let mut s = String::with_capacity(256);
    s.push('h');
    s.push_str(&half.to_string());
    s.push('t');
    s.push_str(&turn_home.to_string());
    s.push_str(&turn_away.to_string());
    s.push('a');
    s.push_str(active);
    s.push('s');
    s.push_str(&score_home.to_string());
    s.push(',');
    s.push_str(&score_away.to_string());
    s.push_str(" b");
    s.push_str(&bx.to_string());
    s.push(',');
    s.push_str(&by.to_string());
    s.push(',');
    s.push_str(if in_play { "true" } else { "false" });
    s.push_str(" p");
    for (i, part) in player_parts.iter().enumerate() {
        if i > 0 {
            s.push('|');
        }
        s.push_str(part);
    }
    s
}

/// Canonical state string + FNV-1a hash.
///
/// Format (must match Java's ParityRunner.stateHash()):
/// `h{half}t{turnHome}{turnAway}a{active}s{home},{away} b{bx},{by},{inPlay} p{parts...}`
pub fn state_hash(game: &Game) -> String {
    let s = state_string(game);
    let hash = fnv1a64(s.as_bytes());
    format!("{hash:016x}")
}

fn collect_player_parts(game: &Game, out: &mut Vec<String>, home: bool) {
    let team = if home { &game.team_home } else { &game.team_away };
    let fm = &game.field_model;
    let prefix = if home { "h" } else { "a" };

    let mut players: Vec<_> = team.players.iter().collect();
    players.sort_by_key(|p| p.nr);
    if players.len() > 11 {
        players.truncate(11);
    }

    for (i, player) in players.iter().enumerate() {
        let coord = fm.player_coordinate(&player.id);
        let (x, y) = match coord {
            Some(c) if c.x >= 0 && c.x <= 25 && c.y >= 0 && c.y <= 14 => {
                (c.x as i32, c.y as i32)
            }
            _ => (-1, -1),
        };
        let state_str = match fm.player_state(&player.id) {
            None => "Reserve",
            Some(ps) => player_state_str(ps.base()),
        };
        out.push(format!("{prefix}{i:02}:{x},{y},{state_str}"));
    }
}

fn player_state_str(base: u32) -> &'static str {
    match base {
        b if b == PS_STANDING => "Standing",
        b if b == PS_MOVING   => "Moving",
        b if b == PS_PRONE    => "Prone",
        b if b == PS_STUNNED  => "Stunned",
        b if b == PS_KNOCKED_OUT    => "Ko",
        b if b == PS_BADLY_HURT     => "Injured",
        b if b == PS_SERIOUS_INJURY => "Injured",
        b if b == PS_RIP            => "Injured",
        _ => "Reserve",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game::Game;
    use crate::model::team::Team;
    use crate::enums::Rules;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 3, apothecaries: 1, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 5, dedicated_fans: 5,
            team_value: 1_000_000, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    #[test]
    fn hash_is_16_hex_chars() {
        let g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        let h = state_hash(&g);
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()), "not hex: {h}");
    }

    #[test]
    fn hash_is_deterministic() {
        let g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        assert_eq!(state_hash(&g), state_hash(&g));
    }

    #[test]
    fn fnv_empty_is_offset_basis() {
        assert_eq!(fnv1a64(b""), 0xcbf29ce484222325u64);
    }

    #[test]
    fn fnv_known_values() {
        // FNV-1a("a") = 0xe40c292c0cbf29ce ^ ... compute via the algorithm
        assert_ne!(fnv1a64(b"hello"), fnv1a64(b"world"));
        assert_ne!(fnv1a64(b"hello"), fnv1a64(b"hello world"));
    }
}
