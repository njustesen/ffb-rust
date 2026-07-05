/// 1:1 translation of com.fumbbl.ffb.server.marking.MarkerGenerator.
use std::collections::HashMap;
use ffb_model::enums::{SkillCategory, SeriousInjuryKind, PlayerStatKey};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::injury_attribute::InjuryAttribute;
use ffb_model::model::skill_def::SkillId;
use ffb_model::marking::sort_mode::SortMode;
use ffb_mechanics::skills::SKILL_TABLE;
use crate::marking::apply_to::ApplyTo;
use crate::marking::auto_marking_config::AutoMarkingConfig;
use crate::marking::auto_marking_record::AutoMarkingRecord;

pub struct MarkerGenerator;

impl MarkerGenerator {
    pub fn new() -> Self { Self }

    /// Java: generate(Game, Player<?>, AutoMarkingConfig, boolean).
    pub fn generate(
        &self,
        game: &Game,
        player: &Player,
        config: &AutoMarkingConfig,
        plays_for_marking_coach: bool,
    ) -> String {
        // Java: baseSkills = player.getPosition().getSkills()
        let base_skill_ids: Vec<SkillId> = player
            .starting_skills
            .iter()
            .map(|sw| sw.skill_id)
            .collect();

        // Java: gainedSkills = player.getSkillsIncludingTemporaryOnes()
        //         .filter(skill -> skill.getCategory() != STAT_INCREASE)
        //         .removeAll(baseSkills)
        let mut gained_skill_ids: Vec<SkillId> = player
            .extra_skills
            .iter()
            .chain(player.temporary_skills.iter())
            .map(|sw| sw.skill_id)
            .filter(|id| !is_stat_increase(*id))
            .collect();

        // removeAll(baseSkills) — remove one copy per base skill
        for base_id in &base_skill_ids {
            if let Some(pos) = gained_skill_ids.iter().position(|id| id == base_id) {
                gained_skill_ids.remove(pos);
            }
        }

        // Java: for each PlayerStatKey, compute statDiff and add stat skills or injury attrs.
        let mut injury_attributes: Vec<InjuryAttribute> = Vec::new();
        for &key in PlayerStatKey::all() {
            let diff = stat_diff(key, player);
            if diff > 0 {
                let skill_id = key.skill_id_for_increase();
                for _ in 0..diff {
                    gained_skill_ids.push(skill_id);
                }
            } else if diff < 0 {
                if let Some(attr) = InjuryAttribute::for_stat_key(key) {
                    for _ in 0..(-diff) {
                        injury_attributes.push(attr);
                    }
                }
            }
        }

        // Java: injuries from game result (current game SI) filtered to NI only
        let player_result = game.game_result.home.player_results.get(&player.id)
            .or_else(|| game.game_result.away.player_results.get(&player.id));
        let result_injuries: Vec<SeriousInjuryKind> = player_result
            .map(|pr| {
                let mut v = Vec::new();
                if let Some(si) = pr.serious_injury { v.push(si); }
                if let Some(si) = pr.serious_injury_decay { v.push(si); }
                v
            })
            .unwrap_or_default();

        // Java: player.getLastingInjuries() — stat_injuries equivalent
        let lasting_injuries: Vec<SeriousInjuryKind> = player.stat_injuries.clone();

        let all_injuries = result_injuries.into_iter().chain(lasting_injuries);
        for si in all_injuries {
            if let Some(attr) = si.injury_attribute() {
                if attr == InjuryAttribute::NI {
                    injury_attributes.push(InjuryAttribute::NI);
                }
            }
        }

        let separator = config.get_separator();
        let records: Vec<&AutoMarkingRecord> = config
            .get_markings()
            .iter()
            .filter(|r| {
                !r.skills().iter().any(|_| false) // Java: !contains(null) — not needed in Rust
                    && applies_to(r.apply_to(), plays_for_marking_coach)
            })
            .collect();

        let mut records_to_apply: Vec<&AutoMarkingRecord> = Vec::new();

        if config.get_sort_mode() == SortMode::None {
            for record in &records {
                populate_marking_records(record, &base_skill_ids, &gained_skill_ids, &injury_attributes, &mut records_to_apply);
            }
            records_to_apply
                .iter()
                .map(|r| r.marking())
                .filter(|m| !m.is_empty())
                .collect::<Vec<_>>()
                .join(separator)
        } else {
            populate_and_sort_records(&records, &base_skill_ids, &gained_skill_ids, &injury_attributes, &mut records_to_apply);
            let mut sorted = records_to_apply;
            sorted.sort_by(|a, b| {
                let injury_a = if a.is_injury_only() { 1 } else { 0 };
                let injury_b = if b.is_injury_only() { 1 } else { 0 };
                injury_a.cmp(&injury_b).then_with(|| a.marking().cmp(b.marking()))
            });
            sorted
                .iter()
                .map(|r| r.marking())
                .filter(|m| !m.is_empty())
                .collect::<Vec<_>>()
                .join(separator)
        }
    }
}

impl Default for MarkerGenerator {
    fn default() -> Self { Self::new() }
}

/// Java: `MarkerGenerator.statDiff(Game, PlayerStatKey, Player)`.
///
/// Returns (current_stat - position_base_stat) for the given key.
/// Positive = stat was gained (advancement); negative = stat was lost (injury).
///
/// For Ag/PA the sign is inverted per BB2020/2025 convention (lower PA = better;
/// higher AG = better in BB2025 but inverted in BB2016 — that distinction is
/// DEFERRED until StatsMechanic is fully ported).
fn stat_diff(key: PlayerStatKey, player: &Player) -> i32 {
    match key {
        PlayerStatKey::Ma => player.movement_with_modifiers() - player.position_movement,
        PlayerStatKey::St => player.strength_with_modifiers() - player.position_strength,
        PlayerStatKey::Ag => player.agility_with_modifiers() - player.position_agility,
        PlayerStatKey::Pa => player.position_passing - player.passing_with_modifiers(),
        PlayerStatKey::Av => player.armour_with_modifiers() - player.position_armour,
    }
}

fn applies_to(apply_to: ApplyTo, plays_for_marking_coach: bool) -> bool {
    (plays_for_marking_coach && apply_to.applies_to_own())
        || (!plays_for_marking_coach && apply_to.applies_to_opponent())
}

fn populate_and_sort_records<'a>(
    records: &[&'a AutoMarkingRecord],
    base_skills: &[SkillId],
    gained_skills: &[SkillId],
    injuries: &[InjuryAttribute],
    records_to_apply: &mut Vec<&'a AutoMarkingRecord>,
) {
    // Group by injury_only, process non-injury-only first, then injury-only
    let mut skill_records: Vec<&AutoMarkingRecord> = records
        .iter()
        .copied()
        .filter(|r| !r.is_injury_only())
        .collect();
    let mut injury_records: Vec<&AutoMarkingRecord> = records
        .iter()
        .copied()
        .filter(|r| r.is_injury_only())
        .collect();

    // Sort each group by complexity descending, then by apply_to, gained_only, apply_repeatedly, marking
    let apply_to_ord = |a: ApplyTo| match a {
        ApplyTo::Both => 0i32,
        ApplyTo::Own => 1,
        ApplyTo::Opponent => 2,
    };

    let sort_group = |group: &mut Vec<&AutoMarkingRecord>| {
        group.sort_by(|a, b| {
            let skill_len = b.skills().len().cmp(&a.skills().len());
            let inj_len = b.injuries().len().cmp(&a.injuries().len());
            skill_len
                .then(inj_len)
                .then_with(|| apply_to_ord(a.apply_to()).cmp(&apply_to_ord(b.apply_to())))
                .then_with(|| a.is_gained_only().cmp(&b.is_gained_only()))
                .then_with(|| b.is_apply_repeatedly().cmp(&a.is_apply_repeatedly()))
                .then_with(|| a.marking().cmp(b.marking()))
        });
    };

    sort_group(&mut skill_records);
    sort_group(&mut injury_records);

    for record in skill_records.iter().chain(injury_records.iter()) {
        populate_marking_records(record, base_skills, gained_skills, injuries, records_to_apply);
    }
}

fn populate_marking_records<'a>(
    record: &'a AutoMarkingRecord,
    base_skills: &[SkillId],
    gained_skills: &[SkillId],
    injuries: &[InjuryAttribute],
    records_to_apply: &mut Vec<&'a AutoMarkingRecord>,
) {
    // Skip if already superseded
    if records_to_apply.iter().any(|r| record.is_subset_of(r)) {
        return;
    }

    let mut skills_to_check = gained_skills.to_vec();
    if !record.is_gained_only() {
        skills_to_check.extend_from_slice(base_skills);
    }

    let skill_matches = is_subset_with_duplicates_skill(record.skills(), &skills_to_check);
    let inj_matches = is_subset_with_duplicates_inj(record.injuries(), injuries);
    let mut matches = find_min(skill_matches, inj_matches);

    if !record.is_apply_repeatedly() {
        matches = matches.min(1);
    }

    if matches > 0 {
        records_to_apply.retain(|r| !r.is_subset_of(record));
    }

    for _ in 0..matches {
        records_to_apply.push(record);
    }
}

fn find_min(first: usize, second: usize) -> usize {
    let result = first.min(second);
    if result == usize::MAX { 0 } else { result }
}

/// Java: isSubSetWithDuplicates for SkillId.
fn is_subset_with_duplicates_skill(subset: &[SkillId], superset: &[SkillId]) -> usize {
    if subset.is_empty() {
        return usize::MAX;
    }

    // Count occurrences in subset and superset
    let sub_counts = count_occurrences(subset);
    let super_counts = count_occurrences(superset);

    sub_counts
        .iter()
        .map(|(id, sub_n)| {
            let super_n = super_counts.get(id).copied().unwrap_or(0);
            if super_n == 0 { 0 } else { super_n / sub_n }
        })
        .min()
        .unwrap_or(0)
}

/// Java: isSubSetWithDuplicates for InjuryAttribute.
fn is_subset_with_duplicates_inj(subset: &[InjuryAttribute], superset: &[InjuryAttribute]) -> usize {
    if subset.is_empty() {
        return usize::MAX;
    }

    let sub_counts = count_occurrences_inj(subset);
    let super_counts = count_occurrences_inj(superset);

    sub_counts
        .iter()
        .map(|(id, sub_n)| {
            let super_n = super_counts.get(id).copied().unwrap_or(0);
            if super_n == 0 { 0 } else { super_n / sub_n }
        })
        .min()
        .unwrap_or(0)
}

fn count_occurrences(items: &[SkillId]) -> HashMap<SkillId, usize> {
    let mut map = HashMap::new();
    for item in items {
        *map.entry(*item).or_insert(0) += 1;
    }
    map
}

fn count_occurrences_inj(items: &[InjuryAttribute]) -> HashMap<InjuryAttribute, usize> {
    let mut map = HashMap::new();
    for item in items {
        *map.entry(*item).or_insert(0) += 1;
    }
    map
}

fn is_stat_increase(id: SkillId) -> bool {
    SKILL_TABLE
        .iter()
        .find(|def| def.id == id)
        .map(|def| def.category == SkillCategory::StatIncrease)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::{SkillId, SkillWithValue};
    use crate::marking::auto_marking_config::AutoMarkingConfig;
    use crate::marking::auto_marking_record::Builder;

    fn make_game() -> Game {
        Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            Rules::Bb2025,
        )
    }

    fn make_player() -> Player {
        Player::default()
    }

    #[test]
    fn generate_empty_config_returns_empty_string() {
        let gen = MarkerGenerator::new();
        let g = make_game();
        let p = make_player();
        let config = AutoMarkingConfig::new();
        let result = gen.generate(&g, &p, &config, false);
        assert!(result.is_empty());
    }

    #[test]
    fn generate_with_no_matching_skills_returns_empty() {
        let gen = MarkerGenerator::new();
        let g = make_game();
        let p = make_player(); // no skills
        let mut config = AutoMarkingConfig::new();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_marking("B")
                .with_gained_only(true)
                .build()
        );
        let result = gen.generate(&g, &p, &config, false);
        assert!(result.is_empty());
    }

    #[test]
    fn generate_matches_gained_skill() {
        let gen = MarkerGenerator::new();
        let g = make_game();
        let mut p = make_player();
        p.extra_skills.push(SkillWithValue::new(SkillId::Block));

        let mut config = AutoMarkingConfig::new();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_marking("B")
                .with_gained_only(true)
                .build()
        );
        let result = gen.generate(&g, &p, &config, false);
        assert_eq!(result, "B");
    }

    #[test]
    fn generate_does_not_match_base_skill_when_gained_only() {
        let gen = MarkerGenerator::new();
        let g = make_game();
        let mut p = make_player();
        p.starting_skills.push(SkillWithValue::new(SkillId::Block));

        let mut config = AutoMarkingConfig::new();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_marking("B")
                .with_gained_only(true)
                .build()
        );
        let result = gen.generate(&g, &p, &config, false);
        assert!(result.is_empty());
    }

    #[test]
    fn generate_matches_base_skill_when_not_gained_only() {
        let gen = MarkerGenerator::new();
        let g = make_game();
        let mut p = make_player();
        p.starting_skills.push(SkillWithValue::new(SkillId::Block));

        let mut config = AutoMarkingConfig::new();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_marking("B")
                .with_gained_only(false)
                .build()
        );
        let result = gen.generate(&g, &p, &config, false);
        assert_eq!(result, "B");
    }

    #[test]
    fn generate_multiple_skills_joined_with_separator() {
        let gen = MarkerGenerator::new();
        let g = make_game();
        let mut p = make_player();
        p.extra_skills.push(SkillWithValue::new(SkillId::Block));
        p.extra_skills.push(SkillWithValue::new(SkillId::Tackle));

        let mut config = AutoMarkingConfig::new();
        config.set_separator("/");
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking("B").with_gained_only(true).build());
        config.markings.push(Builder::new().with_skill(SkillId::Tackle).with_marking("T").with_gained_only(true).build());

        let result = gen.generate(&g, &p, &config, false);
        // Both "B" and "T" should appear (order depends on sort_mode=Default → sorted)
        assert!(result.contains("B"));
        assert!(result.contains("T"));
        assert!(result.contains("/"));
    }

    #[test]
    fn applies_to_both_applies_to_opponent() {
        assert!(applies_to(ApplyTo::Both, false));
    }

    #[test]
    fn applies_to_own_does_not_apply_to_opponent() {
        assert!(!applies_to(ApplyTo::Own, false));
    }

    #[test]
    fn is_subset_with_duplicates_skill_empty_subset_returns_max() {
        let result = is_subset_with_duplicates_skill(&[], &[SkillId::Block]);
        assert_eq!(result, usize::MAX);
    }

    #[test]
    fn is_subset_with_duplicates_skill_not_in_superset() {
        let result = is_subset_with_duplicates_skill(&[SkillId::Block], &[SkillId::Tackle]);
        assert_eq!(result, 0);
    }

    #[test]
    fn is_subset_with_duplicates_skill_in_superset() {
        let result = is_subset_with_duplicates_skill(&[SkillId::Block], &[SkillId::Block]);
        assert_eq!(result, 1);
    }

    #[test]
    fn is_subset_with_duplicates_skill_duplicates_respected() {
        // Want 2× Block, have 4× → can apply twice
        let sub = vec![SkillId::Block, SkillId::Block];
        let sup = vec![SkillId::Block, SkillId::Block, SkillId::Block, SkillId::Block];
        let result = is_subset_with_duplicates_skill(&sub, &sup);
        assert_eq!(result, 2);
    }
}
