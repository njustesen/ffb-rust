/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeKegHit.
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_mechanics::modifiers::ARMOR_CHAINSAW_3;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeKegHit { ctx: InjuryContext }
impl InjuryTypeKegHit { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeKegHit { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeKegHit {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        if !self.ctx.armor_broken {
            let defender_ignores = game.player(defender_id)
                .map(|p| p.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
                .unwrap_or(false);
            if !defender_ignores {
                if game.player(defender_id)
                    .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
                    .unwrap_or(false)
                {
                    self.ctx.add_armor_modifier(ARMOR_CHAINSAW_3);
                }
            }
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        if self.ctx.armor_broken {
            // Java: `factory.findInjuryModifiers(game, injuryContext, null, pDefender, isStab(),
            // isFoul(), isVomitLike())` — attacker is passed as a literal `null` regardless of
            // pAttacker. KegHit isStab/isFoul/isVomitLike all default false (InjuryType base).
            if let Some(defender) = game.player(defender_id) {
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, None, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), None, defender, game.rules));
                }
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        }
        else { self.ctx.injury = Some(PlayerState::new(PS_PRONE)); }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    /// Java: `KegHit()` constructor passes `SendToBoxReason.THROWN_KEG`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::ThrownKeg) }
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
    fn game_with_armor_and_niggling(armour: i32, niggling: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: niggling, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
    ..Default::default() });
        // Bb2016 rules: InjuryModifierFactory has a niggling-injury modifier (Bb2025 has none),
        // used here to prove the factory is now reached from injury_roll.
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2016)
    }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeKegHit::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeKegHit::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }

    #[test]
    fn stores_defender_id() {
        let mut t = InjuryTypeKegHit::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.injury_context().defender_id.as_deref(), Some("p1"));
    }

    #[test]
    fn stores_attacker_id() {
        let mut t = InjuryTypeKegHit::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, Some("atk"), "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.injury_context().attacker_id.as_deref(), Some("atk"));
    }

    #[test]
    fn stores_coordinate() {
        let mut t = InjuryTypeKegHit::new();
        let mut rng = GameRng::new(1);
        let c = FieldCoordinate::new(3, 7);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", c, None, None, ApothecaryMode::Defender);
        assert_eq!(t.injury_context().defender_coordinate, Some(c));
    }

    #[test]
    fn niggling_injury_modifier_applied_when_armor_breaks() {
        // Proves InjuryModifierFactory is now reached from injury_roll (Phase ABJ bug fix):
        // defender has 1 niggling injury, low armour guarantees a break.
        let mut t = InjuryTypeKegHit::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor_and_niggling(2, 1), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")),
            "expected a niggling injury modifier to be present, got {:?}", t.ctx.injury_modifiers);
    }
    #[test]
    fn no_niggling_injury_no_modifier() {
        let mut t = InjuryTypeKegHit::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor_and_niggling(2, 0), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }

    #[test]
    fn send_to_box_reason_is_thrown_keg() {
        assert_eq!(InjuryTypeKegHit::new().send_to_box_reason(), Some(SendToBoxReason::ThrownKeg));
    }

    #[test]
    fn injury_context_mut_allows_modification() {
        let mut t = InjuryTypeKegHit::new();
        t.injury_context_mut().armor_broken = true;
        assert!(t.injury_context().armor_broken);
    }
}
