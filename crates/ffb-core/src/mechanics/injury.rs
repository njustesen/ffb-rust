use crate::model::game_state::GameState;
use crate::rng::GameRng;
use crate::skills::SkillId;
use crate::types::{CasualtyType, InjuryOutcome, PlayerId, PlayerState};

// ── Injury block mode ─────────────────────────────────────────────────────────

/// Controls how armor/injury modifiers (Mighty Blow, Claws, etc.) are applied
/// during block injury resolution, depending on the target relationship.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InjuryBlockMode {
    Regular,
    UseModifiersAgainstTeamMates,
    DoNotUseModifiers,
    /// BB2025: only armor modifiers (Claws, Mighty Blow) apply against team mates.
    UseArmourModifiersOnlyAgainstTeamMates,
}

// ── Armor outcome ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum ArmorOutcome {
    NotBroken,
    Broken,
}

/// Roll armor for a player with `av` armour value.
/// `mighty_blow_bonus`: +1 per Mighty Blow level (BB2025: 1 or 2).
/// `dirty_player_bonus`: +1 if attacker has Dirty Player skill (on foul).
///
/// Armor broken if 2d6 > av.
pub fn armor_roll(
    av: u8,
    mighty_blow_bonus: u8,
    dirty_player_bonus: u8,
    rng: &mut GameRng,
) -> ArmorOutcome {
    let roll = rng.roll_2d6() + mighty_blow_bonus + dirty_player_bonus;
    if roll > av {
        ArmorOutcome::Broken
    } else {
        ArmorOutcome::NotBroken
    }
}

// ── Injury outcome ────────────────────────────────────────────────────────────

/// Roll injury after armor is broken.
/// 2d6 sum: 2–7 = Stunned, 8–9 = KO, 10–12 = Casualty.
/// `mighty_blow_bonus`: +1 for Mighty Blow.
pub fn injury_roll(mighty_blow_bonus: u8, rng: &mut GameRng) -> InjuryOutcome {
    let roll = rng.roll_2d6() + mighty_blow_bonus;
    match roll {
        ..=7 => InjuryOutcome::Stunned,
        8 | 9 => InjuryOutcome::KnockedOut,
        _ => InjuryOutcome::Casualty,
    }
}

/// Roll on the casualty table: d16 (actually 2d8 → sum 2–16).
/// Returns `CasualtyType`.
pub fn casualty_roll(rng: &mut GameRng) -> CasualtyType {
    let roll = rng.roll_2d8();
    CasualtyType::from_d16(roll)
}

// ── Full injury resolution ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct InjuryResolution {
    pub new_state: PlayerState,
    pub casualty: Option<CasualtyType>,
}

/// Resolve injury for a player who has had their armor broken.
/// Rolls injury and optionally casualty; returns resulting player state and
/// any casualty type.
pub fn resolve_injury(
    mighty_blow_bonus: u8,
    rng: &mut GameRng,
) -> InjuryResolution {
    match injury_roll(mighty_blow_bonus, rng) {
        InjuryOutcome::Stunned => InjuryResolution {
            new_state: PlayerState::Stunned,
            casualty: None,
        },
        InjuryOutcome::KnockedOut => InjuryResolution {
            new_state: PlayerState::Ko,
            casualty: None,
        },
        InjuryOutcome::Casualty => {
            let cas = casualty_roll(rng);
            InjuryResolution {
                new_state: PlayerState::Injured,
                casualty: Some(cas),
            }
        }
    }
}

// ── PutTheBootIn ──────────────────────────────────────────────────────────────

/// Roll armor twice and return the result with the higher (better for attacker)
/// total. Used when the fouling player has the PutTheBootIn skill.
/// A higher roll is better for the attacker (more likely to break armor).
pub fn put_the_boot_in_reroll(av: u8, rng: &mut GameRng) -> ArmorOutcome {
    let roll1 = rng.roll_2d6();
    let roll2 = rng.roll_2d6();
    let best = roll1.max(roll2);
    if best > av {
        ArmorOutcome::Broken
    } else {
        ArmorOutcome::NotBroken
    }
}

// ── Regeneration ──────────────────────────────────────────────────────────────

/// Apply Regeneration skill after KO or Injury.
/// If the player has Regeneration and their current state is Ko or Injured,
/// roll d6: on 4+ set state to Stunned and return true.
/// Returns false if skill absent, wrong state, or roll failed.
pub fn apply_regeneration(
    state: &mut GameState,
    player_id: &PlayerId,
    rng: &mut GameRng,
) -> bool {
    let current_state = match state.field.player_state(player_id) {
        Some(s) => s,
        None => return false,
    };

    if !matches!(current_state, PlayerState::Ko | PlayerState::Injured) {
        return false;
    }

    let has_regen = state.home.player_by_id(player_id)
        .or_else(|| state.away.player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::Regeneration))
        .unwrap_or(false);

    if !has_regen {
        return false;
    }

    let roll = rng.roll_d6();
    if roll >= 4 {
        // Blood Bowl rule: successful Regeneration places the player Ko'd
        // (they go to the dugout KO box rather than the injury box).
        state.field.set_player_state(player_id, PlayerState::Ko);
        true
    } else {
        false
    }
}

// ── Apothecary ────────────────────────────────────────────────────────────────

/// Apply apothecary to a casualty result.
/// Rolls a new casualty; returns the better (less severe) of the two.
/// Second return value is true if the reroll improved the outcome.
pub fn apply_apothecary(
    original: CasualtyType,
    rng: &mut GameRng,
) -> (CasualtyType, bool) {
    let reroll = casualty_roll(rng);
    if reroll.is_worse_than(original) {
        // Original was better — keep it
        (original, false)
    } else if original.is_worse_than(reroll) {
        // Reroll was better
        (reroll, true)
    } else {
        // Equal severity — no improvement
        (original, false)
    }
}

// ── Decay ─────────────────────────────────────────────────────────────────────

/// Resolve injury with skill modifiers: Decay (double casualty roll), ThickSkull (KO→Stunned on 8).
pub fn resolve_injury_with_decay(
    player_id: &PlayerId,
    state: &GameState,
    mighty_blow_bonus: u8,
    rng: &mut GameRng,
) -> InjuryResolution {
    let (has_decay, has_thick_skull) = state.home.player_by_id(player_id)
        .or_else(|| state.away.player_by_id(player_id))
        .map(|p| (p.has_skill(SkillId::Decay), p.has_skill(SkillId::ThickSkull)))
        .unwrap_or((false, false));

    // ThickSkull: roll 8 on injury table (after MB) → Stunned instead of KO
    let roll = rng.roll_2d6() + mighty_blow_bonus;
    let base_outcome = if has_thick_skull && roll == 8 {
        InjuryOutcome::Stunned
    } else {
        match roll {
            ..=7 => InjuryOutcome::Stunned,
            8 | 9 => InjuryOutcome::KnockedOut,
            _ => InjuryOutcome::Casualty,
        }
    };

    let base = match base_outcome {
        InjuryOutcome::Stunned => InjuryResolution { new_state: PlayerState::Stunned, casualty: None },
        InjuryOutcome::KnockedOut => InjuryResolution { new_state: PlayerState::Ko, casualty: None },
        InjuryOutcome::Casualty => {
            let cas = casualty_roll(rng);
            InjuryResolution { new_state: PlayerState::Injured, casualty: Some(cas) }
        }
    };

    if has_decay && base.casualty.is_some() {
        // Decay: roll casualty table twice, apply worst result
        let second = casualty_roll(rng);
        let worst = if base.casualty.as_ref().map(|c| c.severity()).unwrap_or(0) >= second.severity() {
            base.casualty
        } else {
            Some(second)
        };
        InjuryResolution { new_state: PlayerState::Injured, casualty: worst }
    } else {
        base
    }
}

// ── NurglesRot ────────────────────────────────────────────────────────────────

/// NurglesRot: post-game effect tracking. Returns true if the attacker has
/// NurglesRot and the victim was killed (Dead casualty).
/// The actual roster modification is handled in post-game processing.
pub fn nurgle_rot_applies(
    attacker_id: &PlayerId,
    state: &GameState,
    victim_casualty: Option<CasualtyType>,
) -> bool {
    if victim_casualty != Some(CasualtyType::Dead) {
        return false;
    }
    state.home.player_by_id(attacker_id)
        .or_else(|| state.away.player_by_id(attacker_id))
        .map(|p| p.has_skill(SkillId::NurglesRot))
        .unwrap_or(false)
}

// ── T-56/57 skill helpers ─────────────────────────────────────────────────────

/// T-56 #5: SlashingNails — when fouling, injury roll treated as Mighty Blow +2.
/// Returns 2 if attacker has SlashingNails, 0 otherwise.
pub fn slashing_nails_injury_bonus(state: &GameState, attacker_id: &PlayerId) -> u8 {
    let has = state.home.player_by_id(attacker_id)
        .or_else(|| state.away.player_by_id(attacker_id))
        .map(|p| p.has_skill(SkillId::SlashingNails))
        .unwrap_or(false);
    if has { 2 } else { 0 }
}

/// T-56 #6: MasterAssassin — when fouling causes an injury, it's automatically
/// a Casualty. Returns true if fouler has MasterAssassin.
pub fn master_assassin_auto_cas(state: &GameState, fouler_id: &PlayerId) -> bool {
    state.home.player_by_id(fouler_id)
        .or_else(|| state.away.player_by_id(fouler_id))
        .map(|p| p.has_skill(SkillId::MasterAssassin))
        .unwrap_or(false)
}

/// T-56 #1: FrenziedRush — after a TD, player moves 4 squares toward nearest
/// opponent. Returns true if scorer has FrenziedRush.
pub fn frenzied_rush_trigger(state: &GameState, scorer_id: &PlayerId) -> bool {
    state.home.player_by_id(scorer_id)
        .or_else(|| state.away.player_by_id(scorer_id))
        .map(|p| p.has_skill(SkillId::FrenziedRush))
        .unwrap_or(false)
}

/// T-57 #8: PutridRegurgitation — once per game, vomit on adjacent opponent.
/// Rolls d6: on 4+ sets target to Ko (no armor roll), returns true.
pub fn putrid_regurgitation(
    state: &mut GameState,
    player_id: &PlayerId,
    target_id: &PlayerId,
    rng: &mut GameRng,
) -> bool {
    let has = state.home.player_by_id(player_id)
        .or_else(|| state.away.player_by_id(player_id))
        .map(|p| p.has_skill(SkillId::PutridRegurgitation))
        .unwrap_or(false);
    if !has {
        return false;
    }
    let roll = rng.roll_d6();
    if roll >= 4 {
        state.field.set_player_state(target_id, PlayerState::Ko);
        true
    } else {
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game_state::GameState;
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::rng::GameRng;
    use crate::skills::{SkillId, SkillSet};
    use crate::types::{FieldCoordinate, PlayerId, TeamId};

    fn make_regen_state(has_regen: bool, player_state: PlayerState) -> (GameState, PlayerId) {
        let pid = PlayerId("p1".into());
        let skills = if has_regen {
            [SkillId::Regeneration].into_iter().collect()
        } else {
            SkillSet::empty()
        };
        let player = Player::new(
            pid.clone(),
            "Regen Player".into(),
            "zombie".into(),
            TeamId::Home,
            1,
            PlayerStats::new(4, 3, 3, 8, None),
            skills,
        );
        let mut home = Team::new("h".into(), "Undead".into(), "Undead".into(), 2, true);
        home.add_player(player);
        let away = Team::new("a".into(), "Humans".into(), "Human".into(), 2, false);
        let mut state = GameState::new(home, away);
        state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(5, 5), player_state);
        (state, pid)
    }

    #[test]
    fn armor_broken_when_roll_exceeds_av() {
        // AV=8; roll 2+7=9 → broken
        let mut rng = GameRng::new_test([5, 4]); // 2d6 = 9
        assert_eq!(armor_roll(8, 0, 0, &mut rng), ArmorOutcome::Broken);
    }

    #[test]
    fn armor_not_broken_when_roll_equals_av() {
        // AV=8; roll 3+5=8 → NOT broken (must EXCEED av)
        let mut rng = GameRng::new_test([3, 5]);
        assert_eq!(armor_roll(8, 0, 0, &mut rng), ArmorOutcome::NotBroken);
    }

    #[test]
    fn mighty_blow_helps_break_armor() {
        // AV=9; roll=9 without MB → not broken; +1 MB → 10 → broken
        let mut rng = GameRng::new_test([4, 5]); // 2d6=9
        assert_eq!(armor_roll(9, 1, 0, &mut rng), ArmorOutcome::Broken);
    }

    #[test]
    fn injury_roll_stunned_on_low_sum() {
        // 2d6 = 3+4=7 → Stunned
        let mut rng = GameRng::new_test([3, 4]);
        assert_eq!(injury_roll(0, &mut rng), InjuryOutcome::Stunned);
    }

    #[test]
    fn injury_roll_ko_on_mid_sum() {
        // 2d6 = 4+5=9 → KO
        let mut rng = GameRng::new_test([4, 5]);
        assert_eq!(injury_roll(0, &mut rng), InjuryOutcome::KnockedOut);
    }

    #[test]
    fn injury_roll_casualty_on_high_sum() {
        // 2d6 = 6+6=12 → Casualty
        let mut rng = GameRng::new_test([6, 6]);
        assert_eq!(injury_roll(0, &mut rng), InjuryOutcome::Casualty);
    }

    #[test]
    fn casualty_all_outcomes_reachable() {
        // casualty_roll uses roll_2d8; inject d8 pairs that produce sums 2..=16
        // Sums 2–8: [1+1, 1+2, 1+3, 1+4, 1+5, 1+6, 1+7]
        // Sums 9–16: [1+8, 2+8, 3+8, 4+8, 5+8, 6+8, 7+8, 8+8]
        let rolls: Vec<u8> = vec![
            1, 1,  // sum 2
            1, 2,  // sum 3
            1, 3,  // sum 4
            1, 4,  // sum 5
            1, 5,  // sum 6
            1, 6,  // sum 7
            1, 7,  // sum 8
            1, 8,  // sum 9
            2, 8,  // sum 10
            3, 8,  // sum 11
            4, 8,  // sum 12
            5, 8,  // sum 13
            6, 8,  // sum 14
            7, 8,  // sum 15
            8, 8,  // sum 16
        ];
        let mut rng = GameRng::new_test(rolls);
        for _ in 0..15 {
            let _ = casualty_roll(&mut rng);
        }
    }

    #[test]
    fn resolve_injury_stunned() {
        let mut rng = GameRng::new_test([2, 3]); // 2d6=5 → Stunned
        let res = resolve_injury(0, &mut rng);
        assert_eq!(res.new_state, PlayerState::Stunned);
        assert!(res.casualty.is_none());
    }

    #[test]
    fn resolve_injury_casualty() {
        let mut rng = GameRng::new_test([6, 6, 6, 6]); // injury=12→Cas, then 2d8
        let res = resolve_injury(0, &mut rng);
        assert_eq!(res.new_state, PlayerState::Injured);
        assert!(res.casualty.is_some());
    }

    #[test]
    fn armor_probability_statistical() {
        // P(break AV=8) = P(2d6 > 8) = P(2d6 ≥ 9) = P(9)+P(10)+P(11)+P(12)
        // = (4+3+2+1)/36 = 10/36 ≈ 0.278
        let mut rng = GameRng::new_live(42);
        let n = 10_000;
        let broken = (0..n)
            .filter(|_| armor_roll(8, 0, 0, &mut rng) == ArmorOutcome::Broken)
            .count();
        let p = broken as f64 / n as f64;
        assert!((p - 0.278).abs() < 0.02, "P(break AV8) = {p:.3}, expected ~0.278");
    }

    #[test]
    fn regeneration_succeeds_on_4plus() {
        let (mut state, pid) = make_regen_state(true, PlayerState::Ko);
        let mut rng = GameRng::new_test([4]); // 4 → success
        let result = apply_regeneration(&mut state, &pid, &mut rng);
        assert!(result);
        // Regeneration places the player Ko (dugout KO box) rather than injured
        assert_eq!(state.field.player_state(&pid), Some(PlayerState::Ko));
    }

    #[test]
    fn regeneration_fails_on_low_roll() {
        let (mut state, pid) = make_regen_state(true, PlayerState::Ko);
        let mut rng = GameRng::new_test([3]); // 3 → fail
        let result = apply_regeneration(&mut state, &pid, &mut rng);
        assert!(!result);
        assert_eq!(state.field.player_state(&pid), Some(PlayerState::Ko));
    }

    #[test]
    fn regeneration_no_skill_returns_false() {
        let (mut state, pid) = make_regen_state(false, PlayerState::Ko);
        let mut rng = GameRng::new_test([]);
        let result = apply_regeneration(&mut state, &pid, &mut rng);
        assert!(!result);
    }

    #[test]
    fn regeneration_not_triggered_when_stunned() {
        let (mut state, pid) = make_regen_state(true, PlayerState::Stunned);
        let mut rng = GameRng::new_test([]);
        let result = apply_regeneration(&mut state, &pid, &mut rng);
        assert!(!result);
    }

    #[test]
    fn regeneration_works_on_injured_state() {
        let (mut state, pid) = make_regen_state(true, PlayerState::Injured);
        let mut rng = GameRng::new_test([6]); // 6 → success
        let result = apply_regeneration(&mut state, &pid, &mut rng);
        assert!(result);
        // Regeneration places the player Ko (dugout KO box)
        assert_eq!(state.field.player_state(&pid), Some(PlayerState::Ko));
    }

    #[test]
    fn apothecary_improves_dead_to_badly_hurt() {
        // Original: Dead (severity 3); reroll: BadlyHurt (severity 0) → improved
        // casualty_roll uses roll_2d8; inject values summing to ≤6 → BadlyHurt
        let mut rng = GameRng::new_test([1, 1]); // 2d8 sum=2 → BadlyHurt
        let (result, improved) = apply_apothecary(CasualtyType::Dead, &mut rng);
        assert!(improved);
        assert_eq!(result, CasualtyType::BadlyHurt);
    }

    #[test]
    fn apothecary_no_improvement_keeps_original() {
        // Original: BadlyHurt; reroll gives something worse → keep BadlyHurt
        let mut rng = GameRng::new_test([8, 8]); // 2d8 sum=16 → SmashedCollarBone
        let (result, improved) = apply_apothecary(CasualtyType::BadlyHurt, &mut rng);
        assert!(!improved);
        assert_eq!(result, CasualtyType::BadlyHurt);
    }

    // ── T-56/57 skill helper tests ─────────────────────────────────────────

    fn make_state_with_single_skill(pid: &str, skill: SkillId, team: TeamId) -> (GameState, PlayerId) {
        let player_id = PlayerId(pid.into());
        let mut skills = SkillSet::empty();
        skills.add(skill);
        let player = crate::model::player::Player::new(
            player_id.clone(), pid.into(), "lineman".into(), team, 1,
            crate::model::player::PlayerStats::new(4, 3, 3, 8, None), skills,
        );
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Human".into(), 2, true);
        let mut away = crate::model::team::Team::new("a".into(), "Away".into(), "Orc".into(), 2, false);
        match team {
            TeamId::Home => home.add_player(player),
            TeamId::Away => away.add_player(player),
        }
        let state = GameState::new(home, away);
        (state, player_id)
    }

    #[test]
    fn slashing_nails_returns_two_with_skill() {
        let (state, pid) = make_state_with_single_skill("att", SkillId::SlashingNails, TeamId::Home);
        assert_eq!(slashing_nails_injury_bonus(&state, &pid), 2);
    }

    #[test]
    fn slashing_nails_returns_zero_without_skill() {
        let (state, pid) = make_state_with_single_skill("att", SkillId::Block, TeamId::Home);
        assert_eq!(slashing_nails_injury_bonus(&state, &pid), 0);
    }

    #[test]
    fn master_assassin_auto_cas_with_skill() {
        let (state, pid) = make_state_with_single_skill("att", SkillId::MasterAssassin, TeamId::Home);
        assert!(master_assassin_auto_cas(&state, &pid));
    }

    #[test]
    fn master_assassin_auto_cas_without_skill() {
        let (state, pid) = make_state_with_single_skill("att", SkillId::Block, TeamId::Home);
        assert!(!master_assassin_auto_cas(&state, &pid));
    }

    #[test]
    fn frenzied_rush_trigger_with_skill() {
        let (state, pid) = make_state_with_single_skill("scorer", SkillId::FrenziedRush, TeamId::Home);
        assert!(frenzied_rush_trigger(&state, &pid));
    }

    #[test]
    fn frenzied_rush_trigger_without_skill() {
        let (state, pid) = make_state_with_single_skill("scorer", SkillId::Block, TeamId::Home);
        assert!(!frenzied_rush_trigger(&state, &pid));
    }

    #[test]
    fn putrid_regurgitation_success_on_4plus() {
        // attacker has skill; roll 4 → target KO'd
        let target_id = PlayerId("target".into());
        let target_player = crate::model::player::Player::new(
            target_id.clone(), "target".into(), "lineman".into(), TeamId::Away, 2,
            crate::model::player::PlayerStats::new(4, 3, 3, 8, None), SkillSet::empty(),
        );
        let attacker_id = PlayerId("att".into());
        let mut att_skills = SkillSet::empty();
        att_skills.add(SkillId::PutridRegurgitation);
        let att_player = crate::model::player::Player::new(
            attacker_id.clone(), "att".into(), "lineman".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(4, 3, 3, 8, None), att_skills,
        );
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Nurgle".into(), 2, true);
        let mut away = crate::model::team::Team::new("a".into(), "Away".into(), "Human".into(), 2, false);
        home.add_player(att_player);
        away.add_player(target_player);
        let mut state = GameState::new(home, away);
        state.field.place_player(attacker_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(target_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);

        let mut rng = GameRng::new_test([4]); // 4 → success
        let result = putrid_regurgitation(&mut state, &attacker_id, &target_id, &mut rng);
        assert!(result);
        assert_eq!(state.field.player_state(&target_id), Some(PlayerState::Ko));
    }

    #[test]
    fn putrid_regurgitation_fail_on_low_roll() {
        let target_id = PlayerId("target".into());
        let target_player = crate::model::player::Player::new(
            target_id.clone(), "target".into(), "lineman".into(), TeamId::Away, 2,
            crate::model::player::PlayerStats::new(4, 3, 3, 8, None), SkillSet::empty(),
        );
        let attacker_id = PlayerId("att".into());
        let mut att_skills = SkillSet::empty();
        att_skills.add(SkillId::PutridRegurgitation);
        let att_player = crate::model::player::Player::new(
            attacker_id.clone(), "att".into(), "lineman".into(), TeamId::Home, 1,
            crate::model::player::PlayerStats::new(4, 3, 3, 8, None), att_skills,
        );
        let mut home = crate::model::team::Team::new("h".into(), "Home".into(), "Nurgle".into(), 2, true);
        let mut away = crate::model::team::Team::new("a".into(), "Away".into(), "Human".into(), 2, false);
        home.add_player(att_player);
        away.add_player(target_player);
        let mut state = GameState::new(home, away);
        state.field.place_player(attacker_id.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(target_id.clone(), TeamId::Away, FieldCoordinate::new(6, 5), PlayerState::Standing);

        let mut rng = GameRng::new_test([3]); // 3 → fail
        let result = putrid_regurgitation(&mut state, &attacker_id, &target_id, &mut rng);
        assert!(!result);
        assert_eq!(state.field.player_state(&target_id), Some(PlayerState::Standing));
    }

    // ── PutTheBootIn ──────────────────────────────────────────────────────────

    #[test]
    fn put_the_boot_in_takes_better_roll() {
        // AV=8; first roll = 3+4=7 (not broken), second roll = 5+5=10 (broken).
        // Best roll = 10 → broken.
        let mut rng = GameRng::new_test([3, 4, 5, 5]);
        assert_eq!(put_the_boot_in_reroll(8, &mut rng), ArmorOutcome::Broken);
    }

    #[test]
    fn put_the_boot_in_not_broken_when_both_low() {
        // AV=8; both rolls ≤ 8 → not broken.
        // roll1 = 3+4=7, roll2 = 2+3=5 → best = 7 → not broken (7 is not > 8)
        let mut rng = GameRng::new_test([3, 4, 2, 3]);
        assert_eq!(put_the_boot_in_reroll(8, &mut rng), ArmorOutcome::NotBroken);
    }

    #[test]
    fn put_the_boot_in_first_roll_good_enough() {
        // AV=7; first roll = 4+5=9 → already broken; second roll irrelevant.
        // (second roll still consumed from queue but doesn't matter)
        let mut rng = GameRng::new_test([4, 5, 1, 1]);
        assert_eq!(put_the_boot_in_reroll(7, &mut rng), ArmorOutcome::Broken);
    }

    // ── InjuryBlockMode (BB2025) ──────────────────────────────────────────────
    // Ported from InjuryTypeBlockBB2025Test.java

    #[test]
    fn use_armour_modifiers_only_against_team_mates_variant_exists() {
        let mode = InjuryBlockMode::UseArmourModifiersOnlyAgainstTeamMates;
        assert_eq!(mode, InjuryBlockMode::UseArmourModifiersOnlyAgainstTeamMates);
    }

    #[test]
    fn injury_block_mode_regular_is_constructible() {
        let mode = InjuryBlockMode::Regular;
        assert_eq!(mode, InjuryBlockMode::Regular);
    }

    #[test]
    fn all_four_injury_block_modes_exist() {
        // Exhaustive match ensures all 4 variants are present and distinct.
        let modes = [
            InjuryBlockMode::Regular,
            InjuryBlockMode::UseModifiersAgainstTeamMates,
            InjuryBlockMode::DoNotUseModifiers,
            InjuryBlockMode::UseArmourModifiersOnlyAgainstTeamMates,
        ];
        assert_eq!(modes.len(), 4);
        for (i, a) in modes.iter().enumerate() {
            for (j, b) in modes.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    // ── Regeneration casualty test ─────────────────────────────────────────────

    #[test]
    fn regeneration_on_4plus() {
        // Player with Regeneration suffers a casualty (Injured state).
        // On d6 roll of 4: regenerates to Ko instead of staying Injured.
        let (mut state, pid) = make_regen_state(true, PlayerState::Injured);
        let mut rng = GameRng::new_test([4]); // 4 → success
        let result = apply_regeneration(&mut state, &pid, &mut rng);
        assert!(result, "Regeneration should succeed on roll of 4");
        // State should be Ko (not Injured) — player goes to the KO dugout box
        assert_eq!(
            state.field.player_state(&pid),
            Some(PlayerState::Ko),
            "Regeneration on 4+ should change Injured to Ko"
        );
    }
}
