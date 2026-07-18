/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeTTMHitPlayerForSpp.
///
/// Identical to InjuryTypeTTMHitPlayer in dice roll logic, plus the LethalFlight
/// (`affectsEitherArmourOrInjuryOnTtm`) either/or modifier: if the attacker has a skill
/// with that property and the armor roll doesn't break armor, the modifier is applied
/// to a recheck of the armor roll; if that still doesn't break armor OR armor was already
/// broken coming in (so the armor-recheck branch never ran), the modifier is instead applied
/// to the injury roll. It applies at most once ("consumed on armour").
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_mechanics::modifiers::{ARMOR_CHAINSAW_3, Modifier};
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player, recalc_armor_broken};

pub struct InjuryTypeTTMHitPlayerForSpp { ctx: InjuryContext }
impl InjuryTypeTTMHitPlayerForSpp { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::HitPlayer) } } }
impl Default for InjuryTypeTTMHitPlayerForSpp { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeTTMHitPlayerForSpp {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;

        // Java: Optional<Skill> lethalFlight = UtilCards.getSkillWithProperty(pAttacker, affectsEitherArmourOrInjuryOnTtm);
        let mut lethal_flight_available = attacker_id
            .and_then(|aid| game.player(aid))
            .map(|a| a.all_skill_ids().any(|id| id.properties().contains(&NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_TTM)))
            .unwrap_or(false);

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

            // Java: if (!isArmorBroken() && lethalFlight.isPresent() && !hasUnusedSkillWithProperty(defender, ignoresArmourModifiersFromSkills))
            if !self.ctx.armor_broken && lethal_flight_available {
                let defender_ignores_lf = game.player(defender_id)
                    .map(|p| p.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
                    .unwrap_or(false);
                if !defender_ignores_lf {
                    let attacker_team = attacker_id.and_then(|aid| game.player_team_id(aid));
                    let defender_team = game.player_team_id(defender_id);
                    if attacker_team.is_some() && attacker_team != defender_team {
                        self.ctx.add_armor_modifier(Modifier::new("Lethal Flight", 1, game.rules));
                    }
                    recalc_armor_broken(game, &mut self.ctx, defender_id);
                    lethal_flight_available = false; // consumed on armour
                }
            }
        }
        if self.ctx.armor_broken {
            if lethal_flight_available {
                let attacker_team = attacker_id.and_then(|aid| game.player_team_id(aid));
                let defender_team = game.player_team_id(defender_id);
                if attacker_team.is_some() && attacker_team != defender_team {
                    self.ctx.add_injury_modifier(Modifier::new("Lethal Flight", 1, game.rules));
                }
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        }
        else { self.ctx.injury = Some(PlayerState::new(PS_PRONE)); }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    // Java: `TTMHitPlayerForSpp` does not override `fallingDownCausesTurnover()`, so the
    // `InjuryType` base default (`true`) applies. Regression fix: was previously inverted to
    // `false` here with no basis in the Java source.
    /// Java: `TTMHitPlayerForSpp.isCausedByOpponent()` → true. Was previously missing
    /// (defaulted to `false`).
    fn is_caused_by_opponent(&self) -> bool { true }
    /// Java: `TTMHitPlayerForSpp` constructed with `super("ttmHitPlayerForSpp", true, ...)`.
    /// Was previously missing (defaulted to `false`).
    fn is_worth_spps(&self) -> bool { true }
    /// Java: `TTMHitPlayerForSpp` constructed with
    /// `super("ttmHitPlayerForSpp", true, SendToBoxReason.HIT_BY_THROWN_PLAYER)`. Was previously
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
        let mut t = InjuryTypeTTMHitPlayerForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::HitPlayer);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeTTMHitPlayerForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::HitPlayer);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
    }
    #[test]
    fn causes_turnover_by_default() {
        assert!(InjuryTypeTTMHitPlayerForSpp::new().falling_down_causes_turnover());
    }
    #[test]
    fn is_caused_by_opponent_and_worth_spps() {
        let t = InjuryTypeTTMHitPlayerForSpp::new();
        assert!(t.is_caused_by_opponent());
        assert!(t.is_worth_spps());
    }
    #[test]
    fn send_to_box_reason_is_hit_by_thrown_player() {
        assert_eq!(InjuryTypeTTMHitPlayerForSpp::new().send_to_box_reason(), Some(SendToBoxReason::HitByThrownPlayer));
    }
    #[test]
    fn new_creates_instance_with_hit_player_apo_mode() {
        let t = InjuryTypeTTMHitPlayerForSpp::new();
        assert_eq!(t.ctx.apothecary_mode, ApothecaryMode::HitPlayer);
    }
    #[test]
    fn sets_attacker_and_defender_ids() {
        let mut t = InjuryTypeTTMHitPlayerForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, Some("atk1"), "p1", coord(), None, None, ApothecaryMode::HitPlayer);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("p1"));
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("atk1"));
    }

    fn lethal_flight_modifier() -> Modifier { Modifier::new("Lethal Flight", 1, Rules::Bb2025) }

    fn game_with_lethal_flight_attacker(defender_armour: i32, same_team: bool) -> Game {
            use std::collections::HashSet;
            use ffb_model::model::player::Player;
            use ffb_model::model::SkillWithValue;
            use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
            let attacker = Player { id: "attacker".into(), name: "attacker".into(), nr: 1,
                position_id: "lineman".into(), player_type: PlayerType::Regular,
                gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
                passing: 4, armour: 8,
                starting_skills: vec![SkillWithValue { skill_id: SkillId::LethalFlight, value: None }],
                extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                is_big_guy: false, ..Default::default() };
            let defender = Player { id: "p1".into(), name: "p1".into(), nr: 1,
                position_id: "lineman".into(), player_type: PlayerType::Regular,
                gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
                passing: 4, armour: defender_armour, starting_skills: vec![], extra_skills: vec![],
                temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                is_big_guy: false, ..Default::default() };
            let mut home = crate::step::framework::test_team("home", 0);
            home.players.push(attacker);
            let mut other = crate::step::framework::test_team("away", 0);
            if same_team {
                home.players.push(defender);
                Game::new(home, other, Rules::Bb2025)
            } else {
                other.players.push(defender);
                Game::new(home, other, Rules::Bb2025)
            }
        }

        #[test]
        fn lethal_flight_adds_armor_modifier_against_opposing_team() {
            // armour 13: even with the +1 Lethal Flight modifier, 2d6+1 (max 13) never exceeds 13,
            // so armor never breaks — isolates the modifier-adding behavior from the break outcome.
            let mut t = InjuryTypeTTMHitPlayerForSpp::new(); let mut rng = GameRng::new(1);
            let game = game_with_lethal_flight_attacker(13, false);
            t.handle_injury(&game, &mut rng, Some("attacker"), "p1", coord(), None, None, ApothecaryMode::HitPlayer);
            assert!(t.ctx.armor_modifiers.contains(&lethal_flight_modifier()));
            assert!(!t.ctx.armor_broken);
            assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
        }

        #[test]
        fn lethal_flight_does_not_apply_against_own_team() {
            let mut t = InjuryTypeTTMHitPlayerForSpp::new(); let mut rng = GameRng::new(1);
            let game = game_with_lethal_flight_attacker(13, true);
            t.handle_injury(&game, &mut rng, Some("attacker"), "p1", coord(), None, None, ApothecaryMode::HitPlayer);
            assert!(!t.ctx.armor_modifiers.contains(&lethal_flight_modifier()));
        }

        #[test]
        fn lethal_flight_applies_to_injury_when_armor_already_broken() {
            let mut t = InjuryTypeTTMHitPlayerForSpp::new();
            t.ctx.armor_broken = true; // armor-recheck branch is skipped entirely
            let mut rng = GameRng::new(1);
            let game = game_with_lethal_flight_attacker(13, false);
            t.handle_injury(&game, &mut rng, Some("attacker"), "p1", coord(), None, None, ApothecaryMode::HitPlayer);
        assert!(t.ctx.injury_modifiers.contains(&lethal_flight_modifier()));
        assert!(!t.ctx.armor_modifiers.contains(&lethal_flight_modifier()));
    }
}
