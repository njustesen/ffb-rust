/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.multiblock.AbstractStepMultiple`.
///
/// Abstract base for multiple-target steps that support a LORD_OF_CHAOS single-use team
/// re-roll.  Subclasses implement `state()` to expose their `SingleReRollUseState`.
///
/// Rust note: because Rust has no runtime inheritance, this is a plain trait object that
/// subclasses _may_ embed and delegate to.  The `HasIdForSingleUseReRoll` Java interface
/// maps to the `id_for_single_use_re_roll()` method here.
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;

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

/// Java: `AbstractStepMultiple.reRollSourceSuccessfully(ReRollSource)`.
///
/// When `source` is "LORD_OF_CHAOS", searches the acting team for players with the
/// `grantsSingleUseTeamRerollWhenOnPitch` property (i.e. Lord of Chaos players with an
/// unused skill).  If > 1, shows a player-choice dialog (not yet ported → returns `false`
/// to signal waiting-for-command).  If exactly 1, sets the id automatically and returns
/// `true`.  If 0, returns `true` (no dialog needed).
/// For all other re-roll sources always returns `true`.
pub fn re_roll_source_successfully(
    state: &mut SingleReRollUseState,
    source: &str,
    game: &Game,
) -> bool {
    if source == "LORD_OF_CHAOS" {
        state.set_re_roll_source(source);
        // Java: Arrays.stream(game.getActingTeam().getPlayers())
        //         .filter(p -> UtilCards.hasUnusedSkillWithProperty(p, grantsSingleUseTeamRerollWhenOnPitch))
        //         .map(Player::getId)
        let lords: Vec<String> = game.active_team().players.iter()
            .filter(|p| p.has_unused_skill_with_property(NamedProperties::GRANTS_SINGLE_USE_TEAM_REROLL_WHEN_ON_PITCH))
            .map(|p| p.id.clone())
            .collect();
        match lords.len() {
            0 => true,
            1 => {
                state.set_id(lords[0].clone());
                true
            }
            _ => {
                // DEFERRED(dialog port): showDialog(DialogPlayerChoiceParameter / LORD_OF_CHAOS, lords)
                // Java shows a PlayerChoiceMode::LordOfChaos dialog; CLIENT_PLAYER_CHOICE then
                // calls apply_lord_of_chaos_command which sets state.id.
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

    fn make_game_with_lords(count: usize) -> ffb_model::model::game::Game {
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::{SkillId, SkillWithValue};
        let mut home = crate::step::framework::test_team("home", 0);
        for i in 0..count {
            let mut p = Player::default();
            p.id = format!("lord{}", i);
            p.starting_skills.push(SkillWithValue { skill_id: SkillId::LordOfChaos, value: None });
            home.players.push(p);
        }
        ffb_model::model::game::Game::new(
            home,
            crate::step::framework::test_team("away", 0),
            ffb_model::enums::Rules::Bb2025,
        )
    }

    #[test]
    fn re_roll_source_non_lord_always_succeeds() {
        let mut state = SingleReRollUseState::new();
        let game = ffb_model::model::game::Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            ffb_model::enums::Rules::Bb2025,
        );
        let result = re_roll_source_successfully(&mut state, "PRO", &game);
        assert!(result);
        assert_eq!(state.re_roll_source.as_deref(), Some("PRO"));
    }

    #[test]
    fn re_roll_source_lord_zero_returns_true() {
        let mut state = SingleReRollUseState::new();
        let game = make_game_with_lords(0);
        let result = re_roll_source_successfully(&mut state, "LORD_OF_CHAOS", &game);
        assert!(result);
    }

    #[test]
    fn re_roll_source_lord_single_sets_id() {
        let mut state = SingleReRollUseState::new();
        let game = make_game_with_lords(1);
        let result = re_roll_source_successfully(&mut state, "LORD_OF_CHAOS", &game);
        assert!(result);
        assert_eq!(state.id.as_deref(), Some("lord0"));
    }

    #[test]
    fn re_roll_source_lord_multiple_returns_false() {
        let mut state = SingleReRollUseState::new();
        let game = make_game_with_lords(2);
        let result = re_roll_source_successfully(&mut state, "LORD_OF_CHAOS", &game);
        assert!(!result);
    }
}
