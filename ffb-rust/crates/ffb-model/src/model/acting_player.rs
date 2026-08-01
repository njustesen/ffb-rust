use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::{PlayerAction, SkillId};
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
    pub jumping: bool,
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
}

impl ActingPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
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
            self.jumping = false;
            self.suffering_blood_lust = false;
            self.suffering_animosity = false;
            self.fumblerooskie_pending = false;
            self.held_in_place = false;
            self.must_complete_action = false;
            self.fell_from_rush = false;
            self.has_triggered_effect = false;
            self.forgone = false;
        }
        // Java setPlayerAction: always (re)assign the action.
        self.player_action = Some(action);
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
    fn add_granted_skill_with_no_player_creates_empty_entry() {
        use crate::enums::SkillId;
        let mut ap = ActingPlayer::new();
        ap.add_granted_skill(SkillId::WisdomOfTheWhiteDwarf, None);
        assert!(ap.skills_granted_by.contains_key(&SkillId::WisdomOfTheWhiteDwarf));
        assert!(ap.skills_granted_by.get(&SkillId::WisdomOfTheWhiteDwarf).unwrap().is_empty());
    }
}
