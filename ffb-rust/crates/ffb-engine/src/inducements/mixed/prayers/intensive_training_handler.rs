/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.IntensiveTrainingHandler`.
/// Extends DialogPrayerHandler — shuffles eligible players, picks one, shows a dialog
/// for the coach to select a primary skill from that player's position categories.
///
/// The player's enhancement is tracked so remove_effect_internal can clean it up, and the chosen
/// player is recoverable from `field_model.prayer_enhancements[PRAYER_NAME]` — which is how the
/// step finds the dialog's subject, mirroring Java carrying the id on the dialog parameter and
/// getting it back in `PrayerDialogSelection(playerId, skill)`.
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;
use ffb_model::factory::skill_factory::SkillFactory;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::random_selection_prayer_handler::{
    init_effect_random_selection, remove_effect_internal_random_selection,
};
use crate::prayer_state::PrayerState;

pub const PRAYER_NAME: &str = "INTENSIVE_TRAINING";

pub fn animation_type() -> AnimationType {
    AnimationType::PRAYER_INTENSIVE_TRAINING
}

/// Java `IntensiveTrainingHandler.createDialog`:
///     Collections.shuffle(players); Player player = players.get(0);
/// `init_effect_random_selection` with `nr_of_players == 1` is exactly that — the bb2020 selector
/// does ONE `collections_shuffle` on the shared `game.collections_rng` and takes element 0 — so the
/// player pick already matched Java before this handler granted anything.
///
/// Returns Java's `handled()`, i.e. `game.getDialogParameter() == null`: true when NO dialog is
/// pending. A dialog is pending exactly when the chosen player has at least one eligible skill;
/// with an empty list Java reports the prayer wasted and moves on.
pub fn init_effect(prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str, selector: &dyn PlayerSelector) -> bool {
    init_effect_random_selection(prayer_state, game, rng, team_id, PRAYER_NAME, 1, selector, &[]);
    // Java: `if (!skills.isEmpty()) showDialog(...) else reports.add(new ReportPrayerWasted(...))`.
    match chosen_player(game) {
        Some(player_id) => eligible_skills(game, &player_id).is_empty(),
        None => true,
    }
}

/// The player this prayer selected, recovered from the enhancement `init_effect` recorded.
/// Java keeps it on the dialog parameter and receives it back with the selection.
pub fn chosen_player(game: &Game) -> Option<String> {
    game.field_model
        .prayer_enhancements
        .get(PRAYER_NAME)
        .and_then(|players| {
            // `nr_of_players == 1`, so there is at most one; `min()` keeps it deterministic
            // regardless of the HashSet's iteration order.
            players.iter().min().cloned()
        })
}

/// Java `IntensiveTrainingHandler.createDialog`, the skill-list half:
///
/// ```text
/// List<SkillCategory> categories = Arrays.asList(player.getPosition().getSkillCategories(false));
/// skillFactory.getSkills().stream()
///     .filter(skill -> skill.eligible()
///         && categories.contains(skill.getCategory())
///         && !Arrays.asList(player.getSkills()).contains(skill)
///         && skill.canBeAssignedTo(player))
///     .sorted(Comparator.comparing(Skill::getName))
/// ```
///
/// `eligible()` is `true` on the base class and no shipped skill overrides it, so it is not modelled.
/// `canBeAssignedTo(player)` is `!conflictsWithAnySkill(player)` — no skill the player already has
/// registers a property this one conflicts with.
pub fn eligible_skills(game: &Game, player_id: &str) -> Vec<SkillId> {
    let Some(player) = game.player(player_id) else { return Vec::new() };
    let categories = &player.skill_categories_normal;
    if categories.is_empty() {
        return Vec::new();
    }
    let rules = game.rules;
    let mut skills: Vec<SkillId> = SkillFactory::new()
        .get_skills()
        .filter(|s| categories.contains(&s.category_and_name_for(rules).0))
        .filter(|s| !player.has_skill(*s))
        .filter(|s| !conflicts_with_any_skill(player, *s, rules))
        .collect();
    // Java: Comparator.comparing(Skill::getName) — by DISPLAY name, not class name.
    skills.sort_by_key(|s| s.category_and_name_for(rules).1);
    skills.dedup();
    skills
}

/// Java `Skill.conflictsWithAnySkill(player)`:
/// `conflictingProperties.stream().anyMatch(player::hasSkillProperty)`
/// Rust models a cancel/conflict registration as the pseudo-property `cancels<Property>`, so a
/// skill's conflicting properties are the `cancels*` entries in its own property list.
fn conflicts_with_any_skill(player: &ffb_model::model::player::Player, skill: SkillId, rules: ffb_model::enums::Rules) -> bool {
    skill.properties_for(rules).iter().any(|prop| {
        prop.strip_prefix("cancels")
            .map(|p| {
                let mut c = p.chars();
                let lowered = match c.next() {
                    Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
                    None => return false,
                };
                player.has_skill_property_in(rules, &lowered)
            })
            .unwrap_or(false)
    })
}

/// Java `applySelection`:
/// `game.getFieldModel().addIntensiveTrainingSkill(selection.getPlayerId(), selection.getSkill())`,
/// which is `player.addTemporarySkills(intensivePrayer().getName(), {skill})`.
pub fn apply_skill_selection(_prayer_state: &mut PrayerState, game: &mut Game, player_id: &str, skill_id: SkillId) {
    if let Some(player) = game.player_mut(player_id) {
        player.add_prayer_skill(PRAYER_NAME, skill_id, None);
    }
}

pub fn remove_effect_internal(game: &mut Game, team_id: &str, selector: &dyn PlayerSelector) {
    remove_effect_internal_random_selection(game, team_id, PRAYER_NAME, selector);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::model::player_status::PlayerStatus;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;
    use crate::inducements::mixed::prayers::player_selector::StubPlayerSelector;
    use crate::inducements::bb2020::prayers::player_selector::PlayerSelector as BB2020Selector;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_reserve_player(game: &mut Game, id: &str) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
        });
        game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
    }

    /// Java offers only skills whose category is in `getSkillCategories(false)`, sorted by name,
    /// so a General-only position (the lineman parity fixture, mirroring
    /// `roster_lineman_parity.xml`) is offered Block first - which is what the harness picks.
    #[test]
    fn eligible_skills_are_general_only_and_block_sorts_first() {
        use ffb_model::enums::SkillCategory;
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        game.player_mut("h1").unwrap().skill_categories_normal = vec![SkillCategory::General];

        let skills = eligible_skills(&game, "h1");
        assert!(!skills.is_empty(), "a General position must have skills to offer");
        assert_eq!(skills.first().copied(), Some(SkillId::Block),
            "Comparator.comparing(Skill::getName) puts Block first among General skills");
        let rules = game.rules;
        assert!(skills.iter().all(|s| s.category_and_name_for(rules).0 == SkillCategory::General),
            "no skill outside the position's normal categories may be offered");
    }

    /// Java filters `!Arrays.asList(player.getSkills()).contains(skill)` - a skill the player
    /// already has is not offered again.
    #[test]
    fn eligible_skills_excludes_skills_the_player_already_has() {
        use ffb_model::enums::SkillCategory;
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        {
            let p = game.player_mut("h1").unwrap();
            p.skill_categories_normal = vec![SkillCategory::General];
            p.add_skill(SkillId::Block);
        }
        let skills = eligible_skills(&game, "h1");
        assert!(!skills.contains(&SkillId::Block));
        assert!(!skills.is_empty(), "the rest of the General list is still on offer");
    }

    /// A position with NO normal categories can be offered nothing, which is Java's
    /// `ReportPrayerWasted` branch - and the reason the lineman fixture had to declare its
    /// categories before this port could have any effect.
    #[test]
    fn a_position_without_categories_is_offered_nothing() {
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        assert!(game.player("h1").unwrap().skill_categories_normal.is_empty());
        assert!(eligible_skills(&game, "h1").is_empty());
    }

    /// Java `applySelection` -> `addIntensiveTrainingSkill` -> `addTemporarySkills(prayer, {skill})`.
    #[test]
    fn apply_skill_selection_grants_the_chosen_skill() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        apply_skill_selection(&mut state, &mut game, "h1", SkillId::Block);
        assert!(game.player("h1").unwrap().has_skill(SkillId::Block));
        // Tracked against the prayer name so `remove_effect_internal` can take it away again.
        assert!(game.player("h1").unwrap().temporary_skill_sources.iter()
            .any(|(src, s)| src == PRAYER_NAME && *s == SkillId::Block));
    }

    #[test]
    fn animation_type_is_correct() {
        assert_eq!(animation_type(), AnimationType::PRAYER_INTENSIVE_TRAINING);
    }

    #[test]
    fn init_effect_returns_true() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        let stub = StubPlayerSelector;
        assert!(init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &stub));
    }

    #[test]
    fn init_effect_marks_prayer_on_selected_player() {
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        let selector = BB2020Selector::new();
        init_effect(&mut state, &mut game, &mut GameRng::new(0), "home", &selector);
        assert!(game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
    }

    // NOTE (test equalization): `apply_selection_is_noop` pruned - the Rust headless
    // apply_selection is a no-op stub, but Java's applySelection applies the dialog-chosen
    // skill (addIntensiveTrainingSkill); asserting noop-ness has no Java counterpart.

    #[test]
    fn remove_effect_clears_enhancement() {
        let mut game = make_game();
        add_reserve_player(&mut game, "h1");
        game.field_model.add_prayer_enhancement("h1", PRAYER_NAME);
        assert!(game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
        remove_effect_internal(&mut game, "home", &StubPlayerSelector);
        assert!(!game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
    }
}
