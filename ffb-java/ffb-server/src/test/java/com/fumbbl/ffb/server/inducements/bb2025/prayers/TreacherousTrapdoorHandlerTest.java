package com.fumbbl.ffb.server.inducements.bb2025.prayers;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2025.Prayer;
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
 * crates/ffb-engine/src/inducements/bb2025/prayers/treacherous_trapdoor_handler.rs tests.
 */
public class TreacherousTrapdoorHandlerTest {

	private GameState gameState;
	private Game game;
	private TreacherousTrapdoorHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		handler = new TreacherousTrapdoorHandler();
	}

	private boolean hasTrapDoor(int x, int y) {
		return game.getFieldModel().getTrapDoors().contains(new TrapDoor(new FieldCoordinate(x, y)));
	}

	// rust: handles_prayer_treacherous_trapdoor
	@Test
	public void handlesPrayerTreacherousTrapdoor() {
		assertEquals(Prayer.TREACHEROUS_TRAPDOOR, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.TREACHEROUS_TRAPDOOR));
		assertFalse(handler.handles(Prayer.THROW_A_ROCK));
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: init_effect_adds_two_trapdoors
	@Test
	public void initEffectAddsTwoTrapdoors() {
		handler.initEffect(gameState, game.getTeamHome());
		assertTrue(hasTrapDoor(6, 1));
		assertTrue(hasTrapDoor(19, 13));
	}

	// rust: remove_effect_clears_trapdoors
	@Test
	public void removeEffectClearsTrapdoors() {
		handler.initEffect(gameState, game.getTeamHome());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(hasTrapDoor(6, 1));
		assertFalse(hasTrapDoor(19, 13));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_TREACHEROUS_TRAPDOOR, handler.animationType());
	}
}
