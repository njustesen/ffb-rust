/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeFumbledKtmApoKo.
/// Armor always broken. Injury roll (no special modifier filter). stunIsTreatedAsKo=true.
use ffb_model::enums::{ApothecaryMode, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeFumbledKtmApoKo { ctx: InjuryContext }
impl InjuryTypeFumbledKtmApoKo { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeFumbledKtmApoKo { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeFumbledKtmApoKo {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        // Java: factory.findInjuryModifiers(game, injuryContext, null, pDefender, isStab(), isFoul(),
        // isVomitLike()) — attacker is passed as `null` explicitly (not pAttacker), and
        // isStab/isFoul/isVomitLike are never overridden (all false).
        if let Some(defender) = game.player(defender_id) {
            let factory = InjuryModifierFactory::new(game.rules);
            for m in factory.find_injury_modifiers(game, None, defender, false, false, false) {
                self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), None, defender, game.rules));
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn stun_is_treated_as_ko(&self) -> bool { true }
    /// Java: `KTMFumbleApoKoInjury()` constructor passes `SendToBoxReason.KICKED`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Kicked) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_always_broken_and_injury_set() {
        let mut t = InjuryTypeFumbledKtmApoKo::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test] fn stun_is_ko() { assert!(InjuryTypeFumbledKtmApoKo::new().stun_is_treated_as_ko()); }
    #[test]
    fn send_to_box_reason_is_kicked() {
        assert_eq!(InjuryTypeFumbledKtmApoKo::new().send_to_box_reason(), Some(SendToBoxReason::Kicked));
    }
    #[test]
    fn causes_turnover_by_default() { assert!(InjuryTypeFumbledKtmApoKo::new().falling_down_causes_turnover()); }
    #[test]
    fn context_stores_attacker_and_defender() {
        let mut t = InjuryTypeFumbledKtmApoKo::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, Some("att"), "def", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("att"));
        assert_eq!(t.ctx.defender_id.as_deref(), Some("def"));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeFumbledKtmApoKo::new();
        let t2 = InjuryTypeFumbledKtmApoKo::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>, defender_niggling: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        fn make_player(id: &str, niggling_injuries: i32, skills: Vec<ffb_model::enums::SkillId>) -> Player {
            Player { id: id.into(), name: id.into(), nr: 1,
                position_id: "lineman".into(), player_type: PlayerType::Regular,
                gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
                passing: 4, armour: 8, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
                temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                is_big_guy: false,
                ..Default::default() }
        }
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 0, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_niggling, vec![]));
        // Bb2016 has niggling-injury modifiers (Bb2025 does not).
        Game::new(home, away, Rules::Bb2016)
    }

    /// Defender's own niggling injuries produce a modifier regardless of attacker — proves the
    /// factory call is wired in and reachable.
    #[test]
    fn niggling_injury_modifier_applied() {
        let game = game_with_attacker_and_defender(vec![], 1);
        let mut t = InjuryTypeFumbledKtmApoKo::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "1 Niggling Injury"));
    }

    /// Java passes `null` for the attacker explicitly (not pAttacker) — attacker-only skills like
    /// Mighty Blow must never apply even though an attacker_id was supplied to handle_injury.
    #[test]
    fn attacker_skill_never_applies_because_attacker_passed_as_null() {
        use ffb_model::enums::SkillId;
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 0);
        let mut t = InjuryTypeFumbledKtmApoKo::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
}
