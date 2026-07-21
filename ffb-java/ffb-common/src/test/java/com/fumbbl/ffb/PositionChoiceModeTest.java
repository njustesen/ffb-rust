package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/position_choice_mode.rs for {@link PositionChoiceMode}.
 */
public class PositionChoiceModeTest {

	@Test
	public void serdeRoundTrip() {
		assertEquals(PositionChoiceMode.RAISE_DEAD, PositionChoiceMode.valueOf(PositionChoiceMode.RAISE_DEAD.name()));
	}

}
