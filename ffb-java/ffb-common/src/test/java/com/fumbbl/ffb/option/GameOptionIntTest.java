package com.fumbbl.ffb.option;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-model/src/option/game_option_int.rs tests.
 * Java's GameOptionInt is keyed by GameOptionId (enum) rather than a free string; TURNTIME is used.
 */
public class GameOptionIntTest {

	// rust: new_has_zero_value
	@Test
	public void newHasZeroValue() {
		assertEquals(0, new GameOptionInt(GameOptionId.TURNTIME).getValue());
	}

	// rust: set_value_from_string
	@Test
	public void setValueFromString() {
		assertEquals(120, new GameOptionInt(GameOptionId.TURNTIME).setValue("120").getValue());
	}

	// rust: set_default_sets_value_too
	@Test
	public void setDefaultSetsValueToo() {
		GameOptionInt opt = new GameOptionInt(GameOptionId.TURNTIME);
		opt.setDefault(60);
		assertEquals(60, opt.getValue());
		assertFalse(opt.isChanged());
	}

	// rust: is_changed_after_setting_non_default
	@Test
	public void isChangedAfterSettingNonDefault() {
		GameOptionInt opt = new GameOptionInt(GameOptionId.TURNTIME);
		opt.setDefault(60);
		opt.setValue(120);
		assertTrue(opt.isChanged());
	}

	// rust: display_message_with_template
	// NOTE: Java StringTool.bind uses "$N" 1-based placeholders (regex [$]([0-9]+)), whereas Rust's
	// bind uses "{0}" 0-based; the template uses Java's "$1" convention (documented divergence).
	@Test
	public void displayMessageWithTemplate() {
		GameOptionInt opt = new GameOptionInt(GameOptionId.TURNTIME);
		opt.setMessage("Turn time: $1s");
		opt.setValue(90);
		assertEquals("Turn time: 90s", opt.getDisplayMessage());
	}
}
