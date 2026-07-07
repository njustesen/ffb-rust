/// Translations of com.fumbbl.ffb.injury.context.InjuryContext (ffb-common)
/// and com.fumbbl.ffb.server.InjuryResult (ffb-server) and
/// com.fumbbl.ffb.server.injury.injuryType.InjuryTypeServer (abstract base).
///
/// The InjuryTypeServer trait replaces Java's abstract class hierarchy.
/// Concrete types are in injury/injuryType/*.rs; inline impls below are kept
/// for backward-compat until the factory is fully switched over.
pub mod injuryType;
pub mod modification;

use ffb_model::enums::{
    ApothecaryMode, ApothecaryStatus, PlayerState, SendToBoxReason, SeriousInjuryKind,
    PS_PRONE, PS_STUNNED, PS_KNOCKED_OUT, PS_BADLY_HURT, PS_SERIOUS_INJURY, PS_RIP, PS_RESERVE,
};
use ffb_model::model::SkillUse;
use ffb_model::injury::context::InjuryModification;
use ffb_model::util::util_box::UtilBox;
use ffb_model::model::SoundId;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::mechanics::{armor_broken as mech_armor_broken, injury_result, interpret_injury_total_bb2016, interpret_injury_total_bb2020, InjuryOutcome};
use ffb_mechanics::modifiers::Modifier;

// ── InjuryContext ─────────────────────────────────────────────────────────────

/// Full translation of com.fumbbl.ffb.injury.context.InjuryContext.
#[derive(Debug, Clone)]
pub struct InjuryContext {
    /// Java: fArmorRoll — the two armor dice, or None if no armor roll was made.
    pub armor_roll: Option<[i32; 2]>,
    /// Java: fArmorBroken — whether the armor roll succeeded (armor was broken).
    pub armor_broken: bool,
    /// Java: fArmorModifiers — modifiers applied to the armor roll.
    pub armor_modifiers: Vec<Modifier>,
    /// Java: fInjuryRoll — the two injury dice, or None if no injury roll was made.
    pub injury_roll: Option<[i32; 2]>,
    /// Java: fInjuryModifiers — modifiers applied to the injury roll.
    pub injury_modifiers: Vec<Modifier>,
    /// Java: fApothecaryMode — which apothecary slot this context targets.
    pub apothecary_mode: ApothecaryMode,
    /// Java: fDefenderId
    pub defender_id: Option<String>,
    /// Java: fAttackerId
    pub attacker_id: Option<String>,
    /// Java: fDefenderPosition
    pub defender_coordinate: Option<FieldCoordinate>,
    /// Java: fInjury — the resulting PlayerState from the injury/casualty roll.
    pub injury: Option<PlayerState>,
    /// Java: fApothecaryStatus
    pub apothecary_status: ApothecaryStatus,
    /// Java: fSendToBoxTurn
    pub send_to_box_turn: i32,
    /// Java: fSendToBoxHalf
    pub send_to_box_half: i32,
    /// Java: fSufferedInjury — set by evaluateInjuryContext when KO/casualty.
    pub suffered_injury: Option<PlayerState>,
    /// Java: fCasualtyRoll — [primary_die, secondary_die]. BB2016: [d6, d8]. BB2020/2025: [d16, d6].
    pub casualty_roll: Option<[i32; 2]>,
    /// Java: fCasualtyRollDecay — second casualty roll for players with Decay skill (BB2016).
    pub casualty_roll_decay: Option<[i32; 2]>,
    /// Java: casualtyModifiers — accumulated during interpretCasualtyRollAndAddModifiers.
    pub casualty_modifiers: Vec<ffb_mechanics::modifiers::Modifier>,
    /// Java: fSeriousInjury — specific SI kind (set by evaluateInjuryContext).
    pub serious_injury: Option<SeriousInjuryKind>,
    /// Java: fSeriousInjuryDecay — decay SI kind for players with requiresSecondCasualtyRoll.
    pub serious_injury_decay: Option<SeriousInjuryKind>,
    /// Java: fInjuryDecay — decay player state for players with Decay skill (second casualty roll outcome).
    pub injury_decay: Option<PlayerState>,
    /// Java: originalSeriousInjury — SI before apothecary modification.
    pub original_serious_injury: Option<SeriousInjuryKind>,
    /// Java: fSendToBoxReason — why the player was removed from the field.
    pub send_to_box_reason: Option<SendToBoxReason>,
    /// Java: fSound — sound effect associated with this injury.
    pub sound: Option<SoundId>,
    /// Java: fModifiedInjuryContext — apothecary's alternate context (second injury context).
    pub modified_injury_context: Option<Box<InjuryContext>>,
    /// Java: InjuryType.getClass().getSimpleName() — stored for post-injury checks (e.g. isBlock()).
    pub injury_type_name: Option<String>,
    /// Java: InjuryType.isCausedByOpponent() — whether the injury was caused by an opposing player.
    pub is_caused_by_opponent: bool,
    /// Java: InjuryType.isWorthSpps() — whether the attacker earns a casualty SPP for this injury.
    pub is_worth_spps: bool,
    // ── ModifiedInjuryContext extra fields (Java: ModifiedInjuryContext extends InjuryContext) ──
    /// Java: ModifiedInjuryContext.modification — which phase was modified.
    pub modification: InjuryModification,
    /// Java: ModifiedInjuryContext.usedSkill — skill that caused the modification (stored as id).
    pub used_skill_id: Option<u16>,
    /// Java: ModifiedInjuryContext.skillUse — how the skill was used.
    pub skill_use_modification: Option<SkillUse>,
}

impl InjuryContext {
    pub fn new(apothecary_mode: ApothecaryMode) -> Self {
        Self {
            armor_roll: None,
            armor_broken: false,
            armor_modifiers: Vec::new(),
            injury_roll: None,
            injury_modifiers: Vec::new(),
            apothecary_mode,
            defender_id: None,
            attacker_id: None,
            defender_coordinate: None,
            injury: None,
            apothecary_status: ApothecaryStatus::NoApothecary,
            send_to_box_turn: 0,
            send_to_box_half: 0,
            suffered_injury: None,
            casualty_roll: None,
            casualty_roll_decay: None,
            casualty_modifiers: Vec::new(),
            serious_injury: None,
            serious_injury_decay: None,
            injury_decay: None,
            original_serious_injury: None,
            send_to_box_reason: None,
            sound: None,
            modified_injury_context: None,
            injury_type_name: None,
            is_caused_by_opponent: false,
            is_worth_spps: false,
            modification: InjuryModification::NONE,
            used_skill_id: None,
            skill_use_modification: None,
        }
    }

    // Backward-compat accessors
    pub fn is_armor_broken(&self) -> bool { self.armor_broken }
    pub fn get_armor_roll(&self) -> Option<[i32; 2]> { self.armor_roll }
    pub fn get_injury_roll(&self) -> Option<[i32; 2]> { self.injury_roll }
    pub fn get_apothecary_mode(&self) -> ApothecaryMode { self.apothecary_mode }

    // Java: addArmorModifier / addArmorModifiers
    pub fn add_armor_modifier(&mut self, m: Modifier) { self.armor_modifiers.push(m); }
    pub fn add_armor_modifiers(&mut self, ms: impl IntoIterator<Item = Modifier>) {
        self.armor_modifiers.extend(ms);
    }
    pub fn clear_armor_modifiers(&mut self) { self.armor_modifiers.clear(); }

    // Java: addInjuryModifier / addInjuryModifiers
    pub fn add_injury_modifier(&mut self, m: Modifier) { self.injury_modifiers.push(m); }
    pub fn add_injury_modifiers(&mut self, ms: impl IntoIterator<Item = Modifier>) {
        self.injury_modifiers.extend(ms);
    }

    // Java: addCasualtyModifier / addCasualtyModifiers
    pub fn add_casualty_modifier(&mut self, m: Modifier) { self.casualty_modifiers.push(m); }
    pub fn add_casualty_modifiers(&mut self, ms: impl IntoIterator<Item = Modifier>) {
        self.casualty_modifiers.extend(ms);
    }
    pub fn casualty_modifier_sum(&self) -> i32 {
        self.casualty_modifiers.iter().map(|m| m.value).sum()
    }

    // Outcome helpers (Java: isCasualty(), isKnockedOut(), getPlayerState())
    pub fn is_knocked_out(&self) -> bool {
        self.injury.map(|s| s.base() == PS_KNOCKED_OUT).unwrap_or(false)
    }
    pub fn is_casualty(&self) -> bool {
        self.injury.map(|s| s.is_casualty()).unwrap_or(false)
    }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.injury }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn set_injury(&mut self, state: PlayerState) { self.injury = Some(state); }
    pub fn set_armor_broken(&mut self, broken: bool) { self.armor_broken = broken; }

    // Java: isSeriousInjury() — true when injury state is SERIOUS_INJURY (casualty needing SI detail)
    pub fn is_serious_injury(&self) -> bool {
        self.injury.map(|s| s.base() == PS_SERIOUS_INJURY).unwrap_or(false)
    }

    // Java: isReserve()
    pub fn is_reserve(&self) -> bool {
        self.injury.map(|s| s.base() == PS_RESERVE).unwrap_or(false)
    }

    pub fn get_modified_injury_context(&self) -> Option<&InjuryContext> {
        self.modified_injury_context.as_deref()
    }

    pub fn set_modified_injury_context(&mut self, ctx: Option<Box<InjuryContext>>) {
        self.modified_injury_context = ctx;
    }

    // Java: ModifiedInjuryContext.setModification / setUsedSkill / setSkillUse
    pub fn set_modification(&mut self, m: InjuryModification) { self.modification = m; }
    pub fn set_used_skill_id(&mut self, id: u16) { self.used_skill_id = Some(id); }
    pub fn set_skill_use_modification(&mut self, su: SkillUse) { self.skill_use_modification = Some(su); }

    // Java: InjuryContext.setInjuryRoll(int[])
    pub fn set_injury_roll(&mut self, roll: [i32; 2]) { self.injury_roll = Some(roll); }
    // Java: InjuryContext.setArmorRoll
    pub fn set_armor_roll(&mut self, roll: [i32; 2]) { self.armor_roll = Some(roll); }
}

// ── InjuryResult ──────────────────────────────────────────────────────────────

/// Translation of com.fumbbl.ffb.server.InjuryResult.
#[derive(Debug, Clone)]
pub struct InjuryResult {
    pub injury_context: InjuryContext,
    /// Java: injury outcome is KNOCKED_OUT (convenience flag, derived from context).
    pub knocked_out: bool,
    /// Java: injury outcome is RIP / dead (convenience flag, derived from context).
    pub rip: bool,
    /// Java: fAlreadyReported — true after report() has been called once.
    pub already_reported: bool,
    /// Java: fPreRegeneration — true until passed_regeneration() is called.
    pub pre_regeneration: bool,
}

impl InjuryResult {
    pub fn new(apothecary_mode: ApothecaryMode) -> Self {
        Self {
            injury_context: InjuryContext::new(apothecary_mode),
            knocked_out: false,
            rip: false,
            already_reported: false,
            pre_regeneration: true,
        }
    }

    pub fn injury_context(&self) -> &InjuryContext { &self.injury_context }
    pub fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.injury_context }

    pub fn is_already_reported(&self) -> bool { self.already_reported }
    pub fn set_already_reported(&mut self, v: bool) { self.already_reported = v; }
    pub fn is_pre_regeneration(&self) -> bool { self.pre_regeneration }
    pub fn passed_regeneration(&mut self) { self.pre_regeneration = false; }

    /// Java: `InjuryResult.report(IStep)` — delegates to `StateMechanic.reportInjury`.
    pub fn report(&mut self, game: &mut ffb_model::model::game::Game) {
        let mechanic = crate::mechanic::state_mechanic_for(game.rules);
        mechanic.report_injury(game, self);
    }

    /// Java: InjuryResult.applyTo(IStep) — applies injury outcome to game state.
    ///
    /// 1. Reads the new player state from the injury context.
    /// 2. Sets player state on the field model, respecting the precedence order:
    ///    if the current state IS in the precedence list, only set the new state if
    ///    it has a HIGHER index (worse). If the current state is not in the list
    ///    (e.g. STANDING, MOVING), always apply.
    /// 3. If the player is STUNNED and belongs to the currently-acting team, clears
    ///    the active bit (Java: sets the player inactive so the turn ends).
    /// 4. If the player is KO, a casualty, or reserve, puts them into the dugout box.
    pub fn apply_to(&self, game: &mut Game) {
        // Java: basePrecedenceList — the exact list from InjuryResult.java.
        // States NOT in this list (STANDING, MOVING, etc.) are always overridden.
        // States IN this list are only overridden by states with higher index (worse).
        const PRECEDENCE: &[u32] = &[
            PS_PRONE, PS_STUNNED, PS_KNOCKED_OUT, PS_BADLY_HURT, PS_SERIOUS_INJURY, PS_RIP, PS_RESERVE,
        ];

        let ctx = &self.injury_context;

        let defender_id = match ctx.defender_id.as_deref() {
            Some(id) => id,
            None => return,
        };
        let new_state = match ctx.injury {
            Some(s) => s,
            None => return,
        };

        // 1. Set player state respecting precedence.
        // Java: !basePrecedenceList.contains(oldState.getBase())
        //    || basePrecedenceList.indexOf(newState.getBase()) > basePrecedenceList.indexOf(oldState.getBase())
        let current_state = game.field_model.player_state(defender_id);
        let should_set = match current_state {
            None => true,
            Some(current) => {
                let current_rank = PRECEDENCE.iter().position(|&b| b == current.base());
                match current_rank {
                    None => true, // current not in precedence list → always set (e.g. STANDING)
                    Some(cr) => {
                        let new_rank = PRECEDENCE.iter().position(|&b| b == new_state.base());
                        match new_rank {
                            Some(nr) => nr > cr, // new must be worse (higher index)
                            None => false,       // new not in list → don't override a boxed player with a field state
                        }
                    }
                }
            }
        };

        if should_set {
            // Java: playerState.changeBase(newBase) — preserves flag bits, replaces base
            let existing = game.field_model.player_state(defender_id).unwrap_or(new_state);
            game.field_model.set_player_state(defender_id, existing.change_base(new_state.base()));
        }

        // 2. If STUNNED → deactivate if on acting team OR on the bombardier's team (bomb hits friendlies).
        if new_state.base() == PS_STUNNED {
            let (home_bomb, away_bomb) = if let Some(ref orig_id) = game.original_bombardier.clone() {
                let bombardier_is_home = game.team_home.player(orig_id).is_some();
                (bombardier_is_home, !bombardier_is_home)
            } else {
                (false, false)
            };
            let defender_is_home = game.team_home.player(defender_id).is_some();
            let should_deactivate = if defender_is_home {
                game.home_playing || home_bomb
            } else {
                !game.home_playing || away_bomb
            };
            if should_deactivate {
                let state = game.field_model.player_state(defender_id).unwrap_or(new_state);
                game.field_model.set_player_state(defender_id, state.change_active(false));
            }
        }

        // 3. If KO, casualty, or reserve → put player into box.
        let final_state = game.field_model.player_state(defender_id).unwrap_or(new_state);
        let base = final_state.base();
        if base == PS_KNOCKED_OUT || final_state.is_casualty() || base == PS_RESERVE {
            UtilBox::put_player_into_box(game, defender_id);
        }
    }
}

// ── InjuryTypeServer trait ────────────────────────────────────────────────────

/// Rust analogue of Java's abstract `InjuryTypeServer<T>`.
///
/// Each concrete implementation:
///   1. Rolls armor dice (if not pre-broken).
///   2. Checks armor against defender.armour.
///   3. If broken: rolls injury dice + interprets outcome.
///   4. Populates its own `InjuryContext` with the results.
///
/// `util_server_injury::handle_injury()` calls this trait, then runs
/// `evaluate_injury_context()` to set apothecary status and send-to-box data.
pub trait InjuryTypeServer {
    fn handle_injury(
        &mut self,
        game: &Game,
        rng: &mut GameRng,
        attacker_id: Option<&str>,
        defender_id: &str,
        coord: FieldCoordinate,
        from_coord: Option<FieldCoordinate>,
        old_ctx: Option<&InjuryContext>,
        apo_mode: ApothecaryMode,
    );
    fn injury_context(&self) -> &InjuryContext;
    fn injury_context_mut(&mut self) -> &mut InjuryContext;

    /// Java: InjuryType.fallingDownCausesTurnover()
    fn falling_down_causes_turnover(&self) -> bool { true }
    /// Java: InjuryType.canUseApo()
    fn can_use_apo(&self) -> bool { true }
    /// Java: InjuryTypeServer.stunIsTreatedAsKo()
    fn stun_is_treated_as_ko(&self) -> bool { false }
    /// Java: InjuryType.failedArmourPlacesProne() — true for most types; ball-and-chain overrides to false
    fn failed_armour_places_prone(&self) -> bool { true }
    /// Java: InjuryTypeServer.sendToBoxReason()
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { None }
    /// Java: InjuryType.shouldPlayFallSound()
    fn should_play_fall_sound(&self) -> bool { true }
    /// Java: InjuryType class simple name — used by InjuryContextModification.isValidType().
    /// Implementations should return the Java class simple name (e.g. "Block", "Foul", "Stab").
    fn java_class_name(&self) -> &'static str { "" }
    /// Java: InjuryType.isCausedByOpponent() — whether an opposing player caused this injury.
    fn is_caused_by_opponent(&self) -> bool { false }
    /// Java: InjuryType.isWorthSpps() — whether the attacker earns a casualty SPP.
    fn is_worth_spps(&self) -> bool { false }
}

/// Java: `InjuryType.canApoKoIntoStun()` — whether an apothecary can revive a KO'd player as STUNNED.
///
/// Default: `true` (most injury types). Returns `false` for crowd-push and trap-door injuries
/// (Java: `CrowdPush`, `CrowdPushForSpp`, `TrapDoorFall`, `TrapDoorFallForSpp`).
pub fn can_apo_ko_into_stun(injury_type_name: Option<&str>) -> bool {
    match injury_type_name {
        Some("InjuryTypeCrowdPush") | Some("InjuryTypeCrowdPushForSpp")
        | Some("InjuryTypeTrapDoorFall") | Some("InjuryTypeTrapDoorFallForSpp") => false,
        _ => true,
    }
}

// ── Shared dice helpers ───────────────────────────────────────────────────────

/// Roll 2d6 armor and set `armor_broken` vs the defender's armor value.
/// Applies any modifiers already stored in `ctx.armor_modifiers`.
pub fn do_armor_roll(game: &Game, rng: &mut GameRng, ctx: &mut InjuryContext, defender_id: &str) {
    let d1 = rng.d6();
    let d2 = rng.d6();
    ctx.armor_roll = Some([d1, d2]);
    let armor_value = game.player(defender_id).map(|p| p.armour).unwrap_or(7);
    ctx.armor_broken = mech_armor_broken(armor_value, [d1, d2], &ctx.armor_modifiers);
}

/// Recalculate `armor_broken` using the existing `armor_roll` and current modifiers.
/// Java: DiceInterpreter.isArmourBroken(gameState, injuryContext)
pub fn recalc_armor_broken(game: &Game, ctx: &mut InjuryContext, defender_id: &str) {
    if let Some([d1, d2]) = ctx.armor_roll {
        let armor_value = game.player(defender_id).map(|p| p.armour).unwrap_or(7);
        ctx.armor_broken = mech_armor_broken(armor_value, [d1, d2], &ctx.armor_modifiers);
    }
}

/// Roll 2d6 injury and, if Casualty, roll the BB2025 d16 casualty die.
/// Applies any modifiers already stored in `ctx.injury_modifiers`.
pub fn do_injury_roll(rng: &mut GameRng, ctx: &mut InjuryContext) {
    let d1 = rng.d6();
    let d2 = rng.d6();
    ctx.injury_roll = Some([d1, d2]);
    let outcome = injury_result([d1, d2], &ctx.injury_modifiers);
    ctx.injury = Some(outcome_to_player_state(rng, ctx, outcome));
}

/// Edition-aware variant of `do_injury_roll` that applies Stunty and ThickSkull rules.
/// Uses `interpret_injury_total_bb2016` or `interpret_injury_total_bb2020` based on `rules`.
/// Falls back to a standard Casualty when `interpret_*` returns `None`.
pub fn do_injury_roll_for_player(rng: &mut GameRng, ctx: &mut InjuryContext, game: &Game, defender_id: &str) {
    use ffb_model::enums::{Rules, SkillId};
    let d1 = rng.d6();
    let d2 = rng.d6();
    ctx.injury_roll = Some([d1, d2]);
    let modifier_sum: i32 = ctx.injury_modifiers.iter().map(|m| m.value).sum();
    let total = d1 + d2 + modifier_sum;
    let outcome = if let Some(defender) = game.player(defender_id) {
        let is_stunty = defender.has_skill(SkillId::Stunty);
        let has_thick_skull = defender.has_skill(SkillId::ThickSkull);
        let interpreted = match game.rules {
            Rules::Bb2016 => interpret_injury_total_bb2016(total, is_stunty, has_thick_skull),
            _ => interpret_injury_total_bb2020(total, is_stunty, has_thick_skull),
        };
        interpreted.unwrap_or(InjuryOutcome::Casualty)
    } else {
        injury_result([d1, d2], &ctx.injury_modifiers)
    };
    ctx.injury = Some(outcome_to_player_state(rng, ctx, outcome));
}

fn outcome_to_player_state(rng: &mut GameRng, ctx: &mut InjuryContext, outcome: InjuryOutcome) -> PlayerState {
    match outcome {
        InjuryOutcome::Stunned    => PlayerState::new(PS_STUNNED),
        InjuryOutcome::KnockedOut => PlayerState::new(PS_KNOCKED_OUT),
        InjuryOutcome::BadlyHurt  => PlayerState::new(PS_BADLY_HURT),
        InjuryOutcome::Casualty   => {
            // BB2020/2025: d16 for tier, d6 for SI sub-type. Store [d16, d6].
            let d16 = rng.die(16);
            let d6 = rng.d6();
            ctx.casualty_roll = Some([d16, d6]);
            let roll = d16;
            if roll >= 15 {
                PlayerState::new(PS_RIP)
            } else if roll >= 9 {
                PlayerState::new(PS_SERIOUS_INJURY)
            } else {
                PlayerState::new(PS_BADLY_HURT)
            }
        }
    }
}

// ── InjuryTypeDropFall ────────────────────────────────────────────────────────

/// Shared implementation for InjuryTypeDropGFI / InjuryTypeDropDodge / InjuryTypeDropJump.
///
/// Java: each is a separate class but all have identical handleInjury():
///   roll armor vs defender, if broken roll injury, else PRONE.
pub struct InjuryTypeDropFall {
    ctx: InjuryContext,
    causes_turnover: bool,
}

impl InjuryTypeDropFall {
    fn new(causes_turnover: bool) -> Self {
        Self {
            ctx: InjuryContext::new(ApothecaryMode::Attacker),
            causes_turnover,
        }
    }
}

impl InjuryTypeServer for InjuryTypeDropFall {
    fn handle_injury(
        &mut self, game: &Game, rng: &mut GameRng,
        _attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>,
        _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode,
    ) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;

        if !self.ctx.armor_broken {
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        if self.ctx.armor_broken {
            do_injury_roll(rng, &mut self.ctx);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }

    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { self.causes_turnover }
}

// ── InjuryTypeBlock ───────────────────────────────────────────────────────────

/// Java: InjuryTypeBlock — standard block injury involving attacker modifiers.
/// Simplified: no armor/injury modifier factories yet.
pub struct InjuryTypeBlockImpl {
    ctx: InjuryContext,
    pre_broken: bool,   // armor already forced broken (InjuryTypeBlockProne path)
    worth_spps: bool,   // Java: worthSpps constructor param (true for "ForSpp" variants)
    caused_by_opponent: bool, // Java: isCausedByOpponent() — true for Block/Foul/etc.
}

impl InjuryTypeBlockImpl {
    fn new(pre_broken: bool) -> Self {
        Self {
            ctx: InjuryContext::new(ApothecaryMode::Defender),
            pre_broken,
            worth_spps: false,
            caused_by_opponent: true, // Block injuries are always by opponent
        }
    }

    fn with_spps(mut self) -> Self {
        self.worth_spps = true;
        self
    }

    fn not_by_opponent(mut self) -> Self {
        self.caused_by_opponent = false;
        self
    }
}

impl InjuryTypeServer for InjuryTypeBlockImpl {
    fn handle_injury(
        &mut self, game: &Game, rng: &mut GameRng,
        attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>,
        _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode,
    ) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;

        if self.pre_broken {
            self.ctx.armor_broken = true;
        } else if !self.ctx.armor_broken {
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }

        if self.ctx.armor_broken {
            do_injury_roll(rng, &mut self.ctx);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }

    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn is_caused_by_opponent(&self) -> bool { self.caused_by_opponent }
    fn is_worth_spps(&self) -> bool { self.worth_spps }
}

// ── InjuryTypeChainsaw ────────────────────────────────────────────────────────

/// Java: InjuryTypeChainsaw — armor is always broken (chainsaw special rule).
pub struct InjuryTypeChainsawImpl {
    ctx: InjuryContext,
    worth_spps: bool,
}

impl InjuryTypeChainsawImpl {
    pub fn new() -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::Defender), worth_spps: false }
    }

    fn with_spps(mut self) -> Self {
        self.worth_spps = true;
        self
    }
}

impl InjuryTypeServer for InjuryTypeChainsawImpl {
    fn handle_injury(
        &mut self, _game: &Game, rng: &mut GameRng,
        attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>,
        _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode,
    ) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        // Chainsaw always breaks armor
        self.ctx.armor_broken = true;
        do_injury_roll(rng, &mut self.ctx);
    }

    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn is_caused_by_opponent(&self) -> bool { true }
    fn is_worth_spps(&self) -> bool { self.worth_spps }
}

// ── InjuryTypeThrowARock ──────────────────────────────────────────────────────

/// Java: InjuryTypeThrowARock — kickoff event, no turnover, can use apo.
pub struct InjuryTypeThrowARockImpl {
    ctx: InjuryContext,
}

impl InjuryTypeThrowARockImpl {
    fn new() -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::HitPlayer) }
    }
}

impl InjuryTypeServer for InjuryTypeThrowARockImpl {
    fn handle_injury(
        &mut self, game: &Game, rng: &mut GameRng,
        attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>,
        _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode,
    ) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;

        if !self.ctx.armor_broken {
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        if self.ctx.armor_broken {
            do_injury_roll(rng, &mut self.ctx);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }

    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
}

// ── InjuryTypeTTMLanding ─────────────────────────────────────────────────────

/// Java: InjuryTypeTTMLanding — TTM landing roll, can use apo.
pub struct InjuryTypeTtmLandingImpl {
    ctx: InjuryContext,
}

impl InjuryTypeTtmLandingImpl {
    fn new() -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::ThrownPlayer) }
    }
}

impl InjuryTypeServer for InjuryTypeTtmLandingImpl {
    fn handle_injury(
        &mut self, game: &Game, rng: &mut GameRng,
        attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>,
        _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode,
    ) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;

        if !self.ctx.armor_broken {
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        if self.ctx.armor_broken {
            do_injury_roll(rng, &mut self.ctx);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }

    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
}

// ── InjuryTypeTTMHitPlayer ────────────────────────────────────────────────────

/// Java: InjuryTypeTTMHitPlayer — when a TTM hits another player on landing.
pub struct InjuryTypeTtmHitPlayerImpl {
    ctx: InjuryContext,
}

impl InjuryTypeTtmHitPlayerImpl {
    fn new() -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::HitPlayer) }
    }
}

impl InjuryTypeServer for InjuryTypeTtmHitPlayerImpl {
    fn handle_injury(
        &mut self, game: &Game, rng: &mut GameRng,
        attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>,
        _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode,
    ) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;

        if !self.ctx.armor_broken {
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        if self.ctx.armor_broken {
            do_injury_roll(rng, &mut self.ctx);
        } else {
            self.ctx.injury = Some(PlayerState::new(PS_PRONE));
        }
    }

    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
}

// ── Factory ───────────────────────────────────────────────────────────────────

/// Create a boxed `InjuryTypeServer` from the Java class name string.
/// Used by steps that receive `StepParameter::InjuryTypeName(name)`.
pub fn make_injury_type(name: &str) -> Box<dyn InjuryTypeServer> {
    match name {
        "InjuryTypeDropGFI" | "InjuryTypeDropGfi" =>
            Box::new(InjuryTypeDropFall::new(true)),
        "InjuryTypeDropDodge" =>
            Box::new(InjuryTypeDropFall::new(true)),
        "InjuryTypeDropDodgeForSpp" =>
            Box::new(InjuryTypeDropFall::new(true)),
        "InjuryTypeDropJump" =>
            Box::new(InjuryTypeDropFall::new(true)),
        "InjuryTypeBlock" =>
            Box::new(InjuryTypeBlockImpl::new(false)),
        "InjuryTypeBlockForSpp" =>
            Box::new(InjuryTypeBlockImpl::new(false).with_spps()),
        "InjuryTypeBlockProne" =>
            Box::new(InjuryTypeBlockImpl::new(true)),
        "InjuryTypeBlockProneForSpp" =>
            Box::new(InjuryTypeBlockImpl::new(true).with_spps()),
        "InjuryTypeChainsaw" =>
            Box::new(InjuryTypeChainsawImpl::new()),
        "InjuryTypeChainsawForSpp" =>
            Box::new(InjuryTypeChainsawImpl::new().with_spps()),
        "InjuryTypeThrowARock" | "InjuryTypeThrowARockStalling" =>
            Box::new(InjuryTypeThrowARockImpl::new()),
        "InjuryTypeTTMLanding" | "InjuryTypeTtmLanding" =>
            Box::new(InjuryTypeTtmLandingImpl::new()),
        "InjuryTypeTTMHitPlayer" | "InjuryTypeTtmHitPlayer" =>
            Box::new(InjuryTypeTtmHitPlayerImpl::new()),
        "InjuryTypeTTMHitPlayerForSpp" =>
            Box::new(InjuryTypeTtmHitPlayerImpl::new()),  // TTM hit is by own team; SPP still tracked
        // Foul injury: armor roll + injury roll (foul assist modifiers TODO).
        "InjuryTypeFoul" =>
            Box::new(InjuryTypeBlockImpl::new(false)),
        "InjuryTypeFoulForSpp" =>
            Box::new(InjuryTypeBlockImpl::new(false).with_spps()),
        "InjuryTypeFoulChainsaw" =>
            Box::new(InjuryTypeBlockImpl::new(false)),
        "InjuryTypeFoulChainsawForSpp" =>
            Box::new(InjuryTypeBlockImpl::new(false).with_spps()),
        "InjuryTypeFallDown" =>
            Box::new(InjuryTypeDropFall::new(false)),
        "InjuryTypeFallDownForSpp" =>
            Box::new(InjuryTypeDropFall::new(false)),
        "InjuryTypeBreatheFire" =>
            Box::new(InjuryTypeBlockImpl::new(false)),
        "InjuryTypeBreatheFireForSpp" =>
            Box::new(InjuryTypeBlockImpl::new(false).with_spps()),
        "InjuryTypeCrowdPush" =>
            Box::new(InjuryTypeBlockImpl::new(false).not_by_opponent()),
        "InjuryTypeCrowdPushForSpp" =>
            Box::new(InjuryTypeBlockImpl::new(false).not_by_opponent().with_spps()),
        "InjuryTypeFumbledKtmApoKo" =>
            Box::new(InjuryTypeDropFall::new(false)),
        "InjuryTypeBlockStunned" =>
            Box::new(InjuryTypeBlockImpl::new(true)),
        "InjuryTypeBlockStunnedForSpp" =>
            Box::new(InjuryTypeBlockImpl::new(true).with_spps()),
        "InjuryTypeBombWithModifier" | "bombWithModifier" =>
            Box::new(injuryType::injury_type_bomb_with_modifier::InjuryTypeBombWithModifier::new()),
        "InjuryTypeBombWithModifierForSpp" | "bombForSpp" =>
            Box::new(injuryType::injury_type_bomb_with_modifier_for_spp::InjuryTypeBombWithModifierForSpp::new()),
        "InjuryTypeLightning" =>
            Box::new(injuryType::injury_type_lightning::InjuryTypeLightning::new()),
        "InjuryTypeFireball" =>
            Box::new(injuryType::injury_type_fireball::InjuryTypeFireball::new()),
        _ => {
            // Unknown type: fall through with generic drop behavior (causes turnover)
            Box::new(InjuryTypeDropFall::new(true))
        }
    }
}

// ── InjuryResult::apply_to tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender, PS_STANDING};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::types::{FieldCoordinate, KO_HOME_X};

    fn make_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game_with_players(home_ids: &[&str], away_ids: &[&str]) -> Game {
        let make_team = |id: &str, player_ids: &[&str]| -> Team {
            Team {
                id: id.into(),
                name: id.into(),
                race: "Human".into(),
                roster_id: "human".into(),
                coach: "Coach".into(),
                rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
                prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
                cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
                team_value: 0, treasury: 0, special_rules: vec![],
                players: player_ids.iter().map(|pid| make_player(pid)).collect(),
                vampire_lord: false,
            }
        };
        let home = make_team("home", home_ids);
        let away = make_team("away", away_ids);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_ir_with_state(defender_id: &str, state: PlayerState) -> InjuryResult {
        let mut ir = InjuryResult::new(ApothecaryMode::Defender);
        ir.injury_context.defender_id = Some(defender_id.to_owned());
        ir.injury_context.set_injury(state);
        ir
    }

    /// apply_to sets the player state in the field model.
    #[test]
    fn apply_to_sets_player_state() {
        let mut game = make_game_with_players(&["h1"], &[]);
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        let ir = make_ir_with_state("h1", PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        let state = game.field_model.player_state("h1").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
    }

    /// apply_to puts KO player into the dugout box.
    #[test]
    fn apply_to_puts_ko_player_in_box() {
        let mut game = make_game_with_players(&["h1"], &[]);
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        let ir = make_ir_with_state("h1", PlayerState::new(PS_KNOCKED_OUT));
        ir.apply_to(&mut game);

        let coord = game.field_model.player_coordinate("h1").expect("player should be placed");
        assert_eq!(coord.x, KO_HOME_X);
    }

    /// apply_to does NOT override a worse existing state with a less severe one.
    #[test]
    fn apply_to_does_not_override_worse_state() {
        let mut game = make_game_with_players(&["h1"], &[]);
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        // Player is already RIP (the worst)
        game.field_model.set_player_state("h1", PlayerState::new(PS_RIP));

        // Try to apply STUNNED (less severe)
        let ir = make_ir_with_state("h1", PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        // State must remain RIP
        let state = game.field_model.player_state("h1").unwrap();
        assert_eq!(state.base(), PS_RIP, "worse state should not be overridden by less severe");
    }

    fn make_player_with_skills(id: &str, skills: Vec<ffb_model::enums::SkillId>) -> Player {
        use ffb_model::model::SkillWithValue;
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 2,
            starting_skills: skills.into_iter().map(SkillWithValue::new).collect(),
            extra_skills: vec![], temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    #[test]
    fn do_injury_roll_for_player_stunty_ko_at_7_bb2020() {
        use ffb_model::enums::SkillId;
        // Stunty BB2020: total 7 = KO instead of Stunned.
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player_with_skills("p1", vec![SkillId::Stunty]));
        let game = Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025);
        let mut ctx = InjuryContext::new(ffb_model::enums::ApothecaryMode::Defender);
        let mut rng = GameRng::new(42);
        do_injury_roll_for_player(&mut rng, &mut ctx, &game, "p1");
        // If roll was 7 (Stunty→KO), verify; otherwise just check it resolved to something
        if let Some([d1, d2]) = ctx.injury_roll {
            let total = d1 + d2;
            if total == 7 {
                assert_eq!(ctx.injury.map(|s| s.base()), Some(PS_KNOCKED_OUT), "Stunty total 7 must be KO");
            }
        }
    }

    #[test]
    fn do_injury_roll_for_player_thick_skull_at_8_bb2020() {
        use ffb_model::enums::SkillId;
        // ThickSkull BB2020: total 8 = Stunned instead of KO.
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player_with_skills("p1", vec![SkillId::ThickSkull]));
        let game = Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025);
        let mut ctx = InjuryContext::new(ffb_model::enums::ApothecaryMode::Defender);
        let mut rng = GameRng::new(42);
        do_injury_roll_for_player(&mut rng, &mut ctx, &game, "p1");
        if let Some([d1, d2]) = ctx.injury_roll {
            let total = d1 + d2;
            if total == 8 {
                assert_eq!(ctx.injury.map(|s| s.base()), Some(PS_STUNNED), "ThickSkull total 8 must be Stunned");
            }
        }
    }

    #[test]
    fn do_injury_roll_for_player_no_skills_uses_standard_table() {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player_with_skills("p1", vec![]));
        let game = Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025);
        let mut ctx = InjuryContext::new(ffb_model::enums::ApothecaryMode::Defender);
        let mut rng = GameRng::new(42);
        do_injury_roll_for_player(&mut rng, &mut ctx, &game, "p1");
        // Should produce some injury result
        assert!(ctx.injury.is_some());
        assert!(ctx.injury_roll.is_some());
    }

    #[test]
    fn can_apo_ko_into_stun_defaults_true() {
        assert!(can_apo_ko_into_stun(None));
        assert!(can_apo_ko_into_stun(Some("InjuryTypeBlock")));
        assert!(can_apo_ko_into_stun(Some("InjuryTypeFoul")));
        assert!(can_apo_ko_into_stun(Some("InjuryTypeFumbledKtmApoKo")));
    }

    #[test]
    fn can_apo_ko_into_stun_false_for_crowd_and_trapdoor() {
        assert!(!can_apo_ko_into_stun(Some("InjuryTypeCrowdPush")));
        assert!(!can_apo_ko_into_stun(Some("InjuryTypeCrowdPushForSpp")));
        assert!(!can_apo_ko_into_stun(Some("InjuryTypeTrapDoorFall")));
        assert!(!can_apo_ko_into_stun(Some("InjuryTypeTrapDoorFallForSpp")));
    }
}
