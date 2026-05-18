use serde::{Deserialize, Serialize};
use crate::skills::{SkillId, SkillSet};
use crate::types::{PlayerId, PlayerGender, SeriousInjury, TeamId};

// ── Player stats ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub ma: u8,
    pub st: u8,
    pub ag: u8,
    pub av: u8,
    /// Passing attribute; None means "no passing stat" (non-thrower positions)
    pub pa: Option<u8>,
}

impl PlayerStats {
    pub fn new(ma: u8, st: u8, ag: u8, av: u8, pa: Option<u8>) -> Self {
        Self { ma, st, ag, av, pa }
    }
}

// ── Lasting injury applied to a stat ─────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct StatImprovement {
    pub stat: StatKey,
    pub delta: i8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StatKey {
    Ma,
    St,
    Ag,
    Av,
    Pa,
}

// ── Player ────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub position_id: String,
    pub team_id: TeamId,
    pub jersey_number: u8,
    pub base_stats: PlayerStats,
    pub skills: SkillSet,
    /// Temporary skills (e.g., from inducement cards) — cleared at end of game
    pub temp_skills: SkillSet,
    pub lasting_injuries: Vec<SeriousInjury>,
    /// Permanent stat improvements (player advancements)
    pub lasting_improvements: Vec<StatImprovement>,
    pub current_spps: u16,
    pub gender: PlayerGender,
    /// Set when the player is recovering from a Miss Next Game injury
    pub recovering_injury: Option<SeriousInjury>,
}

impl Player {
    pub fn new(
        id: PlayerId,
        name: String,
        position_id: String,
        team_id: TeamId,
        jersey_number: u8,
        base_stats: PlayerStats,
        skills: SkillSet,
    ) -> Self {
        Self {
            id,
            name,
            position_id,
            team_id,
            jersey_number,
            base_stats,
            skills,
            temp_skills: SkillSet::empty(),
            lasting_injuries: Vec::new(),
            lasting_improvements: Vec::new(),
            current_spps: 0,
            gender: PlayerGender::default(),
            recovering_injury: None,
        }
    }

    pub fn has_skill(&self, skill: SkillId) -> bool {
        self.skills.has(skill) || self.temp_skills.has(skill)
    }

    pub fn has_permanent_skill(&self, skill: SkillId) -> bool {
        self.skills.has(skill)
    }

    // ── Effective stats (applying lasting improvements and injuries) ───────

    pub fn effective_ma(&self) -> u8 {
        let base = self.base_stats.ma as i16;
        let skill_bonus = if self.has_skill(SkillId::MovementIncrease) { 1i16 } else { 0 };
        let delta: i16 = self.lasting_improvements.iter()
            .filter(|i| i.stat == StatKey::Ma)
            .map(|i| i.delta as i16)
            .sum::<i16>()
            - self.lasting_injuries.iter()
            .filter(|&i| *i == SeriousInjury::MinusOneMovement)
            .count() as i16;
        (base + delta + skill_bonus).clamp(1, 9) as u8
    }

    pub fn effective_st(&self) -> u8 {
        let base = self.base_stats.st as i16;
        let skill_bonus = if self.has_skill(SkillId::StrengthIncrease) { 1i16 } else { 0 };
        let delta: i16 = self.lasting_improvements.iter()
            .filter(|i| i.stat == StatKey::St)
            .map(|i| i.delta as i16)
            .sum::<i16>()
            - self.lasting_injuries.iter()
            .filter(|&i| *i == SeriousInjury::MinusOneStrength)
            .count() as i16;
        (base + delta + skill_bonus).clamp(1, 8) as u8
    }

    pub fn effective_ag(&self) -> u8 {
        let base = self.base_stats.ag as i16;
        let skill_bonus = if self.has_skill(SkillId::AgilityIncrease) { 1i16 } else { 0 };
        let delta: i16 = self.lasting_improvements.iter()
            .filter(|i| i.stat == StatKey::Ag)
            .map(|i| i.delta as i16)
            .sum::<i16>()
            - self.lasting_injuries.iter()
            .filter(|&i| *i == SeriousInjury::MinusOneAgility)
            .count() as i16;
        (base + delta + skill_bonus).clamp(1, 6) as u8
    }

    pub fn effective_av(&self) -> u8 {
        let base = self.base_stats.av as i16;
        let delta: i16 = self.lasting_improvements.iter()
            .filter(|i| i.stat == StatKey::Av)
            .map(|i| i.delta as i16)
            .sum::<i16>()
            - self.lasting_injuries.iter()
            .filter(|&i| *i == SeriousInjury::MinusOneArmour)
            .count() as i16;
        (base + delta).clamp(3, 11) as u8
    }

    pub fn effective_pa(&self) -> Option<u8> {
        let base = self.base_stats.pa? as i16;
        // PassingIncrease reduces PA value by 1 (lower is better); minimum 1
        let skill_bonus = if self.has_skill(SkillId::PassingIncrease) { -1i16 } else { 0 };
        let delta: i16 = self.lasting_improvements.iter()
            .filter(|i| i.stat == StatKey::Pa)
            .map(|i| i.delta as i16)
            .sum::<i16>()
            - self.lasting_injuries.iter()
            .filter(|&i| *i == SeriousInjury::MinusOnePassing)
            .count() as i16;
        Some((base + delta + skill_bonus).clamp(1, 6) as u8)
    }

    pub fn is_dead(&self) -> bool {
        self.lasting_injuries.contains(&SeriousInjury::Dead)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PlayerId;

    fn make_player() -> Player {
        Player::new(
            PlayerId("p1".into()),
            "Griff".into(),
            "blitzer".into(),
            TeamId::Home,
            8,
            PlayerStats::new(9, 3, 4, 8, None),
            [SkillId::Block, SkillId::Dodge, SkillId::Sprint, SkillId::SureFeet]
                .into_iter()
                .collect(),
        )
    }

    #[test]
    fn has_skill_positive() {
        let p = make_player();
        assert!(p.has_skill(SkillId::Block));
        assert!(p.has_skill(SkillId::Dodge));
    }

    #[test]
    fn has_skill_negative() {
        let p = make_player();
        assert!(!p.has_skill(SkillId::Guard));
        assert!(!p.has_skill(SkillId::Tackle));
    }

    #[test]
    fn effective_stats_no_injuries() {
        let p = make_player();
        assert_eq!(p.effective_ma(), 9);
        assert_eq!(p.effective_st(), 3);
        assert_eq!(p.effective_ag(), 4);
        assert_eq!(p.effective_av(), 8);
        assert_eq!(p.effective_pa(), None);
    }

    #[test]
    fn effective_ag_with_injury() {
        let mut p = make_player();
        p.lasting_injuries.push(SeriousInjury::MinusOneAgility);
        assert_eq!(p.effective_ag(), 3);
    }

    #[test]
    fn effective_stat_clamped_at_one() {
        let mut p = make_player();
        for _ in 0..10 {
            p.lasting_injuries.push(SeriousInjury::MinusOneMovement);
        }
        assert_eq!(p.effective_ma(), 1);
    }

    #[test]
    fn temp_skill_visible() {
        let mut p = make_player();
        p.temp_skills.add(SkillId::Loner);
        assert!(p.has_skill(SkillId::Loner));
        assert!(!p.has_permanent_skill(SkillId::Loner));
    }

    #[test]
    fn bincode_round_trip() {
        let p = make_player();
        let encoded = bincode::serde::encode_to_vec(&p, bincode::config::standard()).unwrap();
        let (decoded, _): (Player, _) = bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(p, decoded);
    }

    #[test]
    fn agility_increase_adds_one_effective_ag() {
        let mut p = Player::new(
            PlayerId("ag_test".into()),
            "Test".into(),
            "lineman".into(),
            crate::types::TeamId::Home,
            1,
            PlayerStats::new(6, 3, 3, 8, None),
            [SkillId::AgilityIncrease].into_iter().collect(),
        );
        assert_eq!(p.effective_ag(), 4);
    }

    #[test]
    fn movement_increase_adds_one_effective_ma() {
        let p = Player::new(
            PlayerId("ma_test".into()),
            "Test".into(),
            "lineman".into(),
            crate::types::TeamId::Home,
            1,
            PlayerStats::new(6, 3, 3, 8, None),
            [SkillId::MovementIncrease].into_iter().collect(),
        );
        assert_eq!(p.effective_ma(), 7);
    }

    #[test]
    fn strength_increase_adds_one_effective_st() {
        let p = Player::new(
            PlayerId("st_test".into()),
            "Test".into(),
            "lineman".into(),
            crate::types::TeamId::Home,
            1,
            PlayerStats::new(6, 3, 3, 8, None),
            [SkillId::StrengthIncrease].into_iter().collect(),
        );
        assert_eq!(p.effective_st(), 4);
    }

    #[test]
    fn passing_increase_reduces_effective_pa_by_one() {
        let p = Player::new(
            PlayerId("pa_test".into()),
            "Test".into(),
            "thrower".into(),
            crate::types::TeamId::Home,
            1,
            PlayerStats::new(6, 3, 3, 8, Some(4)),
            [SkillId::PassingIncrease].into_iter().collect(),
        );
        // PA4 - 1 = PA3
        assert_eq!(p.effective_pa(), Some(3));
    }

    #[test]
    fn passing_increase_does_not_go_below_one() {
        let p = Player::new(
            PlayerId("pa_min_test".into()),
            "Test".into(),
            "thrower".into(),
            crate::types::TeamId::Home,
            1,
            PlayerStats::new(6, 3, 3, 8, Some(1)),
            [SkillId::PassingIncrease].into_iter().collect(),
        );
        assert_eq!(p.effective_pa(), Some(1));
    }
}
