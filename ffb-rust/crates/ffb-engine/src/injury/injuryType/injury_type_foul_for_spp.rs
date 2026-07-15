/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeFoulForSpp.
/// ModificationAware: foul armor roll (foul-assist + blatant-foul modifiers) + injury roll.
/// savedByArmour -> PRONE (default). isFoul=true, isStab=false.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE, SkillId};
use ffb_model::model::property::NamedProperties;
use ffb_model::option::game_option_id;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::{foul_assist_armor_modifier, niggling_injury_modifier, ARMOR_CHAINSAW_3, ARMOR_DIRTY_PLAYER_1, ARMOR_FOUL, INJURY_DIRTY_PLAYER_1};
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury};

pub struct InjuryTypeFoulForSpp { ctx: InjuryContext, use_chainsaw: bool }
impl InjuryTypeFoulForSpp {
    pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender), use_chainsaw: false } }
    pub fn new_with_chainsaw(use_chainsaw: bool) -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender), use_chainsaw } }
}
impl Default for InjuryTypeFoulForSpp { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeFoulForSpp {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    /// Java: `FoulForSpp.isWorthSpps()` — true.
    fn is_worth_spps(&self) -> bool { true }
    /// Java: `FoulForSpp.isCausedByOpponent()` — true (overridden, unlike base `Foul`).
    fn is_caused_by_opponent(&self) -> bool { true }
}
impl ModificationAwareInjuryType for InjuryTypeFoulForSpp {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str, _roll: bool) {
        if let Some(aid) = attacker_id {
            let off = UtilPlayer::find_offensive_foul_assists(game, aid, defender_id) as i32;
            let def = UtilPlayer::find_defensive_foul_assists(game, aid, defender_id) as i32;
            if let Some(m) = foul_assist_armor_modifier(off - def) {
                self.ctx.add_armor_modifier(m);
            }
            if game.options.is_enabled(game_option_id::FOUL_BONUS)
                || (game.options.is_enabled(game_option_id::FOUL_BONUS_OUTSIDE_TACKLEZONE)
                    && UtilPlayer::find_tacklezones(game, aid) < 1)
            {
                self.ctx.add_armor_modifier(ARMOR_FOUL);
            }
            // DirtyPlayer: +1 to armor roll for fouls
            if let Some(attacker) = game.player(aid) {
                if attacker.has_skill(SkillId::DirtyPlayer) {
                    self.ctx.add_armor_modifier(ARMOR_DIRTY_PLAYER_1);
                }
            }
        }
        // Java: if (game.isActive(foulBreaksArmourWithoutRoll)) { setArmorBroken(true); }
        //       if (!isArmorBroken()) { rollArmour(); ... setArmorBroken(interpreter.isArmourBroken(...)); }
        if game.is_active(NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL) {
            self.ctx.armor_broken = true;
        }
        if !self.ctx.armor_broken {
            // Java: if (useChainsaw) — chainsaw foul adds +3 unless defender has IronHardSkin
            if self.use_chainsaw {
                let defender_ignores = game.player(defender_id)
                    .map(|p| p.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
                    .unwrap_or(false);
                if !defender_ignores {
                    if let Some(aid) = attacker_id {
                        if game.player(aid)
                            .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
                            .unwrap_or(false)
                        {
                            self.ctx.add_armor_modifier(ARMOR_CHAINSAW_3);
                        }
                    }
                }
            }
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        if let Some(defender) = game.player(defender_id) {
            if let Some(m) = niggling_injury_modifier(defender.niggling_injuries) {
                self.ctx.add_injury_modifier(m);
            }
        }
        // DirtyPlayer: +1 to injury roll for fouls
        if let Some(aid) = attacker_id {
            if let Some(attacker) = game.player(aid) {
                if attacker.has_skill(SkillId::DirtyPlayer) {
                    self.ctx.add_injury_modifier(INJURY_DIRTY_PLAYER_1);
                }
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    // savedByArmour: default PRONE
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_mechanics::modifiers::{ARMOR_DIRTY_PLAYER_1, INJURY_DIRTY_PLAYER_1};

    fn make_player(id: &str, armour: i32, skills: Vec<SkillId>) -> ffb_model::model::player::Player {
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

    fn game_with_armor(armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("p1", armour, vec![]));
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }

    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeFoulForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeFoulForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn dirty_player_adds_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_DIRTY_PLAYER_1));
    }
    #[test]
    fn no_dirty_player_no_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_DIRTY_PLAYER_1));
    }
    #[test]
    fn dirty_player_adds_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(t.ctx.injury_modifiers.contains(&INJURY_DIRTY_PLAYER_1));
    }
    #[test]
    fn no_dirty_player_no_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&INJURY_DIRTY_PLAYER_1));
    }
    #[test]
    fn chainsaw_foul_adds_chainsaw_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 2);
        let mut t = InjuryTypeFoulForSpp::new_with_chainsaw(true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
    #[test]
    fn non_chainsaw_foul_no_chainsaw_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
}
