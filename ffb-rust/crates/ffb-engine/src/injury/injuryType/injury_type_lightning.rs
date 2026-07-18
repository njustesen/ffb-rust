/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeLightning.
/// Rolls armor with no modifier first; only if it doesn't break does the Lightning special-effect
/// armor modifier get looked up (via `ArmorModifierFactory.specialEffectArmourModifiers`, which
/// returns nothing for a defender with an `ignoresArmourModifiersFromSkills` skill like Iron Hard
/// Skin) and applied. If broken: injury roll; the Lightning special-effect *injury* modifier is
/// then added only if no special-effect armor modifier ended up applied (the bonus is either/or,
/// never both). If not broken: PRONE. falling_down_causes_turnover=false.
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::SpecialEffect;
use ffb_mechanics::modifiers::{ARMOR_LIGHTNING, INJURY_LIGHTNING};
use ffb_mechanics::modifiers::armor_modifier_factory::ArmorModifierFactory;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, recalc_armor_broken, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeLightning { ctx: InjuryContext }
impl InjuryTypeLightning { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeLightning { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeLightning {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        if !self.ctx.armor_broken {
            // Java: roll first with no modifier; only look up the Lightning special-effect armor
            // modifier (and re-check) if the raw roll didn't already break armor.
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
            if !self.ctx.armor_broken {
                if let Some(defender) = game.player(defender_id) {
                    let factory = ArmorModifierFactory::new(game.rules);
                    if !factory.special_effect_armour_modifiers(SpecialEffect::LIGHTNING, defender).is_empty() {
                        self.ctx.add_armor_modifier(ARMOR_LIGHTNING);
                    }
                }
                recalc_armor_broken(game, &mut self.ctx, defender_id);
            }
        }
        if self.ctx.armor_broken {
            // Java: `((InjuryModifierFactory) game.getFactory(...)).findInjuryModifiers(game,
            // injuryContext, pAttacker, pDefender, isStab(), isFoul(), isVomitLike())` —
            // Lightning isStab/isFoul/isVomitLike all default false (InjuryType base).
            if let Some(defender) = game.player(defender_id) {
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
            }
            // Java: `if (Arrays.stream(injuryContext.getArmorModifiers())
            // .noneMatch(modifier -> modifier instanceof SpecialEffectArmourModifier))` — the
            // Lightning bonus applies either to armor or to injury, never both. "Lightning" is the
            // only `SpecialEffectArmourModifier` this file ever adds to `armor_modifiers`.
            if !self.ctx.armor_modifiers.iter().any(|m| m.name == "Lightning") {
                self.ctx.add_injury_modifier(INJURY_LIGHTNING);
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    /// Java: `Lightning()` constructor passes `SendToBoxReason.LIGHTNING`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Lightning) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    fn game_with_armor(armour: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
    ..Default::default() });
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }
    fn make_skilled_player(id: &str, armour: i32, skills: Vec<SkillId>) -> ffb_model::model::player::Player {
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
    fn game_with_attacker_and_defender(attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_skilled_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_skilled_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeLightning::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeLightning::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn no_turnover() { assert!(!InjuryTypeLightning::new().falling_down_causes_turnover()); }
    #[test]
    fn send_to_box_reason_is_lightning() {
        assert_eq!(InjuryTypeLightning::new().send_to_box_reason(), Some(SendToBoxReason::Lightning));
    }
    #[test]
    fn new_creates_instance_with_correct_apo_mode() {
        let t = InjuryTypeLightning::new();
        assert_eq!(t.ctx.apothecary_mode, ApothecaryMode::Defender);
    }
    #[test]
    fn injury_context_returns_context() {
        let t = InjuryTypeLightning::new();
        assert_eq!(t.injury_context().apothecary_mode, ApothecaryMode::Defender);
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        // Proves InjuryModifierFactory is now reached from handle_injury (Phase ABJ bug fix):
        // Mighty Blow applies since isStab/isFoul/isVomitLike are all false for Lightning.
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeLightning::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"),
            "expected Mighty Blow injury modifier, got {:?}", t.ctx.injury_modifiers);
    }

    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeLightning::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"));
    }

    fn game_with_defender_skills(armour: i32, defender_skills: Vec<SkillId>) -> Game {
        let home = crate::step::framework::test_team("home", 0);
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_skilled_player("p1", armour, defender_skills));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn lightning_bonus_needed_for_armor_break_excludes_injury_bonus() {
        // seed=0 -> raw roll (3,3)=6, which does not break armour 7 on its own; the Lightning
        // armor modifier (+1) is then needed and applied (6+1=7 >= 7, broken). Java: when the
        // Lightning bonus was consumed by the armor roll, the injury roll must NOT also get it.
        let mut t = InjuryTypeLightning::new();
        let mut rng = GameRng::new(0);
        t.handle_injury(&game_with_armor(7), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Lightning"),
            "Lightning armor modifier should have been needed and applied");
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Lightning"),
            "Lightning injury bonus must not apply when the armor bonus was already used, got {:?}",
            t.ctx.injury_modifiers);
    }

    #[test]
    fn armor_breaks_naturally_lightning_bonus_applies_to_injury_instead() {
        // seed=0 -> raw roll (3,3)=6, which breaks armour 6 on its own (6 >= 6); the Lightning
        // armor modifier is never looked up/added, so the injury roll gets the +1 instead.
        let mut t = InjuryTypeLightning::new();
        let mut rng = GameRng::new(0);
        t.handle_injury(&game_with_armor(6), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.armor_modifiers.iter().any(|m| m.name == "Lightning"),
            "Lightning armor modifier must not be added when the raw roll already broke armor");
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "Lightning"),
            "Lightning injury bonus must apply since the armor bonus was never used, got {:?}",
            t.ctx.injury_modifiers);
    }

    #[test]
    fn defender_ignoring_skill_armor_modifiers_never_gets_lightning_armor_bonus() {
        // Java: `ArmorModifierFactory.specialEffectArmourModifiers` returns nothing for a defender
        // with an `ignoresArmourModifiersFromSkills` skill (e.g. Iron Hard Skin), so the Lightning
        // bonus is never applied to the armor roll for such a defender, even when it was needed.
        let mut t = InjuryTypeLightning::new();
        let mut rng = GameRng::new(0); // raw roll (3,3)=6, would need +1 to break armour 7
        t.handle_injury(&game_with_defender_skills(7, vec![SkillId::IronHardSkin]), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_modifiers.iter().any(|m| m.name == "Lightning"),
            "Iron Hard Skin defender must never receive the Lightning armor modifier");
        assert!(!t.ctx.armor_broken, "armor must stay unbroken since the needed bonus was denied");
    }
}
