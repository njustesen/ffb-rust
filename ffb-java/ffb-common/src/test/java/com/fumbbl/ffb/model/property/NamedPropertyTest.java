package com.fumbbl.ffb.model.property;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/property/named_property.rs for {@link NamedProperty}.
 */
public class NamedPropertyTest {

	@Test
	public void equalityByName() {
		assertEquals(new NamedProperty("canLeap"), new NamedProperty("canLeap"));
		assertNotEquals(new NamedProperty("canLeap"), new NamedProperty("canRun"));
	}
}
