/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeFireball.
/// Armor roll; if the unmodified roll saves, Fireball's +1 is applied to a re-check. If armor
/// breaks (with or without the bonus), injury roll follows, getting the +1 only if the armor
/// side never used it. If armor is not broken: PRONE.
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::{ARMOR_FIREBALL, INJURY_FIREBALL};
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player, recalc_armor_broken};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeFireball { ctx: InjuryContext }
impl InjuryTypeFireball { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeFireball { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeFireball {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        // Java: Fireball's special-effect bonus is single-use — it is applied to the armor
        // roll only if the *unmodified* roll didn't already break armor; otherwise it carries
        // over to boost the injury roll instead. It is never applied to both.
        if !self.ctx.armor_broken {
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
            if !self.ctx.armor_broken {
                self.ctx.add_armor_modifier(ARMOR_FIREBALL);
                recalc_armor_broken(game, &mut self.ctx, defender_id);
            }
        }
        if self.ctx.armor_broken {
            // Java: InjuryModifierFactory factory = game.getFactory(FactoryType.Factory.INJURY_MODIFIER);
            // factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(), isFoul(),
            // isVomitLike()) — Fireball never overrides isStab/isFoul/isVomitLike (all false).
            if let Some(defender) = game.player(defender_id) {
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
            }
            // Java: only add the injury-side Fireball bonus if the armor-side bonus was never
            // used (i.e. `injuryContext.getArmorModifiers()` contains no SpecialEffectArmourModifier).
            if !self.ctx.armor_modifiers.iter().any(|m| *m == ARMOR_FIREBALL) {
                self.ctx.add_injury_modifier(INJURY_FIREBALL);
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    // Java: `Fireball` does not override `fallingDownCausesTurnover()`, so the `InjuryType`
    // base default (`true`) applies — no override needed here.
    /// Java: `Fireball()` constructor passes `SendToBoxReason.FIREBALL`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Fireball) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
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
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeFireball::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeFireball::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn falling_down_causes_turnover_defaults_true() {
        // Java: `Fireball` does not override `fallingDownCausesTurnover()`, so `InjuryType`'s
        // base default (`true`) applies. Regression test for a previously-inverted override
        // (`false`) that had no basis in the Java source.
        assert!(InjuryTypeFireball::new().falling_down_causes_turnover());
    }
    #[test]
    fn send_to_box_reason_is_fireball() {
        assert_eq!(InjuryTypeFireball::new().send_to_box_reason(), Some(SendToBoxReason::Fireball));
    }

    /// Java: Fireball's special-effect bonus is single-use. If the raw (unmodified) armor roll
    /// already saves the defender, the +1 is added to the armor modifiers and re-checked; if
    /// that breaks armor, the injury roll must NOT also get the +1 bonus.
    #[test]
    fn fireball_bonus_applies_to_armor_only_when_raw_roll_saves() {
        let mut probe = GameRng::new(1);
        let natural = probe.d6() + probe.d6();
        let armour = natural + 1; // raw roll saves (natural < armour); +1 breaks it exactly.
        let mut t = InjuryTypeFireball::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(armour), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_FIREBALL));
        assert!(!t.ctx.injury_modifiers.contains(&INJURY_FIREBALL),
            "injury roll should not also get the Fireball bonus once it was used on armor");
    }

    /// Java: if armor is already broken (e.g. pre-broken context), the armor-modifier block is
    /// skipped entirely, so no SpecialEffectArmourModifier is ever added — the Fireball bonus
    /// then carries over to the injury roll instead.
    #[test]
    fn fireball_bonus_applies_to_injury_when_armor_already_broken() {
        let mut t = InjuryTypeFireball::new();
        t.ctx.armor_broken = true;
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(7), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_FIREBALL));
        assert!(t.ctx.injury_modifiers.contains(&INJURY_FIREBALL));
    }
    #[test]
    fn new_creates_instance_with_correct_apo_mode() {
        let t = InjuryTypeFireball::new();
        assert_eq!(t.ctx.apothecary_mode, ApothecaryMode::Defender);
    }
    #[test]
    fn injury_context_returns_context() {
        let t = InjuryTypeFireball::new();
        assert_eq!(t.injury_context().apothecary_mode, ApothecaryMode::Defender);
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>, defender_armour: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        fn make_player(id: &str, armour: i32, skills: Vec<ffb_model::enums::SkillId>) -> Player {
            Player { id: id.into(), name: id.into(), nr: 1,
                position_id: "lineman".into(), player_type: PlayerType::Regular,
                gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
                passing: 4, armour, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
                temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                is_big_guy: false,
                ..Default::default() }
        }
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        use ffb_model::enums::SkillId;
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeFireball::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }

    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeFireball::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
}
