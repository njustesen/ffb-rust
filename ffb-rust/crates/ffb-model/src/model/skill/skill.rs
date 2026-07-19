/// 1:1 translation of com.fumbbl.ffb.model.skill.Skill.
///
/// Java `Skill` is abstract; Rust uses a concrete struct with the same fields.
/// Types for modifiers (PassModifier, DodgeModifier, etc.) are stubbed as `String`
/// until those crates are ported.
use std::collections::HashMap;
use crate::enums::{SkillCategory, SkillUsageType, DeclareCondition, ReRollSource};
use crate::model::re_rolled_action::ReRolledAction;
use crate::model::property::i_skill_property::ISkillProperty;
use crate::model::skill::skill_value_evaluator::SkillValueEvaluator;

// TODO: replace with proper types when modifier crates are ported
pub type PassModifier = String;
pub type PickupModifier = String;
pub type DodgeModifier = String;
pub type JumpModifier = String;
pub type JumpUpModifier = String;
pub type InterceptionModifier = String;
pub type InjuryModifier = String;
pub type ArmorModifier = String;
pub type CatchModifier = String;
pub type GazeModifier = String;
pub type GoForItModifier = String;
pub type RightStuffModifier = String;
pub type CasualtyModifier = String;
pub type PlayerModifierType = String;
/// TODO: replace with proper TemporaryEnhancements type
pub type TemporaryEnhancements = String;
/// TODO: replace with proper StatBasedRollModifierFactory type
pub type StatBasedRollModifierFactory = String;

/// Boxed skill property (Java `ISkillProperty`).
pub type BoxedSkillProperty = Box<dyn ISkillProperty + Send + Sync>;

/// 1:1 translation of com.fumbbl.ffb.model.skill.Skill.
pub struct Skill {
    pub name: String,
    pub category: SkillCategory,
    pub player_modifiers: Vec<PlayerModifierType>,
    pub pass_modifiers: Vec<PassModifier>,
    pub pickup_modifiers: Vec<PickupModifier>,
    pub dodge_modifiers: Vec<DodgeModifier>,
    pub jump_modifiers: Vec<JumpModifier>,
    pub jump_up_modifiers: Vec<JumpUpModifier>,
    pub interception_modifiers: Vec<InterceptionModifier>,
    pub injury_modifiers: Vec<InjuryModifier>,
    pub armor_modifiers: Vec<ArmorModifier>,
    pub catch_modifiers: Vec<CatchModifier>,
    pub gaze_modifiers: Vec<GazeModifier>,
    pub go_for_it_modifiers: Vec<GoForItModifier>,
    pub right_stuff_modifiers: Vec<RightStuffModifier>,
    pub casualty_modifiers: Vec<CasualtyModifier>,
    pub skill_properties: Vec<BoxedSkillProperty>,
    pub reroll_sources: HashMap<ReRolledAction, ReRollSource>,
    pub default_skill_value: i32,
    pub conflicting_properties: Vec<BoxedSkillProperty>,
    pub skill_usage_type: SkillUsageType,
    pub negative_trait: bool,
    pub enhancements: Option<TemporaryEnhancements>,
    pub stat_based_roll_modifier_factory: Option<StatBasedRollModifierFactory>,
    pub declare_condition: DeclareCondition,
}

impl Skill {
    /// Maps to `Skill(String name, SkillCategory category)`.
    pub fn new(name: impl Into<String>, category: SkillCategory) -> Self {
        Self::with_all(name, category, 0, false, SkillUsageType::Regular)
    }

    /// Maps to `Skill(String name, SkillCategory category, SkillUsageType skillUsageType)`.
    pub fn with_usage_type(name: impl Into<String>, category: SkillCategory, usage_type: SkillUsageType) -> Self {
        Self::with_all(name, category, 0, false, usage_type)
    }

    /// Maps to `Skill(String name, SkillCategory category, int defaultSkillValue)`.
    pub fn with_default_value(name: impl Into<String>, category: SkillCategory, default_skill_value: i32) -> Self {
        Self::with_all(name, category, default_skill_value, false, SkillUsageType::Regular)
    }

    /// Maps to `Skill(String name, SkillCategory category, int defaultSkillValue, SkillUsageType skillUsageType)`.
    pub fn with_default_value_and_usage(
        name: impl Into<String>,
        category: SkillCategory,
        default_skill_value: i32,
        usage_type: SkillUsageType,
    ) -> Self {
        Self::with_all(name, category, default_skill_value, false, usage_type)
    }

    /// Maps to `Skill(String name, SkillCategory category, boolean negativeTrait)`.
    pub fn as_negative_trait(name: impl Into<String>, category: SkillCategory) -> Self {
        Self::with_all(name, category, 0, true, SkillUsageType::Regular)
    }

    /// Maps to `Skill(String name, SkillCategory category, int defaultSkillValue, boolean negativeTrait, SkillUsageType skillUsageType)`.
    pub fn with_all(
        name: impl Into<String>,
        category: SkillCategory,
        default_skill_value: i32,
        negative_trait: bool,
        skill_usage_type: SkillUsageType,
    ) -> Self {
        Skill {
            name: name.into(),
            category,
            player_modifiers: Vec::new(),
            pass_modifiers: Vec::new(),
            pickup_modifiers: Vec::new(),
            dodge_modifiers: Vec::new(),
            jump_modifiers: Vec::new(),
            jump_up_modifiers: Vec::new(),
            interception_modifiers: Vec::new(),
            injury_modifiers: Vec::new(),
            armor_modifiers: Vec::new(),
            catch_modifiers: Vec::new(),
            gaze_modifiers: Vec::new(),
            go_for_it_modifiers: Vec::new(),
            right_stuff_modifiers: Vec::new(),
            casualty_modifiers: Vec::new(),
            skill_properties: Vec::new(),
            reroll_sources: HashMap::new(),
            default_skill_value,
            conflicting_properties: Vec::new(),
            skill_usage_type,
            negative_trait,
            enhancements: None,
            stat_based_roll_modifier_factory: None,
            declare_condition: DeclareCondition::None,
        }
    }

    /// Java `getName()`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Java `getCategory()`.
    pub fn get_category(&self) -> SkillCategory {
        self.category
    }

    /// Java `getDefaultSkillValue()`.
    pub fn get_default_skill_value(&self) -> i32 {
        self.default_skill_value
    }

    /// Java `getSkillUsageType()`.
    pub fn get_skill_usage_type(&self) -> SkillUsageType {
        self.skill_usage_type
    }

    /// Java `isNegativeTrait()`.
    pub fn is_negative_trait(&self) -> bool {
        self.negative_trait
    }

    /// Java `getDeclareCondition()`.
    pub fn get_declare_condition(&self) -> DeclareCondition {
        self.declare_condition
    }

    /// Java `setDeclareCondition(DeclareCondition)`.
    pub fn set_declare_condition(&mut self, declare_condition: DeclareCondition) {
        self.declare_condition = declare_condition;
    }

    /// Java `getEnhancements()`.
    pub fn get_enhancements(&self) -> Option<&TemporaryEnhancements> {
        self.enhancements.as_ref()
    }

    /// Java `setEnhancements(TemporaryEnhancements)`.
    pub fn set_enhancements(&mut self, enhancements: TemporaryEnhancements) {
        self.enhancements = Some(enhancements);
    }

    /// Java `getStatBasedRollModifierFactory()`.
    pub fn get_stat_based_roll_modifier_factory(&self) -> Option<&StatBasedRollModifierFactory> {
        self.stat_based_roll_modifier_factory.as_ref()
    }

    /// Java `setStatBasedRollModifierFactory(StatBasedRollModifierFactory)`.
    pub fn set_stat_based_roll_modifier_factory(&mut self, factory: StatBasedRollModifierFactory) {
        self.stat_based_roll_modifier_factory = Some(factory);
    }

    // ── registerModifier helpers (Java overloaded `registerModifier`) ─────────

    pub fn register_jump_modifier(&mut self, m: JumpModifier) {
        self.jump_modifiers.push(m);
    }

    pub fn register_jump_up_modifier(&mut self, m: JumpUpModifier) {
        self.jump_up_modifiers.push(m);
    }

    pub fn register_pass_modifier(&mut self, m: PassModifier) {
        self.pass_modifiers.push(m);
    }

    pub fn register_pickup_modifier(&mut self, m: PickupModifier) {
        self.pickup_modifiers.push(m);
    }

    pub fn register_dodge_modifier(&mut self, m: DodgeModifier) {
        self.dodge_modifiers.push(m);
    }

    pub fn register_player_modifier(&mut self, m: PlayerModifierType) {
        self.player_modifiers.push(m);
    }

    pub fn register_interception_modifier(&mut self, m: InterceptionModifier) {
        self.interception_modifiers.push(m);
    }

    pub fn register_armor_modifier(&mut self, m: ArmorModifier) {
        self.armor_modifiers.push(m);
    }

    pub fn register_injury_modifier(&mut self, m: InjuryModifier) {
        self.injury_modifiers.push(m);
    }

    pub fn register_catch_modifier(&mut self, m: CatchModifier) {
        self.catch_modifiers.push(m);
    }

    pub fn register_gaze_modifier(&mut self, m: GazeModifier) {
        self.gaze_modifiers.push(m);
    }

    pub fn register_go_for_it_modifier(&mut self, m: GoForItModifier) {
        self.go_for_it_modifiers.push(m);
    }

    pub fn register_right_stuff_modifier(&mut self, m: RightStuffModifier) {
        self.right_stuff_modifiers.push(m);
    }

    pub fn register_casualty_modifier(&mut self, m: CasualtyModifier) {
        self.casualty_modifiers.push(m);
    }

    /// Java `registerProperty(ISkillProperty)`.
    pub fn register_property(&mut self, property: BoxedSkillProperty) {
        self.skill_properties.push(property);
    }

    /// Java `registerRerollSource(ReRolledAction, ReRollSource)`.
    pub fn register_reroll_source(&mut self, action: ReRolledAction, source: ReRollSource) {
        self.reroll_sources.insert(action, source);
    }

    /// Java `registerConflictingProperty(ISkillProperty)`.
    pub fn register_conflicting_property(&mut self, property: BoxedSkillProperty) {
        self.conflicting_properties.push(property);
    }

    // ── query helpers ─────────────────────────────────────────────────────────

    /// Java `hasSkillProperty(ISkillProperty)`.
    pub fn has_skill_property(&self, property_name: &str) -> bool {
        self.skill_properties.iter().any(|p| p.name() == property_name)
    }

    /// Java `getRerollSource(ReRolledAction)`.
    pub fn get_reroll_source(&self, action: &ReRolledAction) -> Option<&ReRollSource> {
        self.reroll_sources.get(action)
    }

    /// Java `getSkillProperties()`.
    pub fn get_skill_properties(&self) -> &[BoxedSkillProperty] {
        &self.skill_properties
    }

    /// Java `getConfusionMessage()`.
    pub fn get_confusion_message(&self) -> &'static str {
        "is confused"
    }

    /// Java `getSkillUseDescription()` — returns null in base class.
    pub fn get_skill_use_description(&self) -> Option<Vec<String>> {
        None
    }

    /// Java `eligible()`.
    pub fn eligible(&self) -> bool {
        true
    }

    /// Java `enhancementSourceName()`.
    pub fn enhancement_source_name(&self) -> &str {
        &self.name
    }

    /// Java `evaluator()` — returns `SkillValueEvaluator.DEFAULT` in the base class.
    pub fn evaluator(&self) -> SkillValueEvaluator {
        SkillValueEvaluator::Default
    }
}

impl Default for Skill {
    fn default() -> Self {
        Skill::new("", SkillCategory::General)
    }
}

impl std::fmt::Display for Skill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Debug for Skill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Skill").field("name", &self.name).field("category", &self.category).finish()
    }
}

impl PartialEq for Skill {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Skill {}

impl std::hash::Hash for Skill {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialOrd for Skill {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Skill {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{SkillCategory, SkillUsageType, DeclareCondition};

    #[test]
    fn new_sets_name_and_category() {
        let s = Skill::new("Block", SkillCategory::General);
        assert_eq!(s.get_name(), "Block");
        assert_eq!(s.get_category(), SkillCategory::General);
    }

    #[test]
    fn new_defaults_to_regular_usage_type() {
        let s = Skill::new("Dodge", SkillCategory::Agility);
        assert_eq!(s.get_skill_usage_type(), SkillUsageType::Regular);
    }

    #[test]
    fn new_default_skill_value_is_zero() {
        let s = Skill::new("Block", SkillCategory::General);
        assert_eq!(s.get_default_skill_value(), 0);
    }

    #[test]
    fn negative_trait_false_by_default() {
        let s = Skill::new("Block", SkillCategory::General);
        assert!(!s.is_negative_trait());
    }

    #[test]
    fn as_negative_trait_sets_flag() {
        let s = Skill::as_negative_trait("BoneHead", SkillCategory::Extraordinary);
        assert!(s.is_negative_trait());
    }

    #[test]
    fn with_default_value_stores_value() {
        let s = Skill::with_default_value("Mighty Blow", SkillCategory::Strength, 1);
        assert_eq!(s.get_default_skill_value(), 1);
    }

    #[test]
    fn with_usage_type_stores_type() {
        let s = Skill::with_usage_type("Pro", SkillCategory::General, SkillUsageType::OncePerTurn);
        assert_eq!(s.get_skill_usage_type(), SkillUsageType::OncePerTurn);
    }

    #[test]
    fn declare_condition_defaults_to_none() {
        let s = Skill::new("Block", SkillCategory::General);
        assert_eq!(s.get_declare_condition(), DeclareCondition::None);
    }

    #[test]
    fn set_declare_condition_round_trip() {
        let mut s = Skill::new("Block", SkillCategory::General);
        s.set_declare_condition(DeclareCondition::Standing);
        assert_eq!(s.get_declare_condition(), DeclareCondition::Standing);
    }

    #[test]
    fn equality_based_on_name_only() {
        let a = Skill::new("Block", SkillCategory::General);
        let b = Skill::new("Block", SkillCategory::Strength);
        assert_eq!(a, b);
        let c = Skill::new("Dodge", SkillCategory::General);
        assert_ne!(a, c);
    }

    #[test]
    fn ordering_by_name() {
        let a = Skill::new("Block", SkillCategory::General);
        let b = Skill::new("Dodge", SkillCategory::Agility);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn eligible_returns_true_by_default() {
        let s = Skill::new("Block", SkillCategory::General);
        assert!(s.eligible());
    }

    #[test]
    fn get_confusion_message_returns_non_empty() {
        let s = Skill::new("BoneHead", SkillCategory::Extraordinary);
        assert!(!s.get_confusion_message().is_empty());
    }

    #[test]
    fn register_and_query_reroll_source() {
        use crate::enums::ReRollSource;
        let mut s = Skill::new("Dodge", SkillCategory::Agility);
        let action = ReRolledAction::new("DODGE");
        let source = ReRollSource::new("Dodge");
        s.register_reroll_source(action.clone(), source);
        assert!(s.get_reroll_source(&action).is_some());
        let missing = ReRolledAction::new("BLOCK");
        assert!(s.get_reroll_source(&missing).is_none());
    }

    #[test]
    fn enhancements_none_by_default() {
        let s = Skill::new("Block", SkillCategory::General);
        assert!(s.get_enhancements().is_none());
    }

    #[test]
    fn set_enhancements_stores_value() {
        let mut s = Skill::new("Block", SkillCategory::General);
        s.set_enhancements("some_enhancement".to_string());
        assert!(s.get_enhancements().is_some());
    }

    #[test]
    fn enhancement_source_name_returns_skill_name() {
        let s = Skill::new("Mighty Blow", SkillCategory::Strength);
        assert_eq!(s.enhancement_source_name(), "Mighty Blow");
    }

    #[test]
    fn display_returns_name() {
        let s = Skill::new("Block", SkillCategory::General);
        assert_eq!(format!("{}", s), "Block");
    }

    #[test]
    fn get_skill_use_description_returns_none_by_default() {
        let s = Skill::new("Block", SkillCategory::General);
        assert!(s.get_skill_use_description().is_none());
    }

    #[test]
    fn evaluator_defaults_to_default_variant() {
        let s = Skill::new("Block", SkillCategory::General);
        assert_eq!(s.evaluator(), SkillValueEvaluator::Default);
    }

    #[test]
    fn register_modifier_lists_grow() {
        let mut s = Skill::new("Block", SkillCategory::General);
        s.register_pass_modifier("pass_mod".to_string());
        s.register_dodge_modifier("dodge_mod".to_string());
        assert_eq!(s.pass_modifiers.len(), 1);
        assert_eq!(s.dodge_modifiers.len(), 1);
    }

    #[test]
    fn with_all_stores_negative_trait_and_usage() {
        let s = Skill::with_all("WildAnimal", SkillCategory::Extraordinary, 0, true, SkillUsageType::OncePerTurn);
        assert!(s.is_negative_trait());
        assert_eq!(s.get_skill_usage_type(), SkillUsageType::OncePerTurn);
    }
}
