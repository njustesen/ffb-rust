package com.fumbbl.ffb.util;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-model/src/util/util_player.rs tests.
 * Builds a live Game and places RosterPlayers on the FieldModel (the ffb-common analogue of the
 * Rust minimal_game()/add_player fixture). State ints match the Rust constants:
 * ACTIVE_STANDING = PS_STANDING(0x1) | _BIT_ACTIVE(0x100) = 0x101; ACTIVE_PRONE = 0x3 | 0x100 = 0x103.
 */
public class UtilPlayerTest {

	private static final int ACTIVE_STANDING = 0x101;
	private static final int ACTIVE_PRONE = 0x103;

	private Game game;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
		game.getTeamHome().setId("home");
		game.getTeamAway().setId("away");
	}

	private RosterPlayer addPlayer(boolean home, String id, FieldCoordinate coord, int state) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		(home ? game.getTeamHome() : game.getTeamAway()).addPlayer(p);
		game.getFieldModel().setPlayerCoordinate(p, coord);
		game.getFieldModel().setPlayerState(p, new PlayerState(state));
		return p;
	}

	// rust: find_other_team_home_player_returns_away
	@Test
	public void findOtherTeamHomePlayerReturnsAway() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		assertEquals("away", UtilPlayer.findOtherTeam(game, h1).getId());
	}

	// rust: find_other_team_away_player_returns_home
	@Test
	public void findOtherTeamAwayPlayerReturnsHome() {
		Player<?> a1 = addPlayer(false, "a1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		assertEquals("home", UtilPlayer.findOtherTeam(game, a1).getId());
	}

	// rust: find_adjacent_players_with_tacklezones_returns_standing_adjacent
	@Test
	public void findAdjacentPlayersWithTacklezonesReturnsStandingAdjacent() {
		addPlayer(true, "h1", new FieldCoordinate(6, 5), ACTIVE_STANDING);
		addPlayer(true, "h2", new FieldCoordinate(10, 10), ACTIVE_STANDING);
		Player<?>[] results = UtilPlayer.findAdjacentPlayersWithTacklezones(
			game, game.getTeamHome(), new FieldCoordinate(5, 5), false);
		assertEquals(1, results.length);
		assertEquals("h1", results[0].getId());
	}

	// rust: find_tacklezones_counts_opposing_adjacent_standing
	@Test
	public void findTacklezonesCountsOpposingAdjacentStanding() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(false, "a1", new FieldCoordinate(6, 5), ACTIVE_STANDING);
		addPlayer(false, "a2", new FieldCoordinate(5, 6), ACTIVE_STANDING);
		assertEquals(2, UtilPlayer.findTacklezones(game, h1));
	}

	// rust: find_tacklezones_zero_when_no_adjacent_opponents
	@Test
	public void findTacklezonesZeroWhenNoAdjacentOpponents() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(false, "a1", new FieldCoordinate(10, 10), ACTIVE_STANDING);
		assertEquals(0, UtilPlayer.findTacklezones(game, h1));
	}

	// rust: can_foul_true_when_adjacent_prone_opponent
	@Test
	public void canFoulTrueWhenAdjacentProneOpponent() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(false, "a1", new FieldCoordinate(6, 5), ACTIVE_PRONE);
		assertTrue(UtilPlayer.canFoul(game, h1));
	}

	// rust: can_foul_false_when_no_prone_adjacent
	@Test
	public void canFoulFalseWhenNoProneAdjacent() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(false, "a1", new FieldCoordinate(6, 5), ACTIVE_STANDING);
		assertFalse(UtilPlayer.canFoul(game, h1));
	}

	// rust: has_ball_false_when_ball_not_at_player
	@Test
	public void hasBallFalseWhenBallNotAtPlayer() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		game.getFieldModel().setBallInPlay(true);
		game.getFieldModel().setBallCoordinate(new FieldCoordinate(10, 10));
		assertFalse(UtilPlayer.hasBall(game, h1));
	}

	// rust: has_ball_true_when_ball_at_player
	@Test
	public void hasBallTrueWhenBallAtPlayer() {
		FieldCoordinate coord = new FieldCoordinate(5, 5);
		Player<?> h1 = addPlayer(true, "h1", coord, ACTIVE_STANDING);
		game.getFieldModel().setBallInPlay(true);
		game.getFieldModel().setBallCoordinate(coord);
		assertTrue(UtilPlayer.hasBall(game, h1));
	}

	// rust: is_next_move_possible_false_when_held_in_place
	@Test
	public void isNextMovePossibleFalseWhenHeldInPlace() {
		addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		game.getActingPlayer().setPlayerId("h1");
		game.getActingPlayer().setHeldInPlace(true);
		assertFalse(UtilPlayer.isNextMovePossible(game, false));
	}

	// NOTE (test equalization): the 4 Rust can_gaze tests are fixture-inexpressible here — canGaze
	// casts `(GameMechanic) game.getFactory(MECHANIC).forName(GAME)`, but a Game built via
	// NetCommandTestUtil.applicationSource() has a null `factories` map, so game.getFactory() NPEs.
	// The GAME-mechanic factory chain isn't available in a headless ffb-common test. Left Rust-only.

	// rust: find_stand_up_assists_no_friendly_adjacent_returns_zero
	@Test
	public void findStandUpAssistsNoFriendlyAdjacentReturnsZero() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		assertEquals(0, UtilPlayer.findStandUpAssists(game, h1));
	}

	// rust: find_stand_up_assists_friendly_not_pressured_counts
	@Test
	public void findStandUpAssistsFriendlyNotPressuredCounts() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(true, "h2", new FieldCoordinate(6, 5), ACTIVE_STANDING);
		assertEquals(1, UtilPlayer.findStandUpAssists(game, h1));
	}

	// rust: find_stand_up_assists_friendly_under_pressure_does_not_count
	@Test
	public void findStandUpAssistsFriendlyUnderPressureDoesNotCount() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(true, "h2", new FieldCoordinate(6, 5), ACTIVE_STANDING);
		addPlayer(false, "a1", new FieldCoordinate(7, 5), ACTIVE_STANDING);
		assertEquals(0, UtilPlayer.findStandUpAssists(game, h1));
	}

	// rust: find_stand_up_assists_two_unpressured_friendlies
	@Test
	public void findStandUpAssistsTwoUnpressuredFriendlies() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(true, "h2", new FieldCoordinate(6, 5), ACTIVE_STANDING);
		addPlayer(true, "h3", new FieldCoordinate(5, 6), ACTIVE_STANDING);
		assertEquals(2, UtilPlayer.findStandUpAssists(game, h1));
	}

	// rust: find_standing_or_prone_players_returns_non_stunned_team_mates
	@Test
	public void findStandingOrPronePlayersReturnsNonStunnedTeamMates() {
		addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(true, "h2", new FieldCoordinate(5, 6), ACTIVE_STANDING);
		addPlayer(true, "h3", new FieldCoordinate(5, 7), PlayerState.STUNNED);
		Player<?>[] result = UtilPlayer.findStandingOrPronePlayers(
			game, game.getTeamHome(), new FieldCoordinate(5, 5), 1);
		assertEquals(1, result.length);
		assertEquals("h2", result[0].getId());
	}

	// rust: find_standing_or_prone_players_distance_2_includes_two_squares_away
	@Test
	public void findStandingOrPronePlayersDistance2IncludesTwoSquaresAway() {
		addPlayer(true, "h1", new FieldCoordinate(5, 5), ACTIVE_STANDING);
		addPlayer(true, "h2", new FieldCoordinate(5, 7), ACTIVE_STANDING);
		Player<?>[] result = UtilPlayer.findStandingOrPronePlayers(
			game, game.getTeamHome(), new FieldCoordinate(5, 5), 2);
		boolean found = false;
		for (Player<?> p : result) {
			if ("h2".equals(p.getId())) {
				found = true;
			}
		}
		assertTrue(found, "should find player 2 squares away");
	}

	// rust: find_standing_or_prone_players_excludes_opposing_team
	@Test
	public void findStandingOrPronePlayersExcludesOpposingTeam() {
		addPlayer(false, "a1", new FieldCoordinate(5, 6), ACTIVE_STANDING);
		Player<?>[] result = UtilPlayer.findStandingOrPronePlayers(
			game, game.getTeamHome(), new FieldCoordinate(5, 5), 1);
		assertEquals(0, result.length);
	}
}
