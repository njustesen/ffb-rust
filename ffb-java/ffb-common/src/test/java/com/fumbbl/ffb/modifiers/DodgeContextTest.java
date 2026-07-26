package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/dodge_context.rs tests.
 */
public class DodgeContextTest {

	private Game game;
	private ActingPlayer actingPlayer;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
		game.getTeamHome().setId("home");
		game.getTeamAway().setId("away");
		actingPlayer = game.getActingPlayer();
	}

	// rust: new_has_expected_fields
	@Test
	public void newHasExpectedFields() {
		FieldCoordinate src = new FieldCoordinate(3, 4);
		FieldCoordinate tgt = new FieldCoordinate(5, 6);
		DodgeContext ctx = new DodgeContext(game, actingPlayer, src, tgt);
		assertEquals(src, ctx.getSourceCoordinate());
		assertEquals(tgt, ctx.getTargetCoordinate());
		assertFalse(ctx.isUseBreakTackle());
	}

	// rust: getters_return_set_values
	@Test
	public void gettersReturnSetValues() {
		FieldCoordinate src = new FieldCoordinate(1, 2);
		FieldCoordinate tgt = new FieldCoordinate(7, 8);
		DodgeContext ctx = new DodgeContext(game, actingPlayer, src, tgt);
		assertEquals(src, ctx.getSourceCoordinate());
		assertEquals(tgt, ctx.getTargetCoordinate());
		assertFalse(ctx.isUseBreakTackle());
	}

	// rust: variant_constructor_sets_break_tackle
	@Test
	public void variantConstructorSetsBreakTackle() {
		DodgeContext ctx = new DodgeContext(game, actingPlayer,
			new FieldCoordinate(0, 0), new FieldCoordinate(1, 1), true);
		assertTrue(ctx.isUseBreakTackle());
	}

	// rust: break_tackle_false_by_default
	@Test
	public void breakTackleFalseByDefault() {
		DodgeContext ctx = new DodgeContext(game, actingPlayer,
			new FieldCoordinate(0, 0), new FieldCoordinate(1, 1));
		assertFalse(ctx.isUseBreakTackle());
	}

	// rust: get_game_away_team_id_is_accessible
	@Test
	public void getGameAwayTeamIdIsAccessible() {
		DodgeContext ctx = new DodgeContext(game, actingPlayer,
			new FieldCoordinate(1, 1), new FieldCoordinate(2, 2));
		assertEquals("away", ctx.getGame().getTeamAway().getId());
	}
}
