/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBallAndChain.
/// Armor always broken (ball and chain special rule). Injury roll follows.
use ffb_model::enums::{ApothecaryMode, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeBallAndChain { ctx: InjuryContext }
impl InjuryTypeBallAndChain { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeBallAndChain { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeBallAndChain {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(),
        // isFoul(), isVomitLike())` — BallAndChain never overrides isStab/isFoul/isVomitLike, all false.
        if let Some(defender) = game.player(defender_id) {
            let attacker = attacker_id.and_then(|aid| game.player(aid));
            let factory = InjuryModifierFactory::new(game.rules);
            for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    // Ball-and-Chain: falling down causes injury (never just prone), but the type itself
    // already forces armor_broken=true, so failedArmourPlacesProne() = false in Java.
    fn failed_armour_places_prone(&self) -> bool { false }
    fn send_to_box_reason(&self) -> Option<ffb_model::enums::SendToBoxReason> {
        Some(ffb_model::enums::SendToBoxReason::BallAndChain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let make_player = |id: &str, skills: Vec<ffb_model::enums::SkillId>| Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        use ffb_model::enums::SkillId;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow]);
        let mut t = InjuryTypeBallAndChain::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"));
    }
    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![]);
        let mut t = InjuryTypeBallAndChain::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"));
    }
    #[test]
    fn armor_always_broken() {
        let mut t = InjuryTypeBallAndChain::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn injury_is_set() {
        let mut t = InjuryTypeBallAndChain::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury.is_some());
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn failed_armour_does_not_place_prone() {
        assert!(!InjuryTypeBallAndChain::new().failed_armour_places_prone());
    }
    #[test]
    fn send_to_box_reason_is_ball_and_chain() {
        use ffb_model::enums::SendToBoxReason;
        assert_eq!(InjuryTypeBallAndChain::new().send_to_box_reason(), Some(SendToBoxReason::BallAndChain));
    }
    #[test]
    fn attacker_id_stored_in_context() {
        let mut t = InjuryTypeBallAndChain::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, Some("attacker"), "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("attacker"));
    }
}
