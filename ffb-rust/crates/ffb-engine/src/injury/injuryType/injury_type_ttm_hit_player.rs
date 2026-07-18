/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeTTMHitPlayer.
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_mechanics::modifiers::ARMOR_CHAINSAW_3;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeTTMHitPlayer { ctx: InjuryContext }
impl InjuryTypeTTMHitPlayer { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::HitPlayer) } } }
impl Default for InjuryTypeTTMHitPlayer { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeTTMHitPlayer {
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
            // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender,
            // isStab(), isFoul(), isVomitLike())` — TTMHitPlayer does not override isStab/isFoul/
            // isVomitLike (all default false in InjuryType).
            if let Some(defender) = game.player(defender_id) {
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        }
        else { self.ctx.injury = Some(PlayerState::new(PS_PRONE)); }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    // Java: `TTMHitPlayer` does not override `fallingDownCausesTurnover()`, so the `InjuryType`
    // base default (`true`) applies. Regression fix: was previously inverted to `false` here
    // with no basis in the Java source.
    /// Java: `TTMHitPlayer` constructed with
    /// `super("ttmHitPlayer", false, SendToBoxReason.HIT_BY_THROWN_PLAYER)`. Was previously
    /// missing (defaulted to `None`).
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::HitByThrownPlayer) }
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
        let mut t = InjuryTypeTTMHitPlayer::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::HitPlayer);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeTTMHitPlayer::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::HitPlayer);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
    }
    #[test]
    fn causes_turnover_by_default() {
        assert!(InjuryTypeTTMHitPlayer::new().falling_down_causes_turnover());
    }
    #[test]
    fn send_to_box_reason_is_hit_by_thrown_player() {
        assert_eq!(InjuryTypeTTMHitPlayer::new().send_to_box_reason(), Some(SendToBoxReason::HitByThrownPlayer));
    }
    #[test]
    fn new_creates_instance_with_hit_player_apo_mode() {
        let t = InjuryTypeTTMHitPlayer::new();
        assert_eq!(t.ctx.apothecary_mode, ApothecaryMode::HitPlayer);
    }
    #[test]
    fn sets_attacker_and_defender_ids() {
        let mut t = InjuryTypeTTMHitPlayer::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, Some("atk1"), "p1", coord(), None, None, ApothecaryMode::HitPlayer);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("p1"));
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("atk1"));
    }

    fn make_player(id: &str, armour: i32, skills: Vec<ffb_model::enums::SkillId>) -> ffb_model::model::player::Player {
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
    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>, defender_armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }
    #[test]
    fn mighty_blow_adds_injury_modifier() {
        // TTMHitPlayer does not override isStab/isFoul/isVomitLike (all default false), so
        // MightyBlow (which requires all three false) applies here, unlike DirtyPlayer.
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![ffb_model::enums::SkillId::MightyBlow], 2);
        let mut t = InjuryTypeTTMHitPlayer::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::HitPlayer);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeTTMHitPlayer::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::HitPlayer);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
}
