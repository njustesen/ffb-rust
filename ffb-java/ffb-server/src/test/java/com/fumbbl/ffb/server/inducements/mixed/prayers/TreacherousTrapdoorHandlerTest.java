package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.stadium.TrapDoor;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/mixed/prayers/treacherous_trapdoor_handler.rs tests,
 * exercised through the concrete bb2020 subclass typed as the mixed class.
 */
public class TreacherousTrapdoorHandlerTest {

	private GameState gameState;
	private Game game;
	private TreacherousTrapdoorHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.TreacherousTrapdoorHandler();
	}

	private boolean hasTrapDoor(int x, int y) {
		return game.getFieldModel().getTrapDoors().contains(new TrapDoor(new FieldCoordinate(x, y)));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_TREACHEROUS_TRAPDOOR, handler.animationType());
	}

	// rust: init_effect_adds_two_trapdoors (also covers trapdoor_coordinates_are_correct)
	@Test
	public void initEffectAddsTwoTrapdoors() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(hasTrapDoor(6, 1));
		assertTrue(hasTrapDoor(19, 13));
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: remove_effect_clears_trapdoors
	@Test
	public void removeEffectClearsTrapdoors() {
		handler.initEffect(gameState, game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(hasTrapDoor(6, 1));
		assertFalse(hasTrapDoor(19, 13));
		assertTrue(game.getFieldModel().getTrapDoors().isEmpty());
	}

	// rust: remove_effect_no_panic_when_no_trapdoors
	@Test
	public void removeEffectNoPanicWhenNoTrapdoors() {
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertTrue(game.getFieldModel().getTrapDoors().isEmpty());
	}
}
