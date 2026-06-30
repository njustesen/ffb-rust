/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.multiblock.AbstractStepMultiple`.
///
/// Abstract base for multiple-target steps that support a LORD_OF_CHAOS single-use team
/// re-roll.  Subclasses implement `state()` to expose their `SingleReRollUseState`.
///
/// Rust note: because Rust has no runtime inheritance, this is a plain trait object that
/// subclasses _may_ embed and delegate to.  The `HasIdForSingleUseReRoll` Java interface
/// maps to the `id_for_single_use_re_roll()` method here.
use ffb_model::model::game::Game;

/// Java: `AbstractStepMultiple` — shared re-roll state for multi-target block steps.
///
/// Subclasses carry a `SingleReRollUseState` and expose it so this base can populate it
/// when a LORD_OF_CHAOS command arrives.
///
/// Concrete re-roll state stored by the subclass:
///   - `id`: chosen Lord-of-Chaos player id
///   - `re_roll_source`: resolved re-roll source name
#[derive(Debug, Default, Clone)]
pub struct SingleReRollUseState {
    /// Java: `id` — the player ID selected as the Lord of Chaos (or empty if unused)
    pub id: Option<String>,
    /// Java: `reRollSource` — the re-roll source name (e.g. "LORD_OF_CHAOS")
    pub re_roll_source: Option<String>,
    /// Java: `reRollTarget` — target player ID that triggered this re-roll
    pub re_roll_target: Option<String>,
}

impl SingleReRollUseState {
    pub fn new() -> Self { Self::default() }

    /// Java: `setId`
    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    /// Java: `getId`
    pub fn get_id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Java: `setReRollSource`
    pub fn set_re_roll_source(&mut self, source: impl Into<String>) {
        self.re_roll_source = Some(source.into());
    }
}

/// Helper: returns `true` when the re-roll source can be resolved without showing a dialog
/// (i.e. exactly zero or one Lord-of-Chaos player is on pitch).
///
/// Java: `AbstractStepMultiple.reRollSourceSuccessfully(ReRollSource)` (simplified).
/// When `source` is "LORD_OF_CHAOS", searches the acting team for players with the
/// `grantsSingleUseTeamRerollWhenOnPitch` property.  If > 1, shows a dialog (not yet
/// ported → returns `false` stub).  If 1, sets the id.  Returns `true`.
pub fn re_roll_source_successfully(
    state: &mut SingleReRollUseState,
    source: &str,
    acting_lords: &[String],
) -> bool {
    if source == "LORD_OF_CHAOS" {
        state.set_re_roll_source(source);
        match acting_lords.len() {
            0 => true,
            1 => {
                state.set_id(acting_lords[0].clone());
                true
            }
            _ => {
                // TODO(dialog port): showDialog(DialogPlayerChoiceParameter / LORD_OF_CHAOS)
                false
            }
        }
    } else {
        state.set_re_roll_source(source);
        true
    }
}

/// Java: `AbstractStepMultiple` embedded base type for use by subclasses.
///
/// Subclasses should embed this and call `apply_lord_of_chaos_command` from their
/// `handle_command` before delegating to their own logic.
#[derive(Debug, Default)]
pub struct AbstractStepMultiple {
    pub state: SingleReRollUseState,
}

impl AbstractStepMultiple {
    pub fn new() -> Self { Self::default() }

    /// Java: `handleCommand` — processes `CLIENT_PLAYER_CHOICE(LORD_OF_CHAOS)`.
    /// Returns `true` if the command was consumed (should trigger `executeStep`).
    pub fn apply_lord_of_chaos_command(&mut self, _game: &mut Game, player_id: Option<&str>) -> bool {
        if let Some(id) = player_id {
            self.state.set_id(id);
            true
        } else {
            false
        }
    }

    /// Java: `idForSingleUseReRoll`
    pub fn id_for_single_use_re_roll(&self) -> Option<&str> {
        self.state.get_id()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_empty() {
        let b = AbstractStepMultiple::new();
        assert!(b.state.id.is_none());
        assert!(b.state.re_roll_source.is_none());
    }

    #[test]
    fn lord_of_chaos_command_sets_id() {
        let mut b = AbstractStepMultiple::new();
        let mut game = ffb_model::model::game::Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            ffb_model::enums::Rules::Bb2025,
        );
        let consumed = b.apply_lord_of_chaos_command(&mut game, Some("p1"));
        assert!(consumed);
        assert_eq!(b.id_for_single_use_re_roll(), Some("p1"));
    }

    #[test]
    fn none_player_not_consumed() {
        let mut b = AbstractStepMultiple::new();
        let mut game = ffb_model::model::game::Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            ffb_model::enums::Rules::Bb2025,
        );
        let consumed = b.apply_lord_of_chaos_command(&mut game, None);
        assert!(!consumed);
        assert!(b.id_for_single_use_re_roll().is_none());
    }

    #[test]
    fn re_roll_source_non_lord_always_succeeds() {
        let mut state = SingleReRollUseState::new();
        let result = re_roll_source_successfully(&mut state, "PRO", &[]);
        assert!(result);
        assert_eq!(state.re_roll_source.as_deref(), Some("PRO"));
    }

    #[test]
    fn re_roll_source_lord_single_sets_id() {
        let mut state = SingleReRollUseState::new();
        let lords = vec!["lord1".into()];
        let result = re_roll_source_successfully(&mut state, "LORD_OF_CHAOS", &lords);
        assert!(result);
        assert_eq!(state.id.as_deref(), Some("lord1"));
    }

    #[test]
    fn re_roll_source_lord_multiple_returns_false() {
        let mut state = SingleReRollUseState::new();
        let lords = vec!["l1".into(), "l2".into()];
        let result = re_roll_source_successfully(&mut state, "LORD_OF_CHAOS", &lords);
        assert!(!result);
    }
}
