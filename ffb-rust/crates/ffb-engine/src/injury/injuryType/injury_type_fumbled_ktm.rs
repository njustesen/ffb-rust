/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeFumbledKtm.
/// Armor always broken. Injury roll with block-property modifiers (stub). stunIsTreatedAsKo=true.
use ffb_model::enums::{ApothecaryMode, SendToBoxReason, PS_PRONE, SkillId};
use ffb_model::model::property::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_injury_roll_for_player_no_stunty};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeFumbledKtm { ctx: InjuryContext }
impl InjuryTypeFumbledKtm { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeFumbledKtm { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeFumbledKtm {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        // Java: factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(), isFoul(),
        // isVomitLike()) — FumbledKtm never overrides isStab/isFoul/isVomitLike (all false) — then
        // .filter(injuryModifier -> injuryModifier.isRegisteredToSkillWithProperty(affectsEitherArmourOrInjuryOnBlock)).
        if let Some(defender) = game.player(defender_id) {
            let attacker = attacker_id.and_then(|aid| game.player(aid));
            let factory = InjuryModifierFactory::new(game.rules);
            for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                let registered_to_block = m.registered_to()
                    .and_then(SkillId::from_class_name)
                    .map(|id| id.properties().contains(&NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_BLOCK))
                    .unwrap_or(false);
                if registered_to_block {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
            }
        }
        // Java reads "is stunty" off the INJURY MODIFIERS in the context
        // (`RollMechanic`: anyMatch(isRegisteredToSkillWithProperty(isHurtMoreEasily))), not off the
        // defender's skills. Normally the two agree — but this type FILTERS the modifier set down to
        // `affectsEitherArmourOrInjuryOnBlock` modifiers, which drops Stunty's, so Java never sees a
        // stunty modifier here. Hence the no-stunty variant: a fumbled kick on a Snotling totalling
        // 9 is a plain KNOCKED_OUT, not the 9-and-stunty BADLY_HURT (ogre bb2020 seed 7 i=111).
        do_injury_roll_for_player_no_stunty(rng, &mut self.ctx, game, defender_id);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn stun_is_treated_as_ko(&self) -> bool { true }
    /// Java: `KTMFumbleInjury()` constructor passes `SendToBoxReason.KICKED`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Kicked) }
    /// Java: `KTMFumbleInjury.canApoKoIntoStun()` — false (unlike the base `InjuryType` default
    /// of true, and unlike `InjuryTypeFumbledKtmApoKo`'s sibling class).
    fn java_class_name(&self) -> &'static str { "InjuryTypeFumbledKtm" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_always_broken_and_injury_set() {
        let mut t = InjuryTypeFumbledKtm::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury.is_some());
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    /// Java reads "is stunty" off the INJURY MODIFIERS in the context
    /// (`RollMechanic`: anyMatch(isRegisteredToSkillWithProperty(isHurtMoreEasily))), not off the
    /// defender's skills. This type FILTERS its modifier set down to
    /// `affectsEitherArmourOrInjuryOnBlock` modifiers, which drops Stunty's — so Java never sees a
    /// stunty modifier here and a total of 9 is a plain KNOCKED_OUT, not the 9-and-stunty
    /// BADLY_HURT. Hence the no-stunty roll variant (ogre bb2020 seed 7 i=111).
    #[test]
    fn stunty_defender_does_not_get_the_stunty_injury_band() {
        use ffb_model::enums::{Rules, PS_KNOCKED_OUT, SkillId, PS_STANDING, PlayerState};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        // Seed chosen so the injury dice total 9 (4+5) — the band where stunty would change KO->BH.
        let mut seed = 0u64;
        let (d1, d2) = loop {
            let mut rng = GameRng::new(seed);
            let (a, b) = (rng.d6(), rng.d6());
            if a + b == 9 { break (a, b); }
            seed += 1;
        };
        assert_eq!(d1 + d2, 9);

        let mut game = Game::new(crate::step::framework::test_team("home", 0),
                                 crate::step::framework::test_team("away", 0), Rules::Bb2020);
        let mut p = Player { id: "snot".into(), name: "snot".into(), nr: 1, ..Default::default() };
        p.starting_skills = vec![SkillWithValue::new(SkillId::Stunty)];
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("snot", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("snot", PlayerState::new(PS_STANDING));

        let mut t = InjuryTypeFumbledKtm::new();
        let mut rng = GameRng::new(seed);
        t.handle_injury(&game, &mut rng, None, "snot", FieldCoordinate::new(5, 5),
                        None, None, ApothecaryMode::ThrownPlayer);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_KNOCKED_OUT),
            "a stunty defender must NOT get the 9-and-stunty BADLY_HURT band here — this type's              filtered modifier set contains no stunty modifier for Java to see");
    }

    #[test] fn stun_is_ko() { assert!(InjuryTypeFumbledKtm::new().stun_is_treated_as_ko()); }
    #[test]
    fn send_to_box_reason_is_kicked() {
        assert_eq!(InjuryTypeFumbledKtm::new().send_to_box_reason(), Some(ffb_model::enums::SendToBoxReason::Kicked));
    }
    #[test]
    fn cannot_apo_ko_into_stun() {
        // Regression test: Java `KTMFumbleInjury.canApoKoIntoStun()` returns false, unlike the
        // `InjuryType` base default of true.
        let t = InjuryTypeFumbledKtm::new();
        assert_eq!(t.java_class_name(), "InjuryTypeFumbledKtm");
        assert!(!crate::injury::can_apo_ko_into_stun(Some(t.java_class_name())));
    }
    #[test]
    fn causes_turnover_by_default() { assert!(InjuryTypeFumbledKtm::new().falling_down_causes_turnover()); }
    // NOTE (test equalization): context-storage and Default plumbing tests pruned — no
    // faithful Java twins (Java's CALLER sets the context ids).

    fn game_with_attacker_and_defender(attacker_skills: Vec<SkillId>, defender_niggling: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        fn make_player(id: &str, niggling_injuries: i32, skills: Vec<SkillId>) -> Player {
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
        // Bb2016 has niggling-injury modifiers (Bb2025 does not); use it so the niggling
        // reachability test actually exercises `get_niggling_injury_modifier`.
        Game::new(home, away, Rules::Bb2016)
    }

    /// Reachability proof: the `InjuryModifierFactory` call is wired in and the niggling-injury
    /// modifier the factory returns for a defender with prior niggling injuries is reached by the
    /// code path — but (matching Java's `isRegisteredToSkillWithProperty(affectsEitherArmourOrInjuryOnBlock)`
    /// filter) it is correctly excluded since niggling modifiers aren't registered to any skill.
    #[test]
    fn niggling_injury_modifier_reachable_but_filtered_by_block_property() {
        let game = game_with_attacker_and_defender(vec![], 1);
        let mut t = InjuryTypeFumbledKtm::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }

    /// Bug (fixed, #9): Java's MightyBlow skill DOES register affectsEitherArmourOrInjuryOnBlock,
    /// so the FumbledKtm block-property filter KEEPS Mighty Blow — verified against the Java
    /// twin (a fumbled KTM landing gets the kicker's Mighty Blow bonus). The old test asserted
    /// the opposite because the factory's modifiers carried no registered_to skill.
    #[test]
    fn mighty_blow_kept_by_block_property_filter() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 0);
        let mut t = InjuryTypeFumbledKtm::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
}
