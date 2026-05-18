use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::model::player::Player;
use crate::types::{PlayerId, SpecialRule};

// ── Turn data — reset each turn ───────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TurnData {
    pub turn_number: u8,
    pub reroll_used: bool,
    pub blitz_used: bool,
    pub pass_used: bool,
    pub handoff_used: bool,
    pub foul_used: bool,
    #[serde(default)]
    pub leader_reroll_used: bool,
}

impl TurnData {
    pub fn reset_for_new_turn(&mut self) {
        self.turn_number += 1;
        self.reset_flags();
    }

    /// Reset per-turn flags without touching turn_number.
    /// Used at half-time so begin_turn can increment from 0 to 1.
    pub fn reset_flags(&mut self) {
        self.reroll_used = false;
        self.blitz_used = false;
        self.pass_used = false;
        self.handoff_used = false;
        self.foul_used = false;
        self.leader_reroll_used = false;
    }
}

// ── Team ──────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub race: String,
    pub score: u8,
    pub rerolls_total: u8,
    pub rerolls_remaining: u8,
    pub apothecary_available: bool,
    pub apothecary_used: bool,
    /// Current FAME (Fan Attendance Modifier Effect) value
    pub fame: i8,
    pub special_rules: Vec<SpecialRule>,
    roster: Vec<Player>,
    /// Fast player lookup by PlayerId
    #[serde(skip)]
    index: HashMap<PlayerId, usize>,
}

impl Team {
    pub fn new(id: String, name: String, race: String, rerolls: u8, apothecary: bool) -> Self {
        let mut t = Self {
            id,
            name,
            race,
            score: 0,
            rerolls_total: rerolls,
            rerolls_remaining: rerolls,
            apothecary_available: apothecary,
            apothecary_used: false,
            fame: 0,
            special_rules: Vec::new(),
            roster: Vec::new(),
            index: HashMap::new(),
        };
        t.rebuild_index();
        t
    }

    pub fn add_player(&mut self, player: Player) {
        let idx = self.roster.len();
        self.index.insert(player.id.clone(), idx);
        self.roster.push(player);
    }

    pub fn player_by_id(&self, id: &PlayerId) -> Option<&Player> {
        self.index.get(id).map(|&i| &self.roster[i])
    }

    pub fn player_by_id_mut(&mut self, id: &PlayerId) -> Option<&mut Player> {
        let idx = *self.index.get(id)?;
        Some(&mut self.roster[idx])
    }

    pub fn players(&self) -> &[Player] {
        &self.roster
    }

    pub fn players_mut(&mut self) -> &mut [Player] {
        &mut self.roster
    }

    pub fn use_reroll(&mut self) -> bool {
        if self.rerolls_remaining > 0 {
            self.rerolls_remaining -= 1;
            true
        } else {
            false
        }
    }

    pub fn score_touchdown(&mut self) {
        self.score += 1;
    }

    pub fn reset_for_half(&mut self) {
        self.rerolls_remaining = self.rerolls_total;
        self.apothecary_used = false;
    }

    pub fn has_special_rule(&self, rule: SpecialRule) -> bool {
        self.special_rules.contains(&rule)
    }

    /// Rebuild the internal player index (needed after bincode deserialization).
    pub fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, p) in self.roster.iter().enumerate() {
            self.index.insert(p.id.clone(), i);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::SkillSet;
    use crate::model::player::PlayerStats;
    use crate::types::{TeamId, PlayerId};

    fn make_team() -> Team {
        let mut t = Team::new("home".into(), "Reikland Reavers".into(), "Human".into(), 3, true);
        let p = Player::new(
            PlayerId("p1".into()),
            "Griff Oberwald".into(),
            "blitzer".into(),
            TeamId::Home,
            1,
            PlayerStats::new(9, 3, 4, 8, None),
            SkillSet::empty(),
        );
        t.add_player(p);
        t
    }

    #[test]
    fn player_by_id_found() {
        let t = make_team();
        assert!(t.player_by_id(&PlayerId("p1".into())).is_some());
    }

    #[test]
    fn player_by_id_not_found() {
        let t = make_team();
        assert!(t.player_by_id(&PlayerId("missing".into())).is_none());
    }

    #[test]
    fn use_reroll_decrements() {
        let mut t = make_team();
        assert_eq!(t.rerolls_remaining, 3);
        assert!(t.use_reroll());
        assert_eq!(t.rerolls_remaining, 2);
    }

    #[test]
    fn use_reroll_fails_when_empty() {
        let mut t = make_team();
        t.rerolls_remaining = 0;
        assert!(!t.use_reroll());
    }

    #[test]
    fn bincode_round_trip() {
        let t = make_team();
        let encoded = bincode::serde::encode_to_vec(&t, bincode::config::standard()).unwrap();
        let (mut decoded, _): (Team, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        decoded.rebuild_index();
        assert_eq!(t.score, decoded.score);
        assert_eq!(t.name, decoded.name);
        assert!(decoded.player_by_id(&PlayerId("p1".into())).is_some());
    }
}
