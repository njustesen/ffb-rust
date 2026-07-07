/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBomb.
/// Armor roll with Bomb +1 armor modifier. If broken: injury roll with Bomb +1.
/// No PRONE fallback: if armor holds, injury stays None. falling_down_causes_turnover=false.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_mechanics::modifiers::{ARMOR_BOMB, ARMOR_CHAINSAW_3, INJURY_BOMB};
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};

pub struct InjuryTypeBomb { ctx: InjuryContext }
impl InjuryTypeBomb { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeBomb { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeBomb {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        if !self.ctx.armor_broken {
            self.ctx.add_armor_modifier(ARMOR_BOMB);
            // Java: check defender for ignoresArmourModifiersFromSkills / blocksLikeChainsaw
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
        // Java: no PRONE fallback — bomb sets no injury if armor holds
        if self.ctx.armor_broken {
            self.ctx.add_injury_modifier(INJURY_BOMB);
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
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
    fn armor_not_broken_leaves_injury_unset() {
        let mut t = InjuryTypeBomb::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken); assert!(t.ctx.injury.is_none());
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeBomb::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
    }
    #[test]
    fn no_turnover() { assert!(!InjuryTypeBomb::new().falling_down_causes_turnover()); }
    #[test]
    fn defender_with_chainsaw_gets_chainsaw_modifier() {
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::SkillId;
        let mut game = game_with_armor(13);
        game.team_home.players[0].extra_skills.push(SkillWithValue::new(SkillId::Chainsaw));
        let mut t = InjuryTypeBomb::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = false;
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
    #[test]
    fn defender_with_iron_hard_skin_blocks_chainsaw_modifier() {
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::SkillId;
        let mut game = game_with_armor(13);
        game.team_home.players[0].extra_skills.push(SkillWithValue::new(SkillId::Chainsaw));
        game.team_home.players[0].extra_skills.push(SkillWithValue::new(SkillId::IronHardSkin));
        let mut t = InjuryTypeBomb::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = false;
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
}
