package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/gaze_modifier_context.rs tests.
 */
public class GazeModifierContextTest {

	private Game game;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
		game.getTeamHome().setId("home");
		game.getTeamAway().setId("away");
	}

	private RosterPlayer player(String id, String name) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		p.setName(name);
		return p;
	}

	// rust: new_creates_context_with_player
	@Test
	public void newCreatesContextWithPlayer() {
		GazeModifierContext ctx = new GazeModifierContext(game, player("gazer", "G"));
		assertEquals("gazer", ctx.getPlayer().getId());
	}

	// rust: get_game_returns_game
	@Test
	public void getGameReturnsGame() {
		GazeModifierContext ctx = new GazeModifierContext(game, player("p", "P"));
		assertEquals("home", ctx.getGame().getTeamHome().getId());
	}

	// rust: get_game_away_team_id_is_accessible
	@Test
	public void getGameAwayTeamIdIsAccessible() {
		GazeModifierContext ctx = new GazeModifierContext(game, player("p", "P"));
		assertEquals("away", ctx.getGame().getTeamAway().getId());
	}

	// rust: player_name_is_accessible_via_get_player
	@Test
	public void playerNameIsAccessibleViaGetPlayer() {
		GazeModifierContext ctx = new GazeModifierContext(game, player("p", "Gazer"));
		assertEquals("Gazer", ctx.getPlayer().getName());
	}

	// rust: two_contexts_can_share_same_game
	@Test
	public void twoContextsCanShareSameGame() {
		GazeModifierContext ctx1 = new GazeModifierContext(game, player("p1", "P1"));
		GazeModifierContext ctx2 = new GazeModifierContext(game, player("p2", "P2"));
		assertEquals(ctx1.getGame().getTeamHome().getId(), ctx2.getGame().getTeamHome().getId());
	}
}
