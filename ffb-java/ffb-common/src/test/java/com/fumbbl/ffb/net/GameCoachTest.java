package com.fumbbl.ffb.net;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/game_coach.rs tests.
 * The Rust serde_round_trip test has no Java analogue (GameCoach is not JSON
 * serializable in Java); it is intentionally not ported.
 */
public class GameCoachTest {

	@Test
	public void newSetsFields() {
		GameCoach gc = new GameCoach("game1", "coachA");
		assertEquals("game1", gc.getGame());
		assertEquals("coachA", gc.getCoach());
	}

	@Test
	public void equalitySame() {
		GameCoach a = new GameCoach("g", "c");
		GameCoach b = new GameCoach("g", "c");
		assertEquals(a, b);
	}

	@Test
	public void equalityDifferentCoach() {
		GameCoach a = new GameCoach("g", "c1");
		GameCoach b = new GameCoach("g", "c2");
		assertNotEquals(a, b);
	}
}
