/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.SelectPlayerPrayerHandler`
/// and of the `DialogPrayerHandler.initEffect` it inherits.
///
/// ```java
/// // DialogPrayerHandler
/// public final boolean initEffect(GameState gameState, Team prayingTeam) {
///     Set<Skill> skillsFromEnhancement = handledPrayer().enhancements(mechanic).getSkills()...;
///     List<Player<?>> players = selector().eligiblePlayers(prayingTeam, gameState.getGame(), skillsFromEnhancement);
///     if (players.isEmpty()) {
///         reports.add(new ReportPrayerWasted(this.handledPrayer().getName()));
///         return true;                       // nothing to choose → the step advances
///     }
///     createDialog(players, gameState, prayingTeam);
///     return handled(gameState.getGame());   // SelectPlayerPrayerHandler: always false
/// }
///
/// // SelectPlayerPrayerHandler
/// protected void createDialog(List<Player<?>> players, GameState gameState, Team prayingTeam) {
///     UtilServerDialog.showDialog(gameState,
///         new DialogPlayerChoiceParameter(prayingTeam.getId(), choiceMode(), playerIds, ...));
/// }
/// public void applySelection(Game game, PrayerDialogSelection selection) {
///     Player<?> player = game.getPlayerById(selection.getPlayerId());
///     game.getFieldModel().addPrayerEnhancements(player, handledPrayer());
///     reports.add(new ReportPlayerEvent(player.getId(), handledPrayer().eventMessage()));
/// }
/// ```
///
/// The coach CHOOSES the player here — there is no shuffle. Routing these three prayers
/// (Iron Man, Knuckle Dusters, Blessed Statue of Nuffle) through
/// `RandomSelectionPrayerHandler` instead both picked a different player and drew from
/// `java.util.Collections`' shared stream that Java never touches on this path.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use crate::inducements::mixed::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::prayer_player_effect::apply_prayer_player_effect;

/// Java: `DialogPrayerHandler.initEffect` — the eligible players the dialog offers.
///
/// An EMPTY result is Java's `players.isEmpty()` branch: report the prayer wasted and let the
/// step advance. A non-empty result means a dialog is shown and the step waits.
pub fn eligible_players_for_dialog(
    game: &Game,
    team_id: &str,
    added_skills: &[SkillId],
    selector: &dyn PlayerSelector,
) -> Vec<String> {
    selector.eligible_players(game, team_id, added_skills)
}

/// Java: `SelectPlayerPrayerHandler.applySelection(Game, PrayerDialogSelection)`.
pub fn apply_selection_select_player(game: &mut Game, player_id: &str, prayer_name: &str) {
    if player_id.is_empty() || game.player(player_id).is_none() {
        // Java would NPE on `player.getId()`; the harness never sends an empty selection for
        // these modes (they are declared non-declinable), so this only guards test callers.
        return;
    }
    game.field_model.add_prayer_enhancement(player_id, prayer_name);
    apply_prayer_player_effect(game, player_id, prayer_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, Rules, PS_RESERVE, PS_STANDING, TurnMode};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;
    use crate::inducements::bb2020::prayers::player_selector::PlayerSelector as BB2020Selector;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, id: &str, nr: i32, on_pitch: bool) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr, ..Default::default()
        });
        if on_pitch {
            game.field_model.set_player_coordinate(id, FieldCoordinate::new(5, 5));
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        } else {
            game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
        }
    }

    /// The dialog list must be the eligibility filter ONLY — no shuffle, no dice, and no draw
    /// from the Collections stream (which is what the random-selection path used).
    #[test]
    fn eligible_players_lists_every_on_pitch_player_without_randomness() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        add_player(&mut game, "h1", 1, true);
        add_player(&mut game, "h2", 2, true);
        add_player(&mut game, "h3", 3, false); // reserve → not eligible outside START_GAME
        let ids = eligible_players_for_dialog(&game, "home", &[], &BB2020Selector::new());
        assert_eq!(ids, vec!["h1".to_string(), "h2".to_string()]);
    }

    /// Java's `players.isEmpty()` arm: the prayer is wasted and the step advances.
    #[test]
    fn eligible_players_is_empty_when_nobody_qualifies() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        add_player(&mut game, "h1", 1, false);
        assert!(eligible_players_for_dialog(&game, "home", &[], &BB2020Selector::new()).is_empty());
    }

    #[test]
    fn apply_selection_marks_the_chosen_player() {
        let mut game = make_game();
        add_player(&mut game, "h1", 1, true);
        apply_selection_select_player(&mut game, "h1", "KNUCKLE_DUSTERS");
        assert!(game.field_model.has_prayer_enhancement("h1", "KNUCKLE_DUSTERS"));
    }

    #[test]
    fn apply_selection_with_no_player_is_a_no_op() {
        let mut game = make_game();
        apply_selection_select_player(&mut game, "", "KNUCKLE_DUSTERS");
        apply_selection_select_player(&mut game, "nobody", "KNUCKLE_DUSTERS");
        assert!(!game.field_model.has_prayer_enhancement("nobody", "KNUCKLE_DUSTERS"));
    }
}
