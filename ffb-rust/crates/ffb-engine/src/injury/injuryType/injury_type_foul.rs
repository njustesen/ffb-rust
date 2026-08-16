/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeFoul.
/// ModificationAware: foul armor roll (foul-assist + blatant-foul modifiers) + injury roll.
/// savedByArmour -> PRONE (default). isFoul=true, isStab=false.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE, SendToBoxReason, SkillId};
use ffb_model::model::property::NamedProperties;
use ffb_model::option::game_option_id;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::{
    foul_assist_armor_modifier, Modifier, ARMOR_CHAINSAW_3, ARMOR_DIRTY_PLAYER_1,
    ARMOR_DIRTY_PLAYER_2, ARMOR_FOUL,
};
use ffb_mechanics::mechanics::armor_broken_for_rules;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury, leak_injury_modifier};

pub struct InjuryTypeFoul { ctx: InjuryContext, use_chainsaw: bool }
impl InjuryTypeFoul {
    pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender), use_chainsaw: false } }
    pub fn new_with_chainsaw(use_chainsaw: bool) -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender), use_chainsaw } }
}
impl Default for InjuryTypeFoul { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeFoul {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    /// Java: `Foul.shouldPlayFallSound()` — overridden to `false`.
    fn should_play_fall_sound(&self) -> bool { false }
    /// Java: `Foul()` constructor passes `SendToBoxReason.FOULED`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Fouled) }
}
impl ModificationAwareInjuryType for InjuryTypeFoul {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str, _roll: bool) {
        // TODO: Java (lines 75-91) additionally re-checks `armorModifierFactory.findArmorModifiers`
        // (general attacker/defender skill-based armor modifiers, including a "sneaky pair"
        // affectsEitherArmourOrInjuryWithPartner modifier held back for a second pass) when the
        // roll above didn't break armor. Not modeled here yet — same known gap as InjuryTypeStab.
        // Add foul-assist armor modifier based on net offensive - defensive assists
        if let Some(aid) = attacker_id {
            let off = UtilPlayer::find_offensive_foul_assists(game, aid, defender_id) as i32;
            let def = UtilPlayer::find_defensive_foul_assists(game, aid, defender_id) as i32;
            let net = off - def;
            if let Some(m) = foul_assist_armor_modifier(net) {
                self.ctx.add_armor_modifier(m);
            }
            // Add "Foul" blatant-foul modifier if option enabled or attacker has no tackle zones
            if game.options.is_enabled(game_option_id::FOUL_BONUS)
                || (game.options.is_enabled(game_option_id::FOUL_BONUS_OUTSIDE_TACKLEZONE)
                    && UtilPlayer::find_tacklezones(game, aid) < 1)
            {
                self.ctx.add_armor_modifier(ARMOR_FOUL);
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
            // Java InjuryTypeFoul.armourRoll rolls with the foul-assist (+chainsaw) modifiers and
            // checks `isArmourBroken` FIRST; only `if (!injuryContext.isArmorBroken())` does it then
            // add the general skill-based armour modifiers (Dirty Player) and re-check.
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
            // Dirty Player (registered to affectsEitherArmourOrInjuryOnFoul): spend its +1 on the
            // ARMOUR roll only when the base roll did not already break armour. If it IS spent here it
            // is mutually excluded from the injury roll (see injury_roll); if the base roll already
            // broke armour, Dirty Player is left free to boost the injury roll instead. (dwarf seed
            // 60: base 7 < AV8 -> DP breaks armour, injury 8 stays Thick-Skull Stunned. seed 8: base
            // roll already breaks AV -> DP boosts the injury roll.)
            if !self.ctx.armor_broken {
                if let Some(aid) = attacker_id {
                    if game.player(aid).map(|p| p.has_skill(SkillId::DirtyPlayer)).unwrap_or(false) {
                        // Java's DirtyPlayer registers a VariableArmourModifier whose value is the
                        // attacker's own skill value (`bb2020/DirtyPlayer.java:32` -- the `1` in
                        // `super(..., 1)` is only the DEFAULT). Hardcoding +1 here understated the
                        // dwarf Deathroller's "Dirty Player (2)": a foul armour roll of 7 against
                        // AV9 stayed unbroken where Java reports
                        // `JAVA_AVBROKE armour=9 reduced=9 roll=[1,6] modTotal=2 mods=Dirty Player broken=true`.
                        // bb2016 registers a StaticArmourModifier(+1) instead
                        // (`bb2016/DirtyPlayer.java:31`), so the value only tracks the skill in
                        // bb2020/bb2025.
                        let dp = if game.rules == ffb_model::enums::Rules::Bb2016 {
                            1
                        } else {
                            game.player(aid)
                                .map(|p| p.get_skill_value_int(SkillId::DirtyPlayer, 1))
                                .unwrap_or(1)
                        };
                        self.ctx.add_armor_modifier(match dp {
                            1 => ARMOR_DIRTY_PLAYER_1,
                            2 => ARMOR_DIRTY_PLAYER_2,
                            n => Modifier::new("Dirty Player", n, ffb_model::enums::Rules::Bb2020),
                        });
                        if let Some(roll) = self.ctx.armor_roll {
                            let av = game.player(defender_id)
                                .map(|p| p.armour_with_modifiers()).unwrap_or(7);
                            self.ctx.armor_broken = armor_broken_for_rules(av, roll, &self.ctx.armor_modifiers, game.rules);
                        }
                    }
                }
            }
        }
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(),
        // isFoul(), isVomitLike())` — includes niggling internally (unlike Block's
        // `findInjuryModifiersWithoutNiggling`), so no separate niggling call here. Foul is never
        // stab/vomit-like (separate InjuryType classes), isFoul=true.
        if let Some(defender) = game.player(defender_id) {
            let attacker = attacker_id.and_then(|aid| game.player(aid));
            let factory = InjuryModifierFactory::new(game.rules);
            // Java DirtyPlayer injury modifier `appliesToContext`: applies only if the armour
            // modifiers contain nothing registered to affectsEitherArmourOrInjuryOnFoul. Our only
            // such modifier is Dirty Player's armour +1 — if it was spent on the armour roll, exclude
            // the Dirty Player injury +1 (mutual exclusion).
            // Match by NAME, not by exact constant: the armour-side modifier now carries the
            // attacker's own Dirty Player value (ARMOR_DIRTY_PLAYER_2 for the dwarf Deathroller),
            // so an equality test against the +1 constant would miss it and let Dirty Player boost
            // BOTH rolls, which is exactly what the mutual exclusion forbids.
            let dirty_player_on_armour = self
                .ctx
                .armor_modifiers
                .iter()
                .any(|m| m.name.starts_with("Dirty Player"));
            for m in factory.find_injury_modifiers(game, attacker, defender, false, true, false) {
                let leaked = leak_injury_modifier(m.as_ref(), attacker, defender, game.rules);
                if dirty_player_on_armour && leaked.name == "Dirty Player" {
                    continue;
                }
                self.ctx.add_injury_modifier(leaked);
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
    use ffb_mechanics::modifiers::{ARMOR_DIRTY_PLAYER_1, Modifier};

    /// Real `InjuryModifierFactory`-sourced Dirty Player injury modifier is named "Dirty Player"
    /// (not the pre-Phase-ABJ placeholder constant `INJURY_DIRTY_PLAYER_1` = "Dirty Player +1")
    /// — `injury_roll` now sources it from the real factory, matching Java.
    fn dirty_player_injury_modifier(rules: Rules) -> Modifier {
        Modifier::new("Dirty Player", 1, rules)
    }

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
        let mut t = InjuryTypeFoul::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn should_play_fall_sound_is_false() {
        assert!(!InjuryTypeFoul::new().should_play_fall_sound());
    }
    #[test]
    fn send_to_box_reason_is_fouled() {
        use ffb_model::enums::SendToBoxReason;
        assert_eq!(InjuryTypeFoul::new().send_to_box_reason(), Some(SendToBoxReason::Fouled));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeFoul::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn dirty_player_adds_armor_modifier() {
        // AV 13 is never broken by the base 2d6 roll, so Java's `if (!isArmorBroken())` gate lets
        // Dirty Player's armour modifier be applied. (With a low AV the base roll already breaks
        // armour and Dirty Player is instead reserved for the injury roll — see
        // dirty_player_on_armour_excludes_injury_modifier.)
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 13);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_DIRTY_PLAYER_1));
    }
    #[test]
    fn dirty_player_not_added_to_armor_when_base_roll_breaks() {
        // AV 2 is always broken by the base roll; Java does NOT then apply Dirty Player to armour
        // (the `if (!isArmorBroken())` gate) — it is left free for the injury roll.
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 2);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_DIRTY_PLAYER_1));
    }
    #[test]
    fn dirty_player_on_armour_excludes_injury_modifier() {
        // Mutual exclusion: when Dirty Player's +1 was spent on the armour roll it must NOT also
        // boost the injury roll (Java DirtyPlayer injury modifier `noneMatch(affectsEither...Foul)`).
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 2);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.ctx.add_armor_modifier(ARMOR_DIRTY_PLAYER_1);
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&dirty_player_injury_modifier(game.rules)));
    }
    /// Java registers Dirty Player as a `VariableArmourModifier` (`bb2020/DirtyPlayer.java:32`),
    /// so the armour bonus is the attacker's OWN skill value — the dwarf Deathroller's
    /// "Dirty Player (2)" is +2, not +1. A hardcoded +1 left a foul armour roll of 7 against AV9
    /// unbroken where Java reports `modTotal=2 broken=true`.
    #[test]
    fn dirty_player_armour_bonus_is_the_attackers_own_skill_value() {
        use ffb_model::model::SkillWithValue;
        let mut game = game_with_attacker_and_defender(vec![], 13);
        let att = game.team_home.players.last_mut().unwrap();
        att.starting_skills.push(SkillWithValue {
            skill_id: SkillId::DirtyPlayer,
            value: Some("2".into()),
        });
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_DIRTY_PLAYER_2));
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_DIRTY_PLAYER_1));
    }

    /// The mutual exclusion must key off the modifier NAME, not the +1 constant: with a Dirty
    /// Player (2) attacker the armour list holds ARMOR_DIRTY_PLAYER_2, and an equality test
    /// against the +1 constant would miss it and let Dirty Player boost both rolls.
    #[test]
    fn dirty_player_two_on_armour_still_excludes_the_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 2);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.ctx.add_armor_modifier(ARMOR_DIRTY_PLAYER_2);
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&dirty_player_injury_modifier(game.rules)));
    }

    #[test]
    fn no_dirty_player_no_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_DIRTY_PLAYER_1));
    }
    #[test]
    fn dirty_player_adds_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 2);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(t.ctx.injury_modifiers.contains(&dirty_player_injury_modifier(game.rules)));
    }
    #[test]
    fn no_dirty_player_no_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&dirty_player_injury_modifier(game.rules)));
    }
    // NOTE (test equalization): no_attacker_id_no_dirty_player_modifier pruned — the
    // None-attacker path is a Rust Option-guard; Java's armourRoll derefs the attacker
    // unconditionally in UtilPlayer.findFoulAssists (a foul always has an attacker), so
    // there is no faithful Java twin.
    #[test]
    fn blatant_foul_card_sets_armor_broken() {
        let mut game = game_with_armor(13);
        game.turn_data_home.inducement_set.add_available_card("Blatant Foul");
        game.turn_data_home.inducement_set.activate_card("Blatant Foul");
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, None, "p1", true);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn no_blatant_foul_card_no_forced_armor_broken() {
        let game = game_with_armor(13);
        let mut t = InjuryTypeFoul::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, None, "p1", true);
        // armour 13 ensures the roll never breaks it
        assert!(!t.ctx.armor_broken);
    }
    #[test]
    fn chainsaw_foul_adds_chainsaw_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 2);
        let mut t = InjuryTypeFoul::new_with_chainsaw(true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
    #[test]
    fn chainsaw_foul_no_chainsaw_on_attacker_no_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeFoul::new_with_chainsaw(true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
    #[test]
    fn non_chainsaw_foul_no_chainsaw_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 2);
        let mut t = InjuryTypeFoul::new(); // use_chainsaw = false
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
    #[test]
    fn iron_hard_skin_defender_blocks_chainsaw_modifier() {
        let mut game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 2);
        // Give defender IronHardSkin (has IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS property)
        use ffb_model::model::SkillWithValue;
        game.team_away.players[0].extra_skills.push(SkillWithValue::new(SkillId::IronHardSkin));
        let mut t = InjuryTypeFoul::new_with_chainsaw(true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
}
