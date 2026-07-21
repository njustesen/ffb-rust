package com.fumbbl.ffb.model.change;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/change/model_change.rs for {@link ModelChange}.
 * The Rust ModelChange bundles a ModelChangeDataType; the Java ModelChange derives the data type from the id,
 * so the data_type_stored case has no equivalent and is omitted.
 */
public class ModelChangeTest {

	@Test
	public void newSetsChangeId() {
		ModelChange mc = new ModelChange(ModelChangeId.GAME_SET_HOME_PLAYING, null, null);
		assertEquals(ModelChangeId.GAME_SET_HOME_PLAYING, mc.getChangeId());
	}

}
