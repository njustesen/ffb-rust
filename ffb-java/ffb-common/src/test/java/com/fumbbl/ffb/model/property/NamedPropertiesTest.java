package com.fumbbl.ffb.model.property;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/property/named_properties.rs for {@link NamedProperties}.
 */
public class NamedPropertiesTest {

	@Test
	public void allConstantsAreNonEmpty() {
		ISkillProperty[] all = {
			NamedProperties.addBonusForAccuratePass,
			NamedProperties.canLeap,
			NamedProperties.makesDodgingHarder,
			NamedProperties.preventFallOnBothDown,
			NamedProperties.goForItAfterBlock,
		};
		for (ISkillProperty c : all) {
			assertNotNull(c);
			assertFalse(c.getName().isEmpty(), "constant should not be empty: " + c.getName());
		}
	}
}
