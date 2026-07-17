use ffb_model::enums::Rules;
use ffb_model::model::{Game, Player};
use crate::modifiers::catch_modifier::CatchModifier;
use crate::modifiers::go_for_it_context::GoForItContext;
use crate::modifiers::interception_modifier::InterceptionModifier;
use crate::modifiers::pass_context::PassContext;
use crate::modifiers::pass_modifier::PassModifier;
use crate::modifiers::dodge_modifier::DodgeModifier;
use crate::modifiers::pickup_modifier::PickupModifier;
use crate::modifiers::jump_modifier::JumpModifier;
use crate::modifiers::jump_up_modifier::JumpUpModifier;
use crate::modifiers::gaze_modifier::GazeModifier;
use crate::modifiers::go_for_it_modifier::GoForItModifier;
use crate::modifiers::right_stuff_modifier::RightStuffModifier;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::injury_modifier::InjuryModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.ModifierAggregator.
///
/// Java's version is initialized once per `Game` and stores both a `SkillFactory` and a `Game`
/// reference so its no-arg getters can look both up internally. Holding a `&Game`/`&SkillFactory`
/// reference on this struct would tie its lifetime to a single game instance for no benefit, so
/// each getter takes the game/player/context/rules it needs as a parameter instead.
///
/// **Correction (Phase AD):** this struct's prior doc comment claimed "Rust has no SkillFactory
/// translation yet" — stale. `ffb_model::factory::skill_factory::SkillFactory` is a real, fully
/// translated name/class registry (`for_name`/`for_class_name`/`get_skills`). It carries no
/// per-skill modifier data itself (unlike Java's `Skill` subclasses, which register modifier
/// objects in `postConstruct()`), so each getter below reuses the equivalent static catalog
/// (`find_registered_modifiers`) already added to the corresponding `*ModifierFactory` — the
/// same per-skill data `find_skill_modifiers`-style dispatch already encodes for live gameplay,
/// restructured from "this player's skills" to "every registered skill for this ruleset" to
/// match Java's `skillFactory.getSkills().flatMap(skill -> skill.getXxxModifiers())` union
/// semantics (raw registered objects, not predicate-filtered).
///
/// The card half of the merge (`UtilCards::find_all_cards` + `card_roll_modifiers`) was wired in
/// Phase AC. `Armour`/`Injury` getters remain empty: Java's real per-skill registrants for those
/// two categories (~20 skills across bb2016/bb2020/bb2025/mixed) were not individually audited in
/// this phase — a larger, separately-scoped task; see `SESSION.md` Phase AD.
pub struct ModifierAggregator {}

impl ModifierAggregator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init(&mut self, _game: &Game) {
        // No per-instance state to initialize; SkillFactory/rules are passed to each getter.
    }

    /// Java: `ModifierAggregator.getCatchModifiers()`.
    pub fn get_catch_modifiers(&self, rules: Rules) -> Vec<CatchModifier> {
        crate::modifiers::catch_modifier_factory::CatchModifierFactory::find_registered_modifiers(rules)
    }

    /// Java: `ModifierAggregator.getInterceptionModifiers()`.
    pub fn get_interception_modifiers(&self, game: &Game, interceptor: &Player) -> Vec<InterceptionModifier> {
        crate::modifiers::card_roll_modifiers::find_interception_card_modifiers(game, interceptor)
    }

    /// Java: `ModifierAggregator.getPassModifiers()`.
    pub fn get_pass_modifiers(&self, context: &PassContext<'_>) -> Vec<PassModifier> {
        crate::modifiers::card_roll_modifiers::find_pass_card_modifiers(context)
    }

    /// Java: `ModifierAggregator.getDodgeModifiers()`.
    pub fn get_dodge_modifiers(&self, rules: Rules) -> Vec<DodgeModifier> {
        crate::modifiers::dodge_modifier_factory::DodgeModifierFactory::find_registered_modifiers(rules)
    }

    /// Java: `ModifierAggregator.getPickupModifiers()`.
    pub fn get_pickup_modifiers(&self, rules: Rules) -> Vec<PickupModifier> {
        crate::modifiers::pickup_modifier_factory::PickupModifierFactory::find_registered_modifiers(rules)
    }

    /// Java: `ModifierAggregator.getJumpModifiers()`.
    pub fn get_jump_modifiers(&self, rules: Rules) -> Vec<JumpModifier> {
        crate::modifiers::jump_modifier_factory::JumpModifierFactory::find_registered_modifiers(rules)
    }

    /// Java: `ModifierAggregator.getJumpUpModifiers()`. No skill/card in the Java source
    /// registers a `JumpUpModifier` (confirmed via source read — see
    /// `jump_up_modifier_factory.rs`'s own doc comment), so this correctly stays empty.
    pub fn get_jump_up_modifiers(&self) -> Vec<JumpUpModifier> {
        Vec::new()
    }

    /// Java: `ModifierAggregator.getGazeModifiers()`. No skill/card in the Java source
    /// registers a `GazeModifier` (confirmed via source read), so this correctly stays empty.
    pub fn get_gaze_modifiers(&self) -> Vec<GazeModifier> {
        Vec::new()
    }

    /// Java: `ModifierAggregator.getGoForItModifiers()`.
    pub fn get_go_for_it_modifiers(&self, context: &GoForItContext<'_>) -> Vec<GoForItModifier> {
        crate::modifiers::card_roll_modifiers::find_go_for_it_card_modifiers(context)
    }

    /// Java: `ModifierAggregator.getRightStuffModifiers()`.
    pub fn get_right_stuff_modifiers(&self, rules: Rules) -> Vec<RightStuffModifier> {
        crate::modifiers::right_stuff_modifier_factory::RightStuffModifierFactory::find_registered_modifiers(rules)
    }

    /// Java: `ModifierAggregator.getArmourModifiers()`. Deferred — see struct doc.
    pub fn get_armour_modifiers(&self) -> Vec<Box<dyn ArmorModifier>> {
        Vec::new()
    }

    /// Java: `ModifierAggregator.getInjuryModifiers()`. Deferred — see struct doc.
    pub fn get_injury_modifiers(&self) -> Vec<Box<dyn InjuryModifier>> {
        Vec::new()
    }

    /// Java: `ModifierAggregator.getCasualtyModifiers()`.
    pub fn get_casualty_modifiers(&self) -> Vec<crate::modifiers::modifiers::Modifier> {
        crate::modifiers::casualty_modifier_factory::CasualtyModifierFactory::find_registered_modifiers()
    }
}

impl Default for ModifierAggregator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PassingDistance, Rules};
    use ffb_model::model::Team;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_catch_modifiers_includes_extra_arms_and_diving_catch() {
        let agg = ModifierAggregator::new();
        let mods = agg.get_catch_modifiers(Rules::Bb2025);
        assert_eq!(mods.len(), 2);
    }

    #[test]
    fn get_dodge_modifiers_bb2016_includes_edition_specific_skills() {
        let agg = ModifierAggregator::new();
        let mods = agg.get_dodge_modifiers(Rules::Bb2016);
        assert_eq!(mods.len(), 4);
    }

    #[test]
    fn get_pickup_modifiers_includes_extra_arms() {
        let agg = ModifierAggregator::new();
        assert_eq!(agg.get_pickup_modifiers(Rules::Bb2025).len(), 1);
    }

    #[test]
    fn get_jump_modifiers_bb2025_includes_very_long_legs_and_leap() {
        let agg = ModifierAggregator::new();
        assert_eq!(agg.get_jump_modifiers(Rules::Bb2025).len(), 2);
    }

    #[test]
    fn get_jump_up_and_gaze_modifiers_stay_empty() {
        let agg = ModifierAggregator::new();
        assert!(agg.get_jump_up_modifiers().is_empty());
        assert!(agg.get_gaze_modifiers().is_empty());
    }

    #[test]
    fn get_right_stuff_modifiers_bb2016_includes_swoop() {
        let agg = ModifierAggregator::new();
        assert_eq!(agg.get_right_stuff_modifiers(Rules::Bb2016).len(), 1);
        assert!(agg.get_right_stuff_modifiers(Rules::Bb2025).is_empty());
    }

    #[test]
    fn get_casualty_modifiers_includes_decay() {
        let agg = ModifierAggregator::new();
        let mods = agg.get_casualty_modifiers();
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "Decay");
    }

    #[test]
    fn get_interception_modifiers_empty_when_no_cards_active() {
        let game = make_game();
        let interceptor = Player::default();
        let agg = ModifierAggregator::new();
        assert!(agg.get_interception_modifiers(&game, &interceptor).is_empty());
    }

    #[test]
    fn get_interception_modifiers_finds_magic_gloves() {
        let mut game = make_game();
        let interceptor = Player { id: "i1".into(), ..Default::default() };
        game.field_model.add_card("i1", ffb_model::inducement::card::Card::new("Magic Gloves of Jark Longarm", None::<&str>));
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Magic Gloves of Jark Longarm", None::<&str>));

        let agg = ModifierAggregator::new();
        let mods = agg.get_interception_modifiers(&game, &interceptor);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), -1);
    }

    #[test]
    fn get_pass_modifiers_finds_gromskull() {
        let mut game = make_game();
        let passer = Player { id: "p1".into(), ..Default::default() };
        game.field_model.add_card("p1", ffb_model::inducement::card::Card::new("Gromskull's Exploding Runes", None::<&str>));
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Gromskull's Exploding Runes", None::<&str>));

        let agg = ModifierAggregator::new();
        let ctx = PassContext::new(&game, &passer, PassingDistance::ShortPass, false);
        let mods = agg.get_pass_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), 1);
    }

    #[test]
    fn get_go_for_it_modifiers_finds_greased_shoes() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_away.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Greased Shoes", None::<&str>));

        let player = Player::default();
        let agg = ModifierAggregator::new();
        let ctx = GoForItContext::new(&game, &player);
        let mods = agg.get_go_for_it_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), 3);
    }

    #[test]
    fn get_armour_and_injury_modifiers_stay_empty() {
        let agg = ModifierAggregator::new();
        assert!(agg.get_armour_modifiers().is_empty());
        assert!(agg.get_injury_modifiers().is_empty());
    }
}
