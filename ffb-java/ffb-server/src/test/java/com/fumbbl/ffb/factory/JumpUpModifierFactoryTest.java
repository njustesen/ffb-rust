package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.modifiers.JumpUpContext;
import com.fumbbl.ffb.modifiers.JumpUpModifier;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/jump_up_modifier_factory.rs tests.
 */
public class JumpUpModifierFactoryTest {

	private GameState gameState;
	private Game game;
	private JumpUpModifierFactory factory;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3));
	}

	private void init(GameState gs) {
		gameState = gs;
		game = gameState.getGame();
		factory = new JumpUpModifierFactory();
		factory.initialize(game);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private JumpUpContext context() {
		return new JumpUpContext(game.getActingPlayer(), game);
	}

	// rust: bb2016_selects_bb2016_collection
	@Test
	public void bb2016SelectsBb2016Collection() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		Set<JumpUpModifier> mods = factory.findModifiers(context());
		assertEquals(1, mods.size());
		JumpUpModifier m = mods.iterator().next();
		assertEquals("Jump Up", m.getName());
		assertEquals(-2, m.getModifier());
	}

	// rust: bb2020_selects_mixed_collection
	@Test
	public void bb2020SelectsMixedCollection() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2020));
		Set<JumpUpModifier> mods = factory.findModifiers(context());
		assertEquals(1, mods.size());
		JumpUpModifier m = mods.iterator().next();
		assertEquals("Jump Up", m.getName());
		assertEquals(-1, m.getModifier());
	}

	// rust: bb2025_selects_mixed_collection
	@Test
	public void bb2025SelectsMixedCollection() {
		Set<JumpUpModifier> mods = factory.findModifiers(context());
		assertEquals(1, mods.size());
		assertEquals(-1, mods.iterator().next().getModifier());
	}
}
