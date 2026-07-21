package com.fumbbl.ffb.factory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/game_option_id_factory.rs
 * for {@link GameOptionIdFactory}.
 * <p>
 * The Rust {@code GameOptionId} wraps any non-empty string, so its factory returns {@code Some}
 * for arbitrary non-empty names. The Java {@link com.fumbbl.ffb.option.GameOptionId} is a fixed
 * enum whose factory returns {@code null} for unknown names, so only the empty-string test is a
 * faithful mirror; the arbitrary-name and whitespace cases are intentionally not ported.
 */
public class GameOptionIdFactoryTest {

	@Test
	public void forNameEmptyReturnsNone() {
		assertNull(new GameOptionIdFactory().forName(""));
	}
}
