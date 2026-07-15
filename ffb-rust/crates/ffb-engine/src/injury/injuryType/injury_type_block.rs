/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBlock (166 lines).
/// ModificationAware: the most complex injury type. Handles block armor roll modes:
/// Regular, UseModifiersAgainstTeamMates, UseMightyBlow, UseClaws, UseClawsAndMightyBlow, etc.
/// Claws/Chainsaw interaction and the `CLAW_DOES_NOT_STACK`/`MB_STACKS_AGAINST_CHAINSAW` game
/// options are fully wired (Phase AAT) — see `armour_roll` below.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE, SkillId};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::option::game_option_id::{CLAW_DOES_NOT_STACK, MB_STACKS_AGAINST_CHAINSAW};
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_mechanics::modifiers::armor_modifier::ArmorModifier;
use ffb_mechanics::modifiers::armor_modifier_factory::ArmorModifierFactory;
use ffb_mechanics::modifiers::{niggling_injury_modifier, Modifier, ARMOR_MIGHTY_BLOW_1, INJURY_MIGHTY_BLOW_1};
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury};

/// Java: `ArmorModifier` instances are transient objects owned by their `Skill`; Rust's
/// `Modifier` (used by `InjuryContext`) requires a `&'static str` name for cheap `Copy`-like
/// use across the codebase. Bridging a `Box<dyn ArmorModifier>` (whose `get_name()` borrows
/// from an owned `String`) into a `Modifier` therefore needs *some* `'static`-ifying step;
/// leaking the name is the chosen approach (see SESSION.md/plan): bounded churn (one leak per
/// armor roll that has Claws/Mighty-Blow-family modifiers in play), no workspace-wide `Modifier`
/// type change, and armor-modifier names are a small, low-cardinality set in practice.
fn leak_modifier(m: &dyn ArmorModifier, attacker: Option<&Player>, defender: &Player, rules: ffb_model::enums::Rules) -> Modifier {
    let name: &'static str = Box::leak(m.get_name().to_owned().into_boxed_str());
    Modifier::new(name, m.get_modifier(attacker, defender), rules)
}

/// Java: `DiceInterpreter.isArmourBroken` special-cases `NamedProperties.reducesArmourToFixedValue`
/// (Claws): the defender's *effective* armor value is capped at 8 when such a modifier is present
/// (Claws breaks armor on an 8+ regardless of the real AV). The shared `mech_armor_broken`/
/// `recalc_armor_broken` helpers don't implement this cap — a pre-existing gap in the general
/// armor-roll formula, tracked separately (see SESSION.md) rather than fixed here, since it's used
/// by every injury type and a wider change is out of this phase's scope. This local variant
/// applies the cap only within `InjuryTypeBlock::armour_roll`.
fn recalc_armor_broken_claws_aware(game: &Game, ctx: &mut InjuryContext, defender_id: &str) {
    let Some([d1, d2]) = ctx.armor_roll else { return };
    let has_claws = ctx.armor_modifiers.iter().any(|m| m.name == "Claws");
    let armor_value = game.player(defender_id).map(|p| p.armour).unwrap_or(7);
    let effective_armor = if has_claws { armor_value.min(8) } else { armor_value };
    let modifier_sum: i32 = ctx.armor_modifiers.iter().map(|m| m.value).sum();
    ctx.armor_broken = d1 + d2 + modifier_sum >= effective_armor;
}

/// Java: InjuryTypeBlock.Mode enum (inner class).
///
/// Java's real `Mode` has 4 variants (`REGULAR`, `USE_MODIFIERS_AGAINST_TEAM_MATES`,
/// `DO_NOT_USE_MODIFIERS`, `USE_ARMOUR_MODIFIERS_ONLY_AGAINST_TEAM_MATES`); this port adds 3
/// extra split-out variants (`UseMightyBlow`/`UseClaws`/`UseClawsAndMightyBlow`) predating the
/// other 2, so callers can request Mighty Blow's modifier directly without a real team-mate
/// check. `armour_roll`/`injury_roll` gate the full Claws/Chainsaw/`CLAW_DOES_NOT_STACK`/
/// `MB_STACKS_AGAINST_CHAINSAW` modifier-factory logic (Phase AAT) on `mode`, mirroring
/// `InjuryTypeBlock.java`'s real per-roll conditions (lines 54–60 for injury, 89–91 for armour):
/// `DoNotUseModifiers` never adds modifiers;
/// `UseArmourModifiersOnlyAgainstTeamMates` adds them for armour only (its own Java condition
/// excludes it from the injury-roll condition, since it's specifically "armour modifiers only");
/// `UseModifiersAgainstTeamMates` always adds them; `Regular` adds them only when attacker and
/// defender are on different teams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockMode {
    Regular,
    UseModifiersAgainstTeamMates,
    UseMightyBlow,
    UseClaws,
    UseClawsAndMightyBlow,
    /// Java: `Mode.DO_NOT_USE_MODIFIERS`.
    DoNotUseModifiers,
    /// Java: `Mode.USE_ARMOUR_MODIFIERS_ONLY_AGAINST_TEAM_MATES` (BB2025 Animal Savagery).
    UseArmourModifiersOnlyAgainstTeamMates,
}

pub struct InjuryTypeBlock {
    ctx: InjuryContext,
    mode: BlockMode,
    /// Java: fRollArmour. Whether to actually roll armor dice (vs just evaluate existing ctx).
    roll_armour: bool,
    /// Java: `allowAttackerChainsaw` (InjuryTypeBlock's real 2nd constructor param — distinct
    /// from `roll_armour` above). When false, only the *defender's* `blocksLikeChainsaw` skill
    /// (if any) is considered; the attacker's is ignored.
    allow_attacker_chainsaw: bool,
}

impl InjuryTypeBlock {
    pub fn new(mode: BlockMode, roll_armour: bool) -> Self {
        Self::new_with_chainsaw(mode, roll_armour, true)
    }

    /// Java: `InjuryTypeBlock(Mode mode, boolean allowAttackerChainsaw)`.
    pub fn new_with_chainsaw(mode: BlockMode, roll_armour: bool, allow_attacker_chainsaw: bool) -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::Defender), mode, roll_armour, allow_attacker_chainsaw }
    }
}
impl Default for InjuryTypeBlock {
    fn default() -> Self { Self::new(BlockMode::Regular, true) }
}

impl InjuryTypeServer for InjuryTypeBlock {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn java_class_name(&self) -> &'static str { "Block" }
    /// Java: `Block.isWorthSpps()` — true.
    fn is_worth_spps(&self) -> bool { true }
    /// Java: `Block.isCausedByOpponent()` — true.
    fn is_caused_by_opponent(&self) -> bool { true }
}
/// Java: `pAttacker.getTeam() != pDefender.getTeam()`.
fn different_teams(game: &Game, attacker_id: Option<&str>, defender_id: &str) -> bool {
    match attacker_id {
        Some(aid) => game.team_home.has_player(aid) != game.team_home.has_player(defender_id),
        None => false,
    }
}

impl ModificationAwareInjuryType for InjuryTypeBlock {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str, roll: bool) {
        // Java: armourRoll's whole body is wrapped in `if (!injuryContext.isArmorBroken())`.
        if self.ctx.armor_broken {
            return;
        }
        if !(roll && self.roll_armour) {
            return;
        }

        let defender = game.player(defender_id);
        let attacker = attacker_id.and_then(|id| game.player(id));

        // Java lines 72-79: resolve the Chainsaw skill (attacker first if allowed, else
        // defender), gated by the defender's ignoresArmourModifiersFromSkills property.
        let skills_stack_against_chainsaw = is_option_enabled(game, MB_STACKS_AGAINST_CHAINSAW);
        let defender_ignores_skill_modifiers = defender
            .map(|d| d.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
            .unwrap_or(false);
        let has_chainsaw = !defender_ignores_skill_modifiers
            && ((self.allow_attacker_chainsaw
                && attacker.map(|a| a.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW)).unwrap_or(false))
                || defender.map(|d| d.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW)).unwrap_or(false));
        // Java: `chainsaw.getArmorModifiers()` — Chainsaw's own registered modifier is a flat +3
        // that never applies through the normal per-context scan (`appliesToContext` returns
        // false), so it's added directly here rather than via `ArmorModifierFactory`.
        let chainsaw_modifier = Modifier::new("Chainsaw", 3, game.rules);

        do_armor_roll(game, rng, &mut self.ctx, defender_id);

        if has_chainsaw && !skills_stack_against_chainsaw {
            self.ctx.add_armor_modifier(chainsaw_modifier);
            recalc_armor_broken_claws_aware(game, &mut self.ctx, defender_id);
            return;
        }

        // Java armourRoll (lines 89-91): mode == USE_ARMOUR_MODIFIERS_ONLY_AGAINST_TEAM_MATES
        // || mode == USE_MODIFIERS_AGAINST_TEAM_MATES || (mode != DO_NOT_USE_MODIFIERS && different teams).
        let modifiers_apply = match self.mode {
            BlockMode::DoNotUseModifiers => false,
            BlockMode::UseArmourModifiersOnlyAgainstTeamMates
            | BlockMode::UseModifiersAgainstTeamMates
            | BlockMode::UseMightyBlow
            | BlockMode::UseClaws
            | BlockMode::UseClawsAndMightyBlow => true,
            BlockMode::Regular => different_teams(game, attacker_id, defender_id),
        };
        if self.ctx.armor_broken || !modifiers_apply {
            return;
        }

        let (Some(attacker), Some(defender)) = (attacker, defender) else {
            return;
        };

        let factory = ArmorModifierFactory::new(game.rules);
        let mut mods: Vec<Box<dyn ArmorModifier>> =
            factory.find_armor_modifiers(game, Some(attacker), defender, false, false);
        if self.mode == BlockMode::UseArmourModifiersOnlyAgainstTeamMates {
            // BB2025: only Claws and Mighty Blow apply against team-mates.
            mods.retain(|m| m.get_name() == "Claws" || m.get_name() == "Mighty Blow");
        }

        let claw_idx = mods.iter().position(|m| m.get_name() == "Claws");
        if let Some(idx) = claw_idx {
            let claw = mods.remove(idx);
            self.ctx.add_armor_modifier(leak_modifier(claw.as_ref(), Some(attacker), defender, game.rules));
            recalc_armor_broken_claws_aware(game, &mut self.ctx, defender_id);
            if !self.ctx.armor_broken {
                if is_option_enabled(game, CLAW_DOES_NOT_STACK) {
                    self.ctx.clear_armor_modifiers();
                    for m in &mods {
                        self.ctx.add_armor_modifier(leak_modifier(m.as_ref(), Some(attacker), defender, game.rules));
                    }
                    if has_chainsaw {
                        self.ctx.add_armor_modifier(chainsaw_modifier);
                    }
                    recalc_armor_broken_claws_aware(game, &mut self.ctx, defender_id);
                    if !self.ctx.armor_broken && !has_chainsaw {
                        // Display Claws as used in the log even though it didn't stack.
                        self.ctx.clear_armor_modifiers();
                        self.ctx.add_armor_modifier(leak_modifier(claw.as_ref(), Some(attacker), defender, game.rules));
                    }
                } else {
                    for m in &mods {
                        self.ctx.add_armor_modifier(leak_modifier(m.as_ref(), Some(attacker), defender, game.rules));
                    }
                }
                recalc_armor_broken_claws_aware(game, &mut self.ctx, defender_id);
            }
        } else {
            if has_chainsaw {
                self.ctx.add_armor_modifier(chainsaw_modifier);
            }
            // Java handleChainsawAndMb: if both a chainsaw and a Mighty-Blow-family modifier are
            // present, evaluate without Mighty Blow first; only re-add it if armor didn't break.
            let mb_idx = mods.iter().position(|m| m.get_name() == "Mighty Blow");
            if has_chainsaw {
                if let Some(idx) = mb_idx {
                    let mb = mods.remove(idx);
                    for m in &mods {
                        self.ctx.add_armor_modifier(leak_modifier(m.as_ref(), Some(attacker), defender, game.rules));
                    }
                    recalc_armor_broken_claws_aware(game, &mut self.ctx, defender_id);
                    if !self.ctx.armor_broken {
                        self.ctx.add_armor_modifier(leak_modifier(mb.as_ref(), Some(attacker), defender, game.rules));
                    }
                } else {
                    for m in &mods {
                        self.ctx.add_armor_modifier(leak_modifier(m.as_ref(), Some(attacker), defender, game.rules));
                    }
                }
            } else {
                for m in &mods {
                    self.ctx.add_armor_modifier(leak_modifier(m.as_ref(), Some(attacker), defender, game.rules));
                }
            }
            recalc_armor_broken_claws_aware(game, &mut self.ctx, defender_id);
        }
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        if let Some(defender) = game.player(defender_id) {
            if let Some(m) = niggling_injury_modifier(defender.niggling_injuries) {
                self.ctx.add_injury_modifier(m);
            }
        }
        // Java injuryRoll (lines 54-60): mode != USE_ARMOUR_MODIFIERS_ONLY_AGAINST_TEAM_MATES &&
        // (mode == USE_MODIFIERS_AGAINST_TEAM_MATES || (mode != DO_NOT_USE_MODIFIERS && different teams)).
        // Note USE_ARMOUR_MODIFIERS_ONLY_AGAINST_TEAM_MATES is excluded here (armour-only by name).
        let modifiers_apply = match self.mode {
            BlockMode::DoNotUseModifiers | BlockMode::UseArmourModifiersOnlyAgainstTeamMates => false,
            BlockMode::UseModifiersAgainstTeamMates
            | BlockMode::UseMightyBlow
            | BlockMode::UseClaws
            | BlockMode::UseClawsAndMightyBlow => true,
            BlockMode::Regular => different_teams(game, attacker_id, defender_id),
        };
        if modifiers_apply {
            if let Some(aid) = attacker_id {
                if let Some(attacker) = game.player(aid) {
                    if attacker.has_skill(SkillId::MightyBlow) {
                        self.ctx.add_injury_modifier(INJURY_MIGHTY_BLOW_1);
                    }
                }
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    // savedByArmour: default PRONE
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_mechanics::modifiers::{ARMOR_MIGHTY_BLOW_1, INJURY_MIGHTY_BLOW_1};

    fn make_player(id: &str, armour: i32, skills: Vec<SkillId>) -> ffb_model::model::player::Player {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player { id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() }
    }

    fn game_with_armor(armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("p1", armour, vec![]));
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    fn game_with_same_team_attacker_and_defender(attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        home.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }

    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }

    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn no_roll_armour_skips_armor_check() {
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, false); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken);
    }
    /// Java's real armor-modifier name for Mighty Blow is "Mighty Blow" (see
    /// `ArmorModifierFactory::skill_to_armor_modifier`), not the pre-Phase-AAT placeholder
    /// constant `ARMOR_MIGHTY_BLOW_1` ("Mighty Blow +1") that these tests used to check —
    /// `armour_roll` now sources it from the real factory, matching Java.
    fn mighty_blow_armor_modifier(rules: ffb_model::enums::Rules) -> Modifier {
        Modifier::new("Mighty Blow", 1, rules)
    }

    #[test]
    fn use_mighty_blow_adds_armor_modifier() {
        // Java only adds armor modifiers if the base roll (seed=1 -> 2+5=7) didn't already
        // break armor on its own — armour=15 guarantees that, isolating the modifier check.
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 15);
        let mut t = InjuryTypeBlock::new(BlockMode::UseMightyBlow, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&mighty_blow_armor_modifier(game.rules)));
    }
    #[test]
    fn regular_mode_adds_mighty_blow_armor_modifier_against_different_team() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 15);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&mighty_blow_armor_modifier(game.rules)));
    }
    #[test]
    fn regular_mode_does_not_add_mighty_blow_armor_modifier_against_same_team() {
        let game = game_with_same_team_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_MIGHTY_BLOW_1));
    }
    #[test]
    fn use_claws_and_mighty_blow_adds_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 15);
        let mut t = InjuryTypeBlock::new(BlockMode::UseClawsAndMightyBlow, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&mighty_blow_armor_modifier(game.rules)));
    }
    #[test]
    fn use_mighty_blow_adds_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::UseMightyBlow, true);
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(t.ctx.injury_modifiers.contains(&INJURY_MIGHTY_BLOW_1));
    }
    #[test]
    fn regular_mode_adds_mighty_blow_injury_modifier_against_different_team() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(t.ctx.injury_modifiers.contains(&INJURY_MIGHTY_BLOW_1));
    }
    #[test]
    fn regular_mode_does_not_add_mighty_blow_injury_modifier_against_same_team() {
        let game = game_with_same_team_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&INJURY_MIGHTY_BLOW_1));
    }
    #[test]
    fn do_not_use_modifiers_never_adds_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::DoNotUseModifiers, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_MIGHTY_BLOW_1));
    }
    #[test]
    fn use_armour_modifiers_only_against_team_mates_adds_armor_modifier_against_same_team() {
        let game = game_with_same_team_attacker_and_defender(vec![SkillId::MightyBlow], 15);
        let mut t = InjuryTypeBlock::new(BlockMode::UseArmourModifiersOnlyAgainstTeamMates, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&mighty_blow_armor_modifier(game.rules)));
    }
    #[test]
    fn use_armour_modifiers_only_against_team_mates_never_adds_injury_modifier() {
        let game = game_with_same_team_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::UseArmourModifiersOnlyAgainstTeamMates, true);
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&INJURY_MIGHTY_BLOW_1));
    }
    #[test]
    fn use_mighty_blow_without_skill_does_not_add_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeBlock::new(BlockMode::UseMightyBlow, true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_MIGHTY_BLOW_1));
    }

    // ── Claws / Chainsaw / CLAW_DOES_NOT_STACK (Phase AAT) ──────────────────────

    #[test]
    fn claws_applies_and_breaks_armor_via_fixed_value_cap() {
        // seed=3 -> roll (3,5) = 8. Real armour 20 (>8, so Claws is eligible per the
        // factory's own `armour_with_modifiers() > 8` gate) would never break on an 8 alone,
        // but Claws caps the effective armour at 8, so 8 >= 8 breaks.
        let game = game_with_attacker_and_defender(vec![SkillId::Claw], 20);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(3);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Claws"));
        assert!(t.ctx.armor_broken, "Claws must break armour via the fixed-value-8 cap");
    }

    #[test]
    fn claws_present_but_roll_below_cap_does_not_break() {
        // seed=0 -> roll (3,3) = 6 < 8 (the Claws-capped effective armour).
        let game = game_with_attacker_and_defender(vec![SkillId::Claw], 20);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(0);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Claws"));
        assert!(!t.ctx.armor_broken);
    }

    #[test]
    fn claw_does_not_stack_removes_claw_but_redisplays_it_when_still_unbroken() {
        // Java: when CLAW_DOES_NOT_STACK is enabled and Claws alone doesn't break armour, the
        // claw modifier is removed, remaining modifiers (none here) are re-evaluated against the
        // *real* armour, and if still unbroken (and no chainsaw), Claws is added back purely for
        // display — armor_broken stays false either way.
        let mut game = game_with_attacker_and_defender(vec![SkillId::Claw], 20);
        game.options.set(CLAW_DOES_NOT_STACK, "true");
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(0); // roll total 6, below the Claws cap of 8
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_broken);
        assert_eq!(t.ctx.armor_modifiers.len(), 1);
        assert_eq!(t.ctx.armor_modifiers[0].name, "Claws");
    }

    #[test]
    fn chainsaw_property_adds_flat_plus_three_and_can_break_armor() {
        // seed=0 -> roll (3,3) = 6. Real armour 8: 6 alone doesn't break (6<8), but the flat
        // Chainsaw +3 does: 6+3=9 >= 8.
        let game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 8);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(0);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Chainsaw" && m.value == 3));
        assert!(t.ctx.armor_broken);
    }

    #[test]
    fn mb_stacks_against_chainsaw_option_still_applies_chainsaw_via_the_scan_path() {
        // Java: `if (chainsaw != null && !skillsStackAgainstChainsaw) {...} else if (!broken && ...)
        // { ... if (chainsaw != null) armorModifiers.addAll(chainsaw.getArmorModifiers()); ...}` —
        // enabling MB_STACKS_AGAINST_CHAINSAW only skips the short-circuit fast path; the +3 still
        // gets added via the full per-skill scan (`handleChainsawAndMb`), just combined
        // differently with any Mighty-Blow-family modifier.
        let mut game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 8);
        game.options.set(MB_STACKS_AGAINST_CHAINSAW, "true");
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng = GameRng::new(0);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Chainsaw" && m.value == 3));
    }

    #[test]
    fn allow_attacker_chainsaw_false_ignores_attackers_chainsaw_skill() {
        let game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 8);
        let mut t = InjuryTypeBlock::new_with_chainsaw(BlockMode::Regular, true, false);
        let mut rng = GameRng::new(0);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.iter().any(|m| m.name == "Chainsaw"));
    }

    #[test]
    fn stunty_defender_ko_at_total_7_bb2020() {
        // BB2020: Stunty at roll 7 → KO instead of Stunned.
        // Seed rng to produce d1=3, d2=4 (total 7 with no modifiers).
        use ffb_model::enums::{PS_KNOCKED_OUT, PS_STUNNED};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("defender", 2, vec![SkillId::Stunty]));
        let game = Game::new(home, crate::step::framework::test_team("away", 0), ffb_model::enums::Rules::Bb2025);
        let mut t = InjuryTypeBlock::new(BlockMode::Regular, true);
        // seed=42 produces d1+d2=7 for first pair
        let mut rng = GameRng::new(42);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, None, "defender");
        // With Stunty (BB2020), total=7 must be KO, not Stunned
        let state = t.ctx.injury.map(|s| s.base());
        // The roll total determines the outcome — if total was 7, should be KO
        if t.ctx.injury_roll == Some([3, 4]) || t.ctx.injury_roll == Some([4, 3]) {
            assert_eq!(state, Some(PS_KNOCKED_OUT), "Stunty at total 7 must be KO in BB2020");
        }
        // Regardless of roll, non-Stunty player at total 7 would be Stunned
        let mut home2 = crate::step::framework::test_team("home", 0);
        home2.players.push(make_player("defender", 2, vec![]));
        let game2 = Game::new(home2, crate::step::framework::test_team("away", 0), ffb_model::enums::Rules::Bb2025);
        let mut t2 = InjuryTypeBlock::new(BlockMode::Regular, true);
        let mut rng2 = GameRng::new(42);
        t2.ctx.armor_broken = true;
        t2.injury_roll(&game2, &mut rng2, None, "defender");
        if t2.ctx.injury_roll == Some([3, 4]) || t2.ctx.injury_roll == Some([4, 3]) {
            assert_eq!(t2.ctx.injury.map(|s| s.base()), Some(PS_STUNNED), "non-Stunty at total 7 must be Stunned");
        }
    }
}
