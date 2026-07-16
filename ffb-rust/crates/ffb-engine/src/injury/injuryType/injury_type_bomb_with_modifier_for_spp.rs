/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBombWithModifierForSpp.
/// Armor roll + special effect BOMB modifier stubs + injury or PRONE.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::{ARMOR_BOMB, INJURY_BOMB};
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeBombWithModifierForSpp { ctx: InjuryContext }
impl InjuryTypeBombWithModifierForSpp { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeBombWithModifierForSpp { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeBombWithModifierForSpp {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.add_armor_modifier(ARMOR_BOMB);
        do_armor_roll(game, rng, &mut self.ctx, defender_id);
        if self.ctx.armor_broken {
            self.ctx.add_injury_modifier(INJURY_BOMB);
            // Java: `factory.findInjuryModifiers(game, injuryContext, null, pDefender, isStab(),
            // isFoul(), isVomitLike())` — attacker is hardcoded to null here (unlike the other Bomb
            // variants), so attacker-sourced injury modifiers (e.g. Mighty Blow) never apply.
            if let Some(defender) = game.player(defender_id) {
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, None, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), None, defender, game.rules));
                }
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
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
        let mut t = InjuryTypeBombWithModifierForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeBombWithModifierForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn does_not_cause_turnover() { assert!(!InjuryTypeBombWithModifierForSpp::new().falling_down_causes_turnover()); }
    #[test]
    fn bomb_armor_modifier_is_added() {
        let mut t = InjuryTypeBombWithModifierForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_modifiers.is_empty());
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeBombWithModifierForSpp::new();
        let t2 = InjuryTypeBombWithModifierForSpp::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    fn game_with_attacker_and_defender_rules(rules: Rules, attacker_skills: Vec<ffb_model::enums::SkillId>, defender_armour: i32, defender_nigglings: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player {
            id: "attacker".into(), name: "attacker".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 7, starting_skills: attacker_skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(Player {
            id: "defender".into(), name: "defender".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: defender_armour, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: defender_nigglings, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        Game::new(home, away, rules)
    }

    #[test]
    fn attacker_mighty_blow_does_not_apply_because_attacker_is_ignored() {
        // Java hardcodes `null` for the attacker in this findInjuryModifiers call, so even an
        // attacker with Mighty Blow must not contribute an injury modifier here.
        use ffb_model::enums::SkillId;
        let game = game_with_attacker_and_defender_rules(Rules::Bb2025, vec![SkillId::MightyBlow], 2, 0);
        let mut t = InjuryTypeBombWithModifierForSpp::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"));
    }
    #[test]
    fn niggling_injury_modifier_still_applies() {
        // Bb2016 has niggling injury modifiers; Bb2025's factory has none (see
        // Bb2025InjuryModifiers), so this uses Bb2016 rules to prove the factory is wired in.
        let game = game_with_attacker_and_defender_rules(Rules::Bb2016, vec![], 2, 1);
        let mut t = InjuryTypeBombWithModifierForSpp::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }
}
