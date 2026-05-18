/// Typed modifier system for all roll types.
use crate::types::{FieldCoordinate, PlayerId, TeamId};

// ── Dodge ─────────────────────────────────────────────────────────────────────

pub struct DodgeCtx {
    pub player_id: PlayerId,
    pub team: TeamId,
    pub from: FieldCoordinate,
    pub to: FieldCoordinate,
    pub tackle_zones: u8,
    pub has_dodge_skill: bool,
    pub has_tackle_opponent: bool,
}

// ── Armor ─────────────────────────────────────────────────────────────────────

pub struct ArmorCtx {
    pub attacker_mighty_blow_level: u8,
    pub attacker_dirty_player_level: u8,
    pub is_foul: bool,
    pub is_crowd_push: bool,
}

impl ArmorCtx {
    pub fn mighty_blow_bonus(&self) -> u8 {
        self.attacker_mighty_blow_level
    }
    pub fn dirty_player_bonus(&self) -> u8 {
        if self.is_foul { self.attacker_dirty_player_level } else { 0 }
    }
}

// ── Injury ────────────────────────────────────────────────────────────────────

pub struct InjuryCtx {
    pub attacker_mighty_blow_level: u8,
    pub is_foul: bool,
    pub target_has_thick_skull: bool,
    pub target_has_stunty: bool,
}

impl InjuryCtx {
    pub fn net_bonus(&self) -> i8 {
        let mut m: i8 = self.attacker_mighty_blow_level as i8;
        if self.target_has_stunty { m += 1; }
        if self.target_has_thick_skull { m -= 1; }
        m
    }
}

// ── Pass ──────────────────────────────────────────────────────────────────────

pub struct PassCtx {
    pub has_accurate_skill: bool,
    pub has_nerves_of_steel: bool,
    pub opposing_tz_on_passer: u8,
}

impl PassCtx {
    pub fn net_modifier(&self) -> i8 {
        let mut m: i8 = 0;
        if self.has_accurate_skill { m += 1; }
        if !self.has_nerves_of_steel { m -= self.opposing_tz_on_passer as i8; }
        m
    }
}

// ── Catch ─────────────────────────────────────────────────────────────────────

pub struct CatchCtx {
    pub has_catch_skill: bool,
    pub has_nerves_of_steel: bool,
    pub has_extra_arms: bool,
    pub opposing_tz_on_catcher: u8,
}

impl CatchCtx {
    pub fn net_modifier(&self) -> i8 {
        let mut m: i8 = 0;
        if self.has_extra_arms { m += 1; }
        if !self.has_nerves_of_steel { m -= self.opposing_tz_on_catcher as i8; }
        m
    }

    pub fn has_reroll(&self) -> bool {
        self.has_catch_skill
    }
}

// ── Pickup ────────────────────────────────────────────────────────────────────

pub struct PickupCtx {
    pub player_ag: u8,
    pub opposing_tz: u8,
    pub has_sure_hands: bool,
    pub has_extra_arms: bool,
}

impl PickupCtx {
    pub fn min_roll(&self) -> u8 {
        let base = (5u8).saturating_sub(self.player_ag).max(2);
        let tz_mod = if self.has_extra_arms && self.opposing_tz > 0 {
            self.opposing_tz - 1
        } else {
            self.opposing_tz
        };
        (base + tz_mod).min(6)
    }
    pub fn has_reroll(&self) -> bool {
        self.has_sure_hands
    }
}

// ── Block context ─────────────────────────────────────────────────────────────

pub struct BlockCtx {
    pub attacker_st: u8,
    pub defender_st: u8,
    pub attacker_has_block: bool,
    pub attacker_has_wrestle: bool,
    pub attacker_has_juggernaut: bool,
    pub attacker_has_frenzy: bool,
    pub defender_has_block: bool,
    pub defender_has_wrestle: bool,
    pub defender_has_dodge: bool,
    pub is_blitz: bool,
    pub guard_bonus_attacker: u8,
    pub guard_bonus_defender: u8,
}

impl BlockCtx {
    pub fn dice_count(&self) -> i8 {
        crate::mechanics::block_dice_count(
            self.attacker_st,
            self.defender_st,
            self.guard_bonus_attacker,
            self.guard_bonus_defender,
        )
    }
}
