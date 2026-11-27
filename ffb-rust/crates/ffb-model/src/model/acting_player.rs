use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::enums::{PlayerAction, PlayerState, SkillId};
use crate::model::player::PlayerId;

/// Tracks the currently-acting player during a turn.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActingPlayer {
    pub player_id: Option<PlayerId>,
    pub player_action: Option<PlayerAction>,
    /// For Block/Blitz: the target defender chosen at activation time.
    pub defender_id: Option<PlayerId>,
    pub current_move: i32,
    pub goes_for_it: bool,
    pub standing_up: bool,
    /// Rust-only bridge flag (no Java field): true once this activation POPPED a move square.
    /// Java gets the same fact from step ORDER — its phase-2 deselect runs before JumpUp/StandUp.
    pub took_square: bool,
    /// Java: ActingPlayer.fOldPlayerState — the player's PlayerState captured at the moment of
    /// activation (UtilActingPlayer.changeActingPlayer), STICKY (set once per activation, reset on
    /// player change). TakeRootBehaviour reads `getOldPlayerState().getBase() == STANDING` to decide
    /// whether a Treeman started the activation standing (only then does it roll Take Root).
    pub old_player_state: Option<PlayerState>,
    pub jumping: bool,
    /// Java: `jumpsWithoutModifiers` -- set by a Bounding Leap declaration
    /// (`CLIENT_USE_SKILL` with `canIgnoreJumpModifiers`). Write it through
    /// [`Self::set_jumps_without_modifiers`] / [`Self::set_jumping`], which carry Java's coupling:
    /// setting it true also sets `jumping`, clearing `jumping` also clears it.
    #[serde(default)]
    pub jumps_without_modifiers: bool,
    /// Java: `hasJumped` -- set by the bb2020 StepJump after a leap (success or failure); read by
    /// the bb2020 `JumpMechanic.canStillJump`. Not part of Java's `toJsonValue`.
    #[serde(default)]
    pub has_jumped: bool,
    pub has_acted: bool,
    pub has_fouled: bool,
    pub held_in_place: bool,
    /// For Blitz/StandUpBlitz activations: true once the block step has been executed.
    pub blitz_blocked: bool,
    /// SlashingNails: player temporarily gains Claws for this blitz activation.
    pub temporary_claws: bool,
    /// FrenziedRush: player temporarily gains Frenzy for this blitz activation.
    pub temporary_frenzy: bool,
    /// Incorporeal: player ignores tackle zones when moving this activation (no dodge rolls).
    pub ignore_tackle_zones: bool,
    /// FuryOfTheBloodGod: extra block actions remaining (0 = not active).
    pub fury_of_blood_god_blocks: u8,
    /// Java: fHasMoved — true once the player has taken at least one move step.
    pub has_moved: bool,
    /// Java: fHasBlocked — true once the player has executed a block (StepBlockStatistics).
    pub has_blocked: bool,
    /// Java: fSufferingBloodLust — Vampire player must feed this turn or go frenzy.
    pub suffering_blood_lust: bool,
    /// Rust-only bridge flag (no Java field): a failed Blood Lust roll owes ONE move-stack discard.
    ///
    /// Java gets the same fact from step ORDER. Its client delivers the activation's first move
    /// path at `StepInitSelecting` phase 2, BEFORE the activation's `StepBloodLust` runs, so
    /// `BloodLustBehaviour.handleExecuteStepHook`'s failure branch can throw that path away with
    /// `publishParameter(new StepParameter(MOVE_STACK, null))` + `GOTO_LABEL(goToLabelOnFailure)`.
    /// Rust asks for the path one step LATER (the empty-stack prompt in `StepInitMoving`), so when
    /// the roll fails there is no stack to discard yet; this flag defers the discard to the first
    /// path that arrives. Reset with the rest of the per-activation state in `set_player`.
    pub blood_lust_discards_move_stack: bool,
    /// Java: forgone — player chose to forgo their action (e.g. held by Take Root or flagged as stalling).
    pub forgone: bool,
    /// Java: fStrength — effective strength at block time (with modifiers; 0 = not set).
    pub strength: i32,
    /// Java: fellFromRush — true when a Ball & Chain player fell while going for it.
    pub fell_from_rush: bool,
    /// Java: hasTriggeredEffect — true once an out-of-activation effect (e.g. target-selection
    /// commitment) has been triggered for this player's turn.
    pub has_triggered_effect: bool,
    /// Java: fDodging — true while the player is in the middle of a dodge roll.
    pub dodging: bool,
    /// Java: fHasFed — Vampire has successfully fed this turn (BloodLust satisfied).
    pub has_fed: bool,
    /// Java: fHasPassed — true once the player has made a pass/hand-off this turn.
    pub has_passed: bool,
    /// Java: fSufferingAnimosity — true when the player failed an Animosity check
    /// and must re-select a passing target.
    pub suffering_animosity: bool,
    /// Java: mustCompleteAction — true once the player must complete their current
    /// action (e.g. `StepEndBomb` forces a second Ninja bomb throw) rather than being
    /// allowed to voluntarily end the turn.
    pub must_complete_action: bool,
    /// Java: fumblerooskiePending — true once CLIENT_USE_FUMBLEROOSKIE has been accepted
    /// (player action allows it and the player has the ball); cleared once the ball stops
    /// moving again. Consumed by `StepResetFumblerooskie`.
    pub fumblerooskie_pending: bool,
    /// Java: skillsGrantedBy — for each skill granted this turn (e.g. Wisdom of the White
    /// Dwarf), the list of player IDs that received it (keyed by skill instead of Java's
    /// `skill.getName()` since Rust identifies skills by `SkillId`).
    pub skills_granted_by: HashMap<SkillId, Vec<PlayerId>>,
    /// Java: `ActingPlayer.fUsedSkills` — per-activation skill-use tracking, CLEARED by
    /// `setPlayerId` when a different player is activated. Holds skills whose
    /// `SkillUsageType` is not "track outside activation" (e.g. Bone Head, Really Stupid):
    /// they may fire once per activation and reset for the next. Skills tracked across the
    /// whole turn live on `Player.used_skills` instead.
    pub used_skills: HashSet<SkillId>,
}

impl ActingPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Java `ActingPlayer.hasActed()`:
    ///   `hasMoved() || hasFouled() || hasBlocked() || hasPassed() || hasTriggeredEffect
    ///    || !fUsedSkills.isEmpty() || isForgone()`
    /// Java computes this on the fly from the sub-flags; Rust also carries a stored `has_acted`
    /// flag that some steps (e.g. Hypnotic Gaze) set directly to represent `hasTriggeredEffect`,
    /// so OR it in as well. Callers mirroring Java's `hasActed()` (e.g. changeActingPlayer's
    /// MOVING-state restore) MUST use this, not the bare stored field — a blitzer that moved and
    /// blocked has `has_moved`/`has_blocked` set but the stored `has_acted` stays false, so the
    /// bare field wrongly reported "did not act" and dropped a saved (Steady Footing) attacker
    /// back to PRONE instead of STANDING (high_elf seed 31 i=14).
    pub fn acted(&self) -> bool {
        self.has_moved
            || self.has_fouled
            || self.has_blocked
            || self.has_passed
            || self.has_triggered_effect
            || !self.used_skills.is_empty()
            || self.forgone
            || self.has_acted
    }

    /// Java `ActingPlayer.setPlayer(Player)` → `setPlayerId(id)` followed by `setPlayerAction(action)`.
    /// `setPlayerId` returns early when the id is unchanged (`StringTool.isEqual`), so the per-activation
    /// field reset only fires when a *different* player is set; the action is always (re)assigned.
    /// Mirroring Java's early-return is essential: e.g. Move→Block on the same blitzer must NOT wipe
    /// `has_moved`, and a fresh activation must reset `has_moved` so StepStandUp isn't skipped by a
    /// stale flag left over from the player's previous turn.
    pub fn set_player(&mut self, id: PlayerId, action: PlayerAction) {
        let same_id = self.player_id.as_ref() == Some(&id);
        if !same_id {
            // Java setPlayerId field reset (only the fields modelled in Rust).
            self.player_id = Some(id);
            self.defender_id = None;
            self.current_move = 0;
            self.goes_for_it = false;
            self.dodging = false;
            self.has_blocked = false;
            self.has_fouled = false;
            self.has_passed = false;
            self.has_moved = false;
            self.has_fed = false;
            self.has_acted = false;
            self.standing_up = false;
            self.took_square = false;
            // Java UtilActingPlayer.changeActingPlayer resets oldPlayerState on a genuine player
            // change; change_player_action re-captures it (sticky) right after this.
            self.old_player_state = None;
            self.set_jumping(false);
            self.has_jumped = false;
            self.suffering_blood_lust = false;
            self.blood_lust_discards_move_stack = false;
            self.suffering_animosity = false;
            self.fumblerooskie_pending = false;
            self.held_in_place = false;
            self.must_complete_action = false;
            self.fell_from_rush = false;
            self.has_triggered_effect = false;
            self.forgone = false;
            // Java setPlayerId: fUsedSkills.clear() — per-activation skill-use resets so
            // negatraits like Bone Head roll fresh for each new activation.
            self.used_skills.clear();
            // Java setPlayerId: skillsGrantedBy.clear() — the granted-skill ledger is
            // per-activation; UtilActingPlayer's deselect cleanup reads it BEFORE this reset.
            self.skills_granted_by.clear();
        }
        // Java setPlayerAction: always (re)assign the action.
        self.player_action = Some(action);
    }

    /// Java `ActingPlayer.setPlayerId(null)`: a no-op when no player is set; otherwise the same
    /// per-activation reset as a player change, plus `fPlayerAction = null` and `setStrength(0)`
    /// (no player). The client sees the cleared acting player (e.g. the pass-block window).
    pub fn clear_player_id(&mut self) {
        if self.player_id.is_none() {
            return;
        }
        let action = self.player_action.unwrap_or(PlayerAction::Move);
        self.set_player(String::new(), action);
        self.player_id = None;
        self.player_action = None;
        self.strength = 0;
    }

    /// Java: `setJumping(boolean)` -- clearing the flag also clears `jumpsWithoutModifiers`.
    pub fn set_jumping(&mut self, jumping: bool) {
        if jumping == self.jumping {
            return;
        }
        self.jumping = jumping;
        if !self.jumping {
            self.set_jumps_without_modifiers(false);
        }
    }

    /// Java: `setJumpsWithoutModifiers(boolean)` -- setting it also sets `jumping`.
    pub fn set_jumps_without_modifiers(&mut self, jumps_without_modifiers: bool) {
        if self.jumps_without_modifiers == jumps_without_modifiers {
            return;
        }
        self.jumps_without_modifiers = jumps_without_modifiers;
        if self.jumps_without_modifiers {
            self.set_jumping(true);
        }
    }

    pub fn is_active(&self) -> bool {
        self.player_id.is_some()
    }

    /// Java: `isMustCompleteAction()`.
    pub fn is_must_complete_action(&self) -> bool {
        self.must_complete_action
    }

    /// Java: `setMustCompleteAction(boolean)`.
    pub fn set_must_complete_action(&mut self, must_complete_action: bool) {
        self.must_complete_action = must_complete_action;
    }

    /// Java: `isFumblerooskiePending()`.
    pub fn is_fumblerooskie_pending(&self) -> bool {
        self.fumblerooskie_pending
    }

    /// Java: `setFumblerooskiePending(boolean)`.
    pub fn set_fumblerooskie_pending(&mut self, fumblerooskie_pending: bool) {
        self.fumblerooskie_pending = fumblerooskie_pending;
    }

    /// Java: `addGrantedSkill(Skill skill, Player<?> player)` — records that `player_id`
    /// (if any) was granted `skill_id` this turn.
    pub fn add_granted_skill(&mut self, skill_id: SkillId, player_id: Option<PlayerId>) {
        let players = self.skills_granted_by.entry(skill_id).or_default();
        if let Some(id) = player_id {
            players.push(id);
        }
    }

    /// Java: `ActingPlayer.toJsonValue()`. `rules` = the game's rules (`fGame.getRules()`), whose
    /// `SkillFactory` instance of each skill supplies `getName()`. `Err` for a skill the ruleset
    /// has no class for (Java could not hold such a skill object).
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json::{player_state_or_null, string_or_null};
        use serde_json::Value;
        let skill_name = |id: SkillId| {
            SkillFactory::name_in(rules, id).ok_or_else(|| format!("ActingPlayer: {id:?} is not a {} skill", rules.name()))
        };
        let mut o = serde_json::Map::new();
        o.insert("playerId".into(), string_or_null(self.player_id.as_deref()));
        o.insert("currentMove".into(), Value::from(self.current_move));
        o.insert("goingForIt".into(), Value::Bool(self.goes_for_it));
        o.insert("hasBlocked".into(), Value::Bool(self.has_blocked));
        o.insert("hasFed".into(), Value::Bool(self.has_fed));
        o.insert("hasFouled".into(), Value::Bool(self.has_fouled));
        o.insert("hasMoved".into(), Value::Bool(self.has_moved));
        o.insert("hasPassed".into(), Value::Bool(self.has_passed));
        o.insert("playerAction".into(), string_or_null(self.player_action.map(PlayerAction::name)));
        o.insert("standingUp".into(), Value::Bool(self.standing_up));
        o.insert("sufferingAnimosity".into(), Value::Bool(self.suffering_animosity));
        o.insert("sufferingBloodlust".into(), Value::Bool(self.suffering_blood_lust));
        o.insert("fumblerooskiePending".into(), Value::Bool(self.fumblerooskie_pending));
        o.insert("jumpsWithoutModifiers".into(), Value::Bool(self.jumps_without_modifiers));
        // Java iterates a HashSet: the order is not model state (canonical form sorts this array).
        let mut used: Vec<Value> = Vec::with_capacity(self.used_skills.len());
        for id in &self.used_skills {
            used.push(Value::String(skill_name(*id)?.to_string()));
        }
        o.insert("usedSkills".into(), Value::Array(used));
        // JsonStringListMapOption.addTo: one JsonStringArrayOption per entry.
        let mut granted = serde_json::Map::new();
        for (id, players) in &self.skills_granted_by {
            let list = players.iter().map(|p| Value::String(p.clone())).collect();
            granted.insert(skill_name(*id)?.to_string(), Value::Array(list));
        }
        o.insert("skillsGrantedBy".into(), Value::Object(granted));
        o.insert(
            "playerStateOld".into(),
            player_state_or_null(self.old_player_state.map(|s| s.0 as i32)),
        );
        o.insert("heldInPlace".into(), Value::Bool(self.held_in_place));
        o.insert("mustCompleteAction".into(), Value::Bool(self.must_complete_action));
        o.insert("fellFromRush".into(), Value::Bool(self.fell_from_rush));
        o.insert("hasTriggeredEffect".into(), Value::Bool(self.has_triggered_effect));
        Ok(Value::Object(o))
    }

    /// Java: `ActingPlayer.initFrom(IFactorySource, JsonValue)`, 1:1 (fields Java only sets when
    /// the key `isDefinedIn` keep their value otherwise). `Err` where Java throws (a missing
    /// primitive boolean, a wrong JSON type) and where the Java result is not representable here:
    /// an unknown skill name (Java adds `null` to the set / keeps a raw string key) or a
    /// `skillsGrantedBy` entry whose list is absent or empty (Java stores `null`).
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as j;
        let o = j::to_json_object(json)?;
        let skill = |name: &str| {
            SkillFactory::for_name_in(rules, name)
                .ok_or_else(|| format!("ActingPlayer.initFrom: skill {name:?} unknown to {} (Java null)", rules.name()))
        };
        self.player_id = j::get_string(o, "playerId")?;
        self.current_move = j::get_int(o, "currentMove")?;
        self.goes_for_it = j::get_boolean(o, "goingForIt")?;
        self.has_blocked = j::get_boolean(o, "hasBlocked")?;
        self.has_fed = j::get_boolean(o, "hasFed")?;
        self.has_fouled = j::get_boolean(o, "hasFouled")?;
        self.has_moved = j::get_boolean(o, "hasMoved")?;
        self.has_passed = j::get_boolean(o, "hasPassed")?;
        self.player_action = j::get_enum_name(o, "playerAction")?.and_then(PlayerAction::from_name);
        self.standing_up = j::get_boolean(o, "standingUp")?;
        self.suffering_animosity = j::get_boolean(o, "sufferingAnimosity")?;
        self.suffering_blood_lust = j::get_boolean(o, "sufferingBloodlust")?;
        self.fumblerooskie_pending = j::get_boolean_object(o, "fumblerooskiePending")? == Some(true);
        if j::is_defined_in(o, "jumpsWithoutModifiers") {
            self.jumps_without_modifiers = j::get_boolean(o, "jumpsWithoutModifiers")?;
        }
        self.used_skills.clear();
        if let Some(used) = j::get_array(o, "usedSkills")? {
            for element in used {
                let name = element.as_str().ok_or_else(|| format!("usedSkills: element {element}"))?;
                self.used_skills.insert(skill(name)?);
            }
        }
        if j::is_defined_in(o, "skillsGrantedBy") {
            self.skills_granted_by.clear();
            // JsonStringListMapOption.getFrom: a non-object value yields an empty map.
            if let Some(map) = o.get("skillsGrantedBy").and_then(serde_json::Value::as_object) {
                for (name, list) in map {
                    let players = list
                        .as_array()
                        .filter(|a| !a.is_empty())
                        .ok_or_else(|| format!("skillsGrantedBy.{name}: {list} (Java stores null)"))?;
                    let players = players
                        .iter()
                        .map(|p| p.as_str().map(str::to_string).ok_or_else(|| format!("skillsGrantedBy.{name}: element {p}")))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.skills_granted_by.insert(skill(name)?, players);
                }
            }
        }
        self.old_player_state = j::get_player_state_id(o, "playerStateOld")?
            .map(|id| u32::try_from(id).map(PlayerState).map_err(|_| format!("playerStateOld {id}")))
            .transpose()?;
        if j::is_defined_in(o, "heldInPlace") {
            self.held_in_place = j::get_boolean(o, "heldInPlace")?;
        }
        if j::is_defined_in(o, "mustCompleteAction") {
            self.must_complete_action = j::get_boolean(o, "mustCompleteAction")?;
        }
        if j::is_defined_in(o, "fellFromRush") {
            self.fell_from_rush = j::get_boolean(o, "fellFromRush")?;
        }
        if j::is_defined_in(o, "hasTriggeredEffect") {
            self.has_triggered_effect = j::get_boolean(o, "hasTriggeredEffect")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::PlayerAction;

    #[test]
    fn set_and_clear() {
        let mut ap = ActingPlayer::new();
        ap.set_player("p1".into(), PlayerAction::Move);
        assert!(ap.is_active());
        ap.clear();
        assert!(!ap.is_active());
    }

    #[test]
    fn serde_round_trip() {
        let mut ap = ActingPlayer::new();
        ap.set_player("p1".into(), PlayerAction::Blitz);
        let json = serde_json::to_string(&ap).unwrap();
        let back: ActingPlayer = serde_json::from_str(&json).unwrap();
        assert_eq!(ap.player_id, back.player_id);
    }

    #[test]
    fn set_player_resets_movement_counters() {
        let mut ap = ActingPlayer::new();
        ap.current_move = 5;
        ap.goes_for_it = true;
        ap.set_player("p1".into(), PlayerAction::Move);
        assert_eq!(ap.current_move, 0);
        assert!(!ap.goes_for_it);
    }

    #[test]
    fn set_player_resets_must_complete_action() {
        let mut ap = ActingPlayer::new();
        ap.set_must_complete_action(true);
        ap.set_player("p1".into(), PlayerAction::Move);
        assert!(!ap.is_must_complete_action());
    }

    #[test]
    fn set_player_new_id_resets_has_moved() {
        // Java setPlayerId resets fHasMoved on a fresh activation. A stale has_moved would make
        // StepStandUp's `isStandingUp() && !hasMoved()` guard skip the free stand-up.
        let mut ap = ActingPlayer::new();
        ap.set_player("p1".into(), PlayerAction::Move);
        ap.has_moved = true;
        ap.has_blocked = true;
        ap.dodging = true;
        // Activate a different player: everything resets.
        ap.set_player("p2".into(), PlayerAction::Move);
        assert!(!ap.has_moved);
        assert!(!ap.has_blocked);
        assert!(!ap.dodging);
    }

    #[test]
    fn set_player_same_id_preserves_has_moved_updates_action() {
        // Java setPlayerId early-returns on an unchanged id (StringTool.isEqual), so per-activation
        // flags survive; only the action is (re)assigned. E.g. Move→Block on the same blitzer.
        let mut ap = ActingPlayer::new();
        ap.set_player("p1".into(), PlayerAction::Move);
        ap.has_moved = true;
        ap.set_player("p1".into(), PlayerAction::Block);
        assert!(ap.has_moved, "same-id set_player must not wipe has_moved");
        assert_eq!(ap.player_action, Some(PlayerAction::Block), "action must update");
    }

    #[test]
    fn add_granted_skill_tracks_players() {
        use crate::enums::SkillId;
        let mut ap = ActingPlayer::new();
        ap.add_granted_skill(SkillId::WisdomOfTheWhiteDwarf, Some("p1".into()));
        ap.add_granted_skill(SkillId::WisdomOfTheWhiteDwarf, Some("p2".into()));
        assert_eq!(
            ap.skills_granted_by.get(&SkillId::WisdomOfTheWhiteDwarf).map(|v| v.as_slice()),
            Some(["p1".to_string(), "p2".to_string()].as_slice())
        );
    }

    #[test]
    fn init_from_to_json_value_round_trip_java_shape() {
        use crate::enums::{Rules, SkillId};
        // The shape of ActingPlayer.toJsonValue() in a real replay serverGameState (1965890-like),
        // with a used skill and a granted skill added.
        let java = serde_json::json!({
            "playerId": "18160488", "currentMove": 4, "goingForIt": true, "hasBlocked": false,
            "hasFed": false, "hasFouled": false, "hasMoved": true, "hasPassed": false,
            "playerAction": "blitzMove", "standingUp": false, "sufferingAnimosity": false,
            "sufferingBloodlust": false, "fumblerooskiePending": false, "jumpsWithoutModifiers": false,
            "usedSkills": ["Bone Head"], "skillsGrantedBy": { "Wisdom of the White Dwarf": ["p1", "p2"] },
            "playerStateOld": 257, "heldInPlace": false, "mustCompleteAction": false,
            "fellFromRush": false, "hasTriggeredEffect": false
        });
        let mut ap = ActingPlayer::new();
        ap.init_from(Rules::Bb2020, &java).unwrap();
        assert_eq!(ap.player_action, Some(PlayerAction::BlitzMove));
        assert!(ap.used_skills.contains(&SkillId::BoneHead));
        assert_eq!(ap.old_player_state, Some(PlayerState(257)));
        assert_eq!(ap.to_json_value(Rules::Bb2020).unwrap(), java);
        // bb2016 spells Bone Head "Bone-Head": Java's forName returns null for the bb2020 spelling.
        assert!(ActingPlayer::new().init_from(Rules::Bb2016, &java).is_err());
    }

    #[test]
    fn init_from_follows_java_option_rules() {
        use crate::enums::Rules;
        let mut java = serde_json::json!({
            "playerId": null, "currentMove": 0, "goingForIt": false, "hasBlocked": false,
            "hasFed": false, "hasFouled": false, "hasMoved": false, "hasPassed": false,
            "playerAction": "noSuchAction", "standingUp": false, "sufferingAnimosity": false,
            "sufferingBloodlust": false, "usedSkills": [], "playerStateOld": null
        });
        let mut ap = ActingPlayer::new();
        ap.held_in_place = true; // not defined in the JSON -> Java keeps the old value
        ap.init_from(Rules::Bb2025, &java).unwrap();
        assert_eq!(ap.player_action, None); // PlayerActionFactory.forName -> null
        assert!(ap.held_in_place);
        assert!(!ap.fumblerooskie_pending); // absent Boolean -> false
        java.as_object_mut().unwrap().remove("hasFed"); // primitive from a null Boolean -> NPE
        assert!(ActingPlayer::new().init_from(Rules::Bb2025, &java).is_err());
    }

    #[test]
    fn add_granted_skill_with_no_player_creates_empty_entry() {
        use crate::enums::SkillId;
        let mut ap = ActingPlayer::new();
        ap.add_granted_skill(SkillId::WisdomOfTheWhiteDwarf, None);
        assert!(ap.skills_granted_by.contains_key(&SkillId::WisdomOfTheWhiteDwarf));
        assert!(ap.skills_granted_by.get(&SkillId::WisdomOfTheWhiteDwarf).unwrap().is_empty());
    }
}
