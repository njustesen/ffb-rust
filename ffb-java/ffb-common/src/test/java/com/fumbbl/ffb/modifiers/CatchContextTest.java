package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.CatchScatterThrowInMode;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/catch_context.rs tests.
 */
public class CatchContextTest {

	private Game game;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
		game.getTeamHome().setId("home");
		game.getTeamAway().setId("away");
	}

	// rust: new_has_expected_fields
	@Test
	public void newHasExpectedFields() {
		CatchContext ctx = new CatchContext(game, null, CatchScatterThrowInMode.CATCH_ACCURATE_PASS, null);
		assertEquals(CatchScatterThrowInMode.CATCH_ACCURATE_PASS, ctx.getCatchMode());
		assertFalse(ctx.getUsingBlastIt());
		assertNull(ctx.getPlayer());
	}

	// rust: getters_return_set_values
	@Test
	public void gettersReturnSetValues() {
		RosterPlayer player = new RosterPlayer();
		player.setId("p");
		CatchContext ctx = new CatchContext(game, player, CatchScatterThrowInMode.CATCH_HAND_OFF, Boolean.TRUE);
		assertEquals(CatchScatterThrowInMode.CATCH_HAND_OFF, ctx.getCatchMode());
		assertTrue(ctx.getUsingBlastIt());
		assertNotNull(ctx.getPlayer());
	}

	// rust: flag_toggles_blast_it_none_defaults_false
	@Test
	public void flagTogglesBlastItNoneDefaultsFalse() {
		CatchContext ctxDefault = new CatchContext(game, null, CatchScatterThrowInMode.CATCH_BOMB, null);
		CatchContext ctxExplicit = new CatchContext(game, null, CatchScatterThrowInMode.CATCH_BOMB, Boolean.TRUE);
		assertFalse(ctxDefault.getUsingBlastIt());
		assertTrue(ctxExplicit.getUsingBlastIt());
	}

	// rust: player_none_when_not_provided
	@Test
	public void playerNoneWhenNotProvided() {
		CatchContext ctx = new CatchContext(game, null, CatchScatterThrowInMode.CATCH_HAND_OFF, null);
		assertNull(ctx.getPlayer());
	}

	// rust: get_game_away_team_id_is_accessible
	@Test
	public void getGameAwayTeamIdIsAccessible() {
		CatchContext ctx = new CatchContext(game, null, CatchScatterThrowInMode.CATCH_SCATTER, null);
		assertEquals("away", ctx.getGame().getTeamAway().getId());
	}
}
