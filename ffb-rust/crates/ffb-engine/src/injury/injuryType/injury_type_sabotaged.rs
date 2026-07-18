/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeSabotaged.
/// Armor roll with no block modifier checks. Injury roll or PRONE.
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};

pub struct InjuryTypeSabotaged { ctx: InjuryContext }
impl InjuryTypeSabotaged { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeSabotaged { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeSabotaged {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        do_armor_roll(game, rng, &mut self.ctx, defender_id);
        // Java: `setInjury(defender, gameState, diceRoller, injuryContext)` routes through
        // `RollMechanic.interpretInjuryRoll`, which is edition-aware (Stunty/ThickSkull handling)
        // — the same path every other "standard" injury type uses. This previously called the
        // non-edition-aware `do_injury_roll`, silently dropping Stunty/ThickSkull interpretation
        // for Sabotaged injuries.
        if self.ctx.armor_broken { do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id); }
        else { self.ctx.injury = Some(PlayerState::new(PS_PRONE)); }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    /// Java: `Sabotaged()` constructor passes `SendToBoxReason.SABOTAGED`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Sabotaged) }
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
        let mut t = InjuryTypeSabotaged::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeSabotaged::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn causes_turnover_by_default() { assert!(InjuryTypeSabotaged::new().falling_down_causes_turnover()); }
    #[test]
    fn context_stores_attacker_and_defender() {
        let mut t = InjuryTypeSabotaged::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, Some("saboteur"), "target", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("saboteur"));
        assert_eq!(t.ctx.defender_id.as_deref(), Some("target"));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeSabotaged::new();
        let t2 = InjuryTypeSabotaged::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }
    #[test]
    fn send_to_box_reason_is_sabotaged() {
        assert_eq!(InjuryTypeSabotaged::new().send_to_box_reason(), Some(SendToBoxReason::Sabotaged));
    }

    #[test]
    fn stunty_defender_ko_at_total_7_bb2020() {
        // Regression test: Java's Sabotaged.setInjury routes through the edition-aware
        // RollMechanic.interpretInjuryRoll (Stunty/ThickSkull handling), which this file
        // previously bypassed by calling the non-edition-aware `do_injury_roll` helper (Stunty
        // would then incorrectly be Stunned instead of KO'd at an injury total of 7).
        use ffb_model::enums::{SkillId, PS_STUNNED, PS_KNOCKED_OUT};
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        fn game_with_defender(armour: i32, skills: Vec<SkillId>) -> Game {
            let mut home = crate::step::framework::test_team("home", 0);
            home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
                position_id: "lineman".into(), player_type: PlayerType::Regular,
                gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
                passing: 4, armour, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
                temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                is_big_guy: false,
        ..Default::default() });
            Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2020)
        }
        let mut found_total_7 = false;
        for seed in 1..=100u64 {
            let stunty_game = game_with_defender(2, vec![SkillId::Stunty]);
            let mut t = InjuryTypeSabotaged::new();
            let mut rng = GameRng::new(seed);
            t.handle_injury(&stunty_game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
            if let Some([d1, d2]) = t.ctx.injury_roll {
                if d1 + d2 == 7 {
                    found_total_7 = true;
                    assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_KNOCKED_OUT),
                        "seed={seed}: Stunty at injury total 7 must be KO in BB2020, got {:?}", t.ctx.injury);
                }
            }
        }
        assert!(found_total_7, "test setup failure: no seed in range produced an injury total of 7");
        let _ = PS_STUNNED;
    }
}
