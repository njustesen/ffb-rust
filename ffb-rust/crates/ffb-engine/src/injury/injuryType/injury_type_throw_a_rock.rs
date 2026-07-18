/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeThrowARock.
/// Armor is always broken (no roll). Injury roll follows. No turnover.
use ffb_model::enums::{ApothecaryMode, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeThrowARock { ctx: InjuryContext }
impl InjuryTypeThrowARock { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeThrowARock { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeThrowARock {
    fn handle_injury(&mut self, _game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(),
        // isFoul(), isVomitLike())` — ThrowARock does not override isStab/isFoul/isVomitLike
        // (all default false in InjuryType).
        if let Some(defender) = _game.player(defender_id) {
            let attacker = attacker_id.and_then(|aid| _game.player(aid));
            let factory = InjuryModifierFactory::new(_game.rules);
            for m in factory.find_injury_modifiers(_game, attacker, defender, false, false, false) {
                self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, _game.rules));
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, _game, defender_id);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    // Java: `ThrowARock` does not override `fallingDownCausesTurnover()`, so the `InjuryType`
    // base default (`true`) applies. Regression fix: was previously inverted to `false` here
    // with no basis in the Java source.
    /// Java: `ThrowARock` constructed with `super("throwARock", false, SendToBoxReason.HIT_BY_ROCK)`.
    /// Was previously missing (defaulted to `None`).
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::HitByRock) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_is_always_broken() {
        let mut t = InjuryTypeThrowARock::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn injury_is_set() {
        let mut t = InjuryTypeThrowARock::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury.is_some());
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn causes_turnover_by_default() {
        // Java: `ThrowARock` does not override `fallingDownCausesTurnover()`, so `InjuryType`'s
        // base default (`true`) applies. Regression test for a previously-inverted override.
        assert!(InjuryTypeThrowARock::new().falling_down_causes_turnover());
    }
    #[test]
    fn send_to_box_reason_is_hit_by_rock() {
        assert_eq!(InjuryTypeThrowARock::new().send_to_box_reason(), Some(SendToBoxReason::HitByRock));
    }
    #[test]
    fn new_creates_instance_with_correct_apo_mode() {
        let t = InjuryTypeThrowARock::new();
        assert_eq!(t.ctx.apothecary_mode, ApothecaryMode::Defender);
    }
    #[test]
    fn sets_attacker_and_defender_ids() {
        let mut t = InjuryTypeThrowARock::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, Some("atk1"), "def1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("def1"));
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("atk1"));
    }

    fn make_player(id: &str, skills: Vec<ffb_model::enums::SkillId>) -> ffb_model::model::player::Player {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player { id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 7, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() }
    }
    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", vec![]));
        Game::new(home, away, Rules::Bb2025)
    }
    #[test]
    fn mighty_blow_adds_injury_modifier() {
        // ThrowARock does not override isStab/isFoul/isVomitLike (all default false), so
        // MightyBlow (which requires all three false) applies here, unlike DirtyPlayer.
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![ffb_model::enums::SkillId::MightyBlow]);
        let mut t = InjuryTypeThrowARock::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![]);
        let mut t = InjuryTypeThrowARock::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
}
