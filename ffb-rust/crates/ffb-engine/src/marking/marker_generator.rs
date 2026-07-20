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
/// higher AG = better in BB2025 but inverted in BB2016 — uses player.position_* fields directly).
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
    use ffb_model::enums::{Rules, SeriousInjuryKind};
    use ffb_model::marking::sort_mode::SortMode;
    use ffb_model::model::game::Game;
    use ffb_model::model::game_result::PlayerResult;
    use ffb_model::model::injury_attribute::InjuryAttribute;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::{SkillId, SkillWithValue};
    use crate::marking::auto_marking_config::AutoMarkingConfig;
    use crate::marking::auto_marking_record::Builder;

    // ------------------------------------------------------------------
    // Mirror of ffb-java MarkerGeneratorTest (ffb-server, 47 @Test methods).
    // Java constants:
    // ------------------------------------------------------------------
    const BLOCK_MARKING: &str = "B";
    const BLODGE_MARKING: &str = "X";
    const DODGE_MARKING: &str = "D";
    const BLACKLE_MARKING: &str = "Y";
    const WRECKLE_MARKING: &str = "Q";
    const MA_MARKING: &str = "Ma";
    const AG_MARKING: &str = "Ag";
    const NI_MARKING: &str = "Ni";
    const TACKLE_MARKING: &str = "T";
    const WRESTLE_MARKING: &str = "W";
    const OTHER_MARKING: &str = "O";
    const SEPARATOR: &str = ", ";

    const MOVE: i32 = 6;
    const STRENGTH: i32 = 3;
    const AGILITY: i32 = 3;
    const PASSING: i32 = 4;
    const ARMOUR: i32 = 8;

    fn make_game() -> Game {
        Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            Rules::Bb2025,
        )
    }

    /// Mirror of Java `MarkerGeneratorTest.setUp()`:
    /// - position (base) skills: wrestle, tackle
    /// - gained skills (`getSkillsIncludingTemporaryOnes`): block, dodge
    /// - movement 4 vs position 6 → two MA injury attributes
    /// - agility: Java stubs `getAgilityWithModifiers` = AGILITY + 1 under the
    ///   mixed `StatsMechanic` where `improvementIncreasesValue()` is false
    ///   (BB2020: higher AG number = worse), i.e. one net AG loss. Rust's
    ///   `stat_diff` uses the BB2025 convention (higher AG = better), so the
    ///   same single AG loss is expressed here as agility 2 vs position 3.
    /// - passing / strength / armour unchanged → no attributes
    /// - lasting injuries HEAD_INJURY (AV, filtered) + SERIOUS_INJURY (NI)
    ///   → one NI injury attribute
    fn setup_player() -> Player {
        let mut p = Player::default();
        p.id = "p1".to_string();
        p.starting_skills.push(SkillWithValue::new(SkillId::Wrestle));
        p.starting_skills.push(SkillWithValue::new(SkillId::Tackle));
        p.extra_skills.push(SkillWithValue::new(SkillId::Block));
        p.extra_skills.push(SkillWithValue::new(SkillId::Dodge));
        p.position_movement = MOVE;
        p.movement = MOVE - 2;
        p.position_strength = STRENGTH;
        p.strength = STRENGTH;
        p.position_agility = AGILITY;
        p.agility = AGILITY - 1;
        p.position_passing = PASSING;
        p.passing = PASSING;
        p.position_armour = ARMOUR;
        p.armour = ARMOUR;
        p.stat_injuries = vec![
            SeriousInjuryKind::HeadInjuryAv,
            SeriousInjuryKind::SeriousInjuryNi,
        ];
        p
    }

    /// Java equivalent of `@BeforeEach setUp()` — generator, game, player, config.
    fn setup() -> (MarkerGenerator, Game, Player, AutoMarkingConfig) {
        (MarkerGenerator::new(), make_game(), setup_player(), AutoMarkingConfig::new())
    }

    /// Java: `given(player.getSkillsIncludingTemporaryOnes()).willReturn(increases)` —
    /// the gained-skill list is replaced by "+AG" stat increases, and the player's
    /// modified agility reflects `count` net increases (Java: AGILITY - count under
    /// the inverted BB2020 sign; BB2025 convention here: AGILITY + count).
    fn replace_gained_with_ag_increases(p: &mut Player, count: i32) {
        p.extra_skills.clear();
        for _ in 0..count {
            p.extra_skills.push(SkillWithValue::new(SkillId::AgilityIncrease));
        }
        p.agility = AGILITY + count;
    }

    // ------------------------------------------------------------------
    // Ported Java tests (same order as MarkerGeneratorTest.java)
    // ------------------------------------------------------------------

    /// Java: generate().
    #[test]
    fn generate() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLOCK_MARKING, marking);
    }

    /// Java: generateWithoutSorting().
    #[test]
    fn generate_without_sorting() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());

        config.set_sort_mode(SortMode::None);

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(format!("{}{}{}", DODGE_MARKING, MA_MARKING, BLOCK_MARKING), marking);
    }

    /// Java: generateForSuperSetsWithoutSorting().
    #[test]
    fn generate_for_super_sets_without_sorting() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_skill(SkillId::Dodge)
                .with_skill(SkillId::Tackle)
                .with_marking(BLACKLE_MARKING)
                .build(),
        );

        config.set_sort_mode(SortMode::None);

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLACKLE_MARKING, marking);
    }

    /// Java: generateWithSeparator().
    #[test]
    fn generate_with_separator() {
        let (generator, game, player, mut config) = setup();
        config.set_separator(SEPARATOR);
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).build());
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::MA)
                .with_marking(MA_MARKING)
                .with_apply_repeatedly(true)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(
            format!("{0}{4}{1}{4}{2}{4}{3}", BLOCK_MARKING, DODGE_MARKING, MA_MARKING, MA_MARKING, SEPARATOR),
            marking
        );
    }

    // Java: ignoreUnknownSkills() — NOT PORTABLE: relies on SkillFactory returning
    // null for an unknown skill name so the record contains a null skill; Rust
    // records hold SkillId values which cannot be null/unknown.

    // Java: ignoreMarkingWithUnknownSkills() — NOT PORTABLE: same null-skill
    // mechanism (record with a null skill is filtered out).

    // Java: ignoreMarkingWithUnknownInjuries() — NOT PORTABLE: builds a record
    // with a null InjuryAttribute via withInjury(null); Rust InjuryAttribute is
    // a plain enum with no null state.

    /// Java: generateNoMarking().
    #[test]
    fn generate_no_marking() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::SneakyGit).with_marking(BLOCK_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert!(marking.is_empty());
    }

    /// Java: generateOnlyForPresentSkills().
    #[test]
    fn generate_only_for_present_skills() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::SneakyGit).with_marking(DODGE_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLOCK_MARKING, marking);
    }

    /// Java: generateMarkingsForOverlappingConfigs().
    #[test]
    fn generate_markings_for_overlapping_configs() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_skill(SkillId::Dodge).with_marking(BLODGE_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_skill(SkillId::Tackle).with_marking(BLACKLE_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(format!("{}{}", BLODGE_MARKING, BLACKLE_MARKING), marking);
    }

    /// Java: ignoreCombinedConfigsForGainedSkillsWithOnlyPartialMatch().
    #[test]
    fn ignore_combined_configs_for_gained_skills_with_only_partial_match() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_skill(SkillId::Dodge).with_marking(BLODGE_MARKING).build());
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_skill(SkillId::Tackle)
                .with_marking(BLACKLE_MARKING)
                .with_gained_only(true)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLODGE_MARKING, marking);
    }

    /// Java: ignoreSubsets().
    #[test]
    fn ignore_subsets() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_skill(SkillId::Dodge).with_marking(BLODGE_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLODGE_MARKING, marking);
    }

    /// Java: ignoreSubsetUnlessApplyToMakesDifference().
    #[test]
    fn ignore_subset_unless_apply_to_makes_difference() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_skill(SkillId::Dodge)
                .with_marking(BLODGE_MARKING)
                .with_apply_to(ApplyTo::Own)
                .build(),
        );
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).with_apply_to(ApplyTo::Opponent).build());
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).with_apply_to(ApplyTo::Both).build());

        let marking = generator.generate(&game, &player, &config, false);

        assert_eq!(format!("{}{}", BLOCK_MARKING, DODGE_MARKING), marking);
    }

    /// Java: ignoreSubsetUnlessGainedOnlyMakesDifference().
    #[test]
    fn ignore_subset_unless_gained_only_makes_difference() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Wrestle)
                .with_skill(SkillId::Tackle)
                .with_marking(WRECKLE_MARKING)
                .with_gained_only(true)
                .build(),
        );
        config.markings.push(Builder::new().with_skill(SkillId::Wrestle).with_marking(WRESTLE_MARKING).build());

        let marking = generator.generate(&game, &player, &config, false);

        assert_eq!(WRESTLE_MARKING, marking);
    }

    /// Java: generateForAllMatchingConfigs().
    #[test]
    fn generate_for_all_matching_configs() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(format!("{}{}", BLOCK_MARKING, DODGE_MARKING), marking);
    }

    /// Java: generateForAllMatchingConfigsWithOpponent().
    #[test]
    fn generate_for_all_matching_configs_with_opponent() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).build());

        let marking = generator.generate(&game, &player, &config, false);

        assert_eq!(format!("{}{}", BLOCK_MARKING, DODGE_MARKING), marking);
    }

    /// Java: generateForAllMatchingConfigsWithMatchingApplyTo().
    #[test]
    fn generate_for_all_matching_configs_with_matching_apply_to() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).with_apply_to(ApplyTo::Own).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).with_apply_to(ApplyTo::Opponent).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLOCK_MARKING, marking);
    }

    /// Java: generateForAllMatchingConfigsWithOpponentAndMatchingApplyTo().
    #[test]
    fn generate_for_all_matching_configs_with_opponent_and_matching_apply_to() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).with_apply_to(ApplyTo::Own).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).with_apply_to(ApplyTo::Opponent).build());

        let marking = generator.generate(&game, &player, &config, false);

        assert_eq!(DODGE_MARKING, marking);
    }

    /// Java: generateForAllMatchingConfigsWithGainedOnly().
    #[test]
    fn generate_for_all_matching_configs_with_gained_only() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Wrestle).with_marking(WRESTLE_MARKING).with_gained_only(true).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).with_gained_only(true).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(DODGE_MARKING, marking);
    }

    /// Java: generateForAllMatchingConfigsWithOpponentAndGainedOnly().
    #[test]
    fn generate_for_all_matching_configs_with_opponent_and_gained_only() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Wrestle).with_marking(WRESTLE_MARKING).with_gained_only(true).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).with_gained_only(true).build());

        let marking = generator.generate(&game, &player, &config, false);

        assert_eq!(DODGE_MARKING, marking);
    }

    /// Java: generateForAllMatchingConfigsWithMatchingGainedAndApplyTo().
    #[test]
    fn generate_for_all_matching_configs_with_matching_gained_and_apply_to() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Wrestle)
                .with_marking(WRESTLE_MARKING)
                .with_gained_only(true)
                .with_apply_to(ApplyTo::Own)
                .build(),
        );
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).with_apply_to(ApplyTo::Own).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).with_apply_to(ApplyTo::Opponent).build());
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Tackle)
                .with_marking(TACKLE_MARKING)
                .with_gained_only(true)
                .with_apply_to(ApplyTo::Opponent)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLOCK_MARKING, marking);
    }

    /// Java: generateForAllMatchingConfigsWithOpponentAndMatchingGainedAndApplyTo().
    #[test]
    fn generate_for_all_matching_configs_with_opponent_and_matching_gained_and_apply_to() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Wrestle)
                .with_marking(WRESTLE_MARKING)
                .with_gained_only(true)
                .with_apply_to(ApplyTo::Own)
                .build(),
        );
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).with_apply_to(ApplyTo::Own).build());
        config.markings.push(Builder::new().with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).with_apply_to(ApplyTo::Opponent).build());
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Tackle)
                .with_marking(TACKLE_MARKING)
                .with_gained_only(true)
                .with_apply_to(ApplyTo::Opponent)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, false);

        assert_eq!(DODGE_MARKING, marking);
    }

    /// Java: generateForSingleInjuryMarkings().
    #[test]
    fn generate_for_single_injury_markings() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AG).with_marking(AG_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(format!("{}{}", AG_MARKING, MA_MARKING), marking);
    }

    /// Java: ignoreGainedOnlyOnInjuryMarkings().
    #[test]
    fn ignore_gained_only_on_injury_markings() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AG).with_marking(AG_MARKING).with_gained_only(true).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).with_gained_only(true).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(format!("{}{}", AG_MARKING, MA_MARKING), marking);
    }

    /// Java: generateForMultiInjuryMarkings().
    #[test]
    fn generate_for_multi_injury_markings() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::AG)
                .with_injury(InjuryAttribute::AG)
                .with_marking(AG_MARKING)
                .build(),
        );
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::MA)
                .with_injury(InjuryAttribute::MA)
                .with_marking(MA_MARKING)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(MA_MARKING, marking);
    }

    /// Java: generateForSingleInjuryMarkingsOnlyForOwnPlayer().
    #[test]
    fn generate_for_single_injury_markings_only_for_own_player() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AG).with_apply_to(ApplyTo::Own).with_marking(AG_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_apply_to(ApplyTo::Opponent).with_marking(MA_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(AG_MARKING, marking);
    }

    /// Java: generateForSingleInjuryMarkingsOnlyForOpponent().
    #[test]
    fn generate_for_single_injury_markings_only_for_opponent() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AG).with_apply_to(ApplyTo::Own).with_marking(AG_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_apply_to(ApplyTo::Opponent).with_marking(MA_MARKING).build());

        let marking = generator.generate(&game, &player, &config, false);

        assert_eq!(MA_MARKING, marking);
    }

    /// Java: generateForCombinedSkillAndInjuryMarkings().
    #[test]
    fn generate_for_combined_skill_and_injury_markings() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AV).with_skill(SkillId::Dodge).with_marking(DODGE_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AG).with_skill(SkillId::SneakyGit).with_marking(AG_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLOCK_MARKING, marking);
    }

    /// Java: ignoreInjuryOnlyMarkingsIfTheyAreASubset().
    #[test]
    fn ignore_injury_only_markings_if_they_are_a_subset() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLOCK_MARKING, marking);
    }

    /// Java: ignoreCombinedSkillAndInjuryMarkingsIfGainedOnlyDoesNotMatch().
    #[test]
    fn ignore_combined_skill_and_injury_markings_if_gained_only_does_not_match() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::MA)
                .with_skill(SkillId::Wrestle)
                .with_gained_only(true)
                .with_marking(WRESTLE_MARKING)
                .build(),
        );
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(MA_MARKING, marking);
    }

    /// Java: generateSingleMarkingForMultiStatIncreases().
    #[test]
    fn generate_single_marking_for_multi_stat_increases() {
        let (generator, game, mut player, mut config) = setup();
        replace_gained_with_ag_increases(&mut player, 2);
        config.markings.push(Builder::new().with_skill(SkillId::AgilityIncrease).with_marking(AG_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(AG_MARKING, marking);
    }

    /// Java: generateMarkingMatchingForMultiStatIncreases().
    #[test]
    fn generate_marking_matching_for_multi_stat_increases() {
        let (generator, game, mut player, mut config) = setup();
        replace_gained_with_ag_increases(&mut player, 2);
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::AgilityIncrease)
                .with_skill(SkillId::AgilityIncrease)
                .with_marking(AG_MARKING)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(AG_MARKING, marking);
    }

    /// Java: ignoreMatchingForMultiStatIncreasesIfOnlyOneIsPresent().
    #[test]
    fn ignore_matching_for_multi_stat_increases_if_only_one_is_present() {
        let (generator, game, mut player, mut config) = setup();
        replace_gained_with_ag_increases(&mut player, 1);
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::AgilityIncrease)
                .with_skill(SkillId::AgilityIncrease)
                .with_marking(AG_MARKING)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert!(marking.is_empty());
    }

    /// Java: generateOnlyForNetStatIncreases().
    /// (Java stubs `player.getSkills()` with two +AG increases, but the generator
    /// only reads `getSkillsIncludingTemporaryOnes()`, so the net stat picture is
    /// unchanged: one AG loss. Only the injury-based record applies.)
    #[test]
    fn generate_only_for_net_stat_increases() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::AgilityIncrease).with_marking(AG_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AG).with_marking(AG_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(AG_MARKING, marking);
    }

    /// Java: generateOnlyForNetInjuries().
    /// (Same as above: the `getSkills()` stub of "+MA" is never read; the player's
    /// net MA is a loss, so only the injury record applies — exactly once.)
    #[test]
    fn generate_only_for_net_injuries() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::MovementIncrease).with_marking(MA_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(MA_MARKING, marking);
    }

    /// Java: generateSingleMarkingForMultiInjuries().
    #[test]
    fn generate_single_marking_for_multi_injuries() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(MA_MARKING, marking);
    }

    /// Java: ignoreStatInjuries().
    #[test]
    fn ignore_stat_injuries() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AV).with_marking("Some marking").build());

        let marking = generator.generate(&game, &player, &config, true);

        assert!(marking.is_empty());
    }

    /// Java: generateNigglingMarker().
    #[test]
    fn generate_niggling_marker() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::NI).with_marking(NI_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(NI_MARKING, marking);
    }

    /// Java: generateCombinedInjuryMarkerWhenPlayerWasHurtDuringTheGame().
    #[test]
    fn generate_combined_injury_marker_when_player_was_hurt_during_the_game() {
        let (generator, mut game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::NI).with_marking(NI_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AG).with_marking(AG_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());

        // Java: playerResult.getSeriousInjury() = NECK_INJURY,
        //       playerResult.getSeriousInjuryDecay() = SMASHED_KNEE
        let player_result = PlayerResult {
            serious_injury: Some(SeriousInjuryKind::NeckInjuryAg),
            serious_injury_decay: Some(SeriousInjuryKind::SmashedKneeMa),
            ..Default::default()
        };
        game.game_result.home.player_results.insert(player.id.clone(), player_result);

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(format!("{}{}{}", AG_MARKING, MA_MARKING, NI_MARKING), marking);
    }

    /// Java: generateMarkingMatchingForMultiInjuries().
    #[test]
    fn generate_marking_matching_for_multi_injuries() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::MA)
                .with_injury(InjuryAttribute::MA)
                .with_marking(MA_MARKING)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(MA_MARKING, marking);
    }

    /// Java: ignoreMatchingForMultiInjuriesIfOnlyOneIsPresent().
    #[test]
    fn ignore_matching_for_multi_injuries_if_only_one_is_present() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::AG)
                .with_injury(InjuryAttribute::AG)
                .with_marking(MA_MARKING)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert!(marking.is_empty());
    }

    /// Java: sortInjuriesLastAndAlphabeticallyOtherwise().
    #[test]
    fn sort_injuries_last_and_alphabetically_otherwise() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_skill(SkillId::Dodge).with_marking(BLODGE_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Wrestle).with_marking(WRESTLE_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(MA_MARKING).build());
        config.markings.push(Builder::new().with_injury(InjuryAttribute::AG).with_skill(SkillId::Tackle).with_marking(OTHER_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(
            format!("{}{}{}{}", OTHER_MARKING, WRESTLE_MARKING, BLODGE_MARKING, MA_MARKING),
            marking
        );
    }

    /// Java: ignoreIdenticalMarkingWithGainedOnly().
    #[test]
    fn ignore_identical_marking_with_gained_only() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_gained_only(true).with_marking(OTHER_MARKING).build());
        config.markings.push(Builder::new().with_skill(SkillId::Block).with_marking(BLOCK_MARKING).build());

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLOCK_MARKING, marking);
    }

    /// Java: ignoreIdenticalMarkingWithNoRepetition().
    #[test]
    fn ignore_identical_marking_with_no_repetition() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(Builder::new().with_injury(InjuryAttribute::MA).with_marking(OTHER_MARKING).build());
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::MA)
                .with_marking(MA_MARKING)
                .with_apply_repeatedly(true)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(format!("{}{}", MA_MARKING, MA_MARKING), marking);
    }

    /// Java: generateRepeatedMarking().
    #[test]
    fn generate_repeated_marking() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::MA)
                .with_marking(MA_MARKING)
                .with_apply_repeatedly(true)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(format!("{}{}", MA_MARKING, MA_MARKING), marking);
    }

    /// Java: generateMultiInjuryMarkingOverRepeated().
    #[test]
    fn generate_multi_injury_marking_over_repeated() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::MA)
                .with_marking(OTHER_MARKING)
                .with_apply_repeatedly(true)
                .build(),
        );
        config.markings.push(
            Builder::new()
                .with_injury(InjuryAttribute::MA)
                .with_injury(InjuryAttribute::MA)
                .with_marking(MA_MARKING)
                .with_apply_repeatedly(true)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(MA_MARKING, marking);
    }

    /// Java: generateRepeatedMarkingOnlyOnceIfNotCompletelyApplicable().
    #[test]
    fn generate_repeated_marking_only_once_if_not_completely_applicable() {
        let (generator, game, player, mut config) = setup();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_injury(InjuryAttribute::MA)
                .with_marking(BLOCK_MARKING)
                .with_apply_repeatedly(true)
                .build(),
        );

        let marking = generator.generate(&game, &player, &config, true);

        assert_eq!(BLOCK_MARKING, marking);
    }

    // ------------------------------------------------------------------
    // Rust-only extras (no Java counterpart in MarkerGeneratorTest)
    // ------------------------------------------------------------------

    #[test]
    fn generate_empty_config_returns_empty_string() {
        let gen = MarkerGenerator::new();
        let g = make_game();
        let p = Player::default();
        let config = AutoMarkingConfig::new();
        let result = gen.generate(&g, &p, &config, false);
        assert!(result.is_empty());
    }

    #[test]
    fn generate_with_no_matching_skills_returns_empty() {
        let gen = MarkerGenerator::new();
        let g = make_game();
        let p = Player::default(); // no skills
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
