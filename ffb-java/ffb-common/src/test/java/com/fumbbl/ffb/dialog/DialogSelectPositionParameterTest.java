package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.PositionChoiceMode;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_select_position_parameter.rs for
 * {@link DialogSelectPositionParameter}.
 *
 * <p>The Rust {@code add_position_filters_empty_string} test is not ported: the
 * Java class has no add-with-filter method (the constructor copies the list
 * verbatim), so there is no equivalent behavior to assert.
 */
public class DialogSelectPositionParameterTest {

	@Test
	public void transformResetsMinMaxSelectToOne() {
		DialogSelectPositionParameter p = new DialogSelectPositionParameter(Arrays.asList("Blitzer"),
			PositionChoiceMode.RAISE_DEAD, 2, 5, "home");
		DialogSelectPositionParameter transformed = (DialogSelectPositionParameter) p.transform();
		assertEquals(1, transformed.getMinSelect());
		assertEquals(1, transformed.getMaxSelect());
		assertEquals("home", transformed.getTeamId());
		assertEquals(Arrays.asList("Blitzer"), transformed.getPositions());
	}

}
