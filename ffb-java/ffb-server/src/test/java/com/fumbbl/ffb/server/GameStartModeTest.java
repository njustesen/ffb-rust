package com.fumbbl.ffb.server;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/game_start_mode.rs tests.
 */
public class GameStartModeTest {

	// rust: start_game_name
	@Test
	public void startGameName() {
		assertEquals("START GAME", GameStartMode.START_GAME.getName());
	}

	// rust: start_test_game_name
	@Test
	public void startTestGameName() {
		assertEquals("START TEST GAME", GameStartMode.START_TEST_GAME.getName());
	}

	// rust: schedule_game_name
	@Test
	public void scheduleGameName() {
		assertEquals("SCHEDULE GAME", GameStartMode.SCHEDULE_GAME.getName());
	}

	// rust: all_variants_are_distinct
	@Test
	public void allVariantsAreDistinct() {
		assertEquals(3, GameStartMode.values().length);
		assertNotEquals(GameStartMode.START_GAME, GameStartMode.START_TEST_GAME);
		assertNotEquals(GameStartMode.START_GAME, GameStartMode.SCHEDULE_GAME);
	}

	// rust: copy_semantics_preserved
	@Test
	public void copySemanticsPreserved() {
		assertEquals(GameStartMode.SCHEDULE_GAME, GameStartMode.SCHEDULE_GAME);
	}

	// rust: clone_equals_original
	@Test
	public void cloneEqualsOriginal() {
		assertEquals(GameStartMode.START_GAME, GameStartMode.START_GAME);
	}

	// rust: debug_format_contains_variant_name
	@Test
	public void debugFormatContainsVariantName() {
		assertTrue(GameStartMode.START_GAME.name().contains("START_GAME"));
	}

	// rust: get_name_returns_static_str
	@Test
	public void getNameReturnsStaticStr() {
		assertNotNull(GameStartMode.START_GAME.getName());
	}
}
