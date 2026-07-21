package com.fumbbl.ffb.injury.context;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/injury/context/injury_modification.rs for {@link InjuryModification}.
 */
public class InjuryModificationTest {

	@Test
	public void serdeRoundTrip() {
		for (InjuryModification v : new InjuryModification[] { InjuryModification.ARMOUR, InjuryModification.INJURY,
			InjuryModification.NONE }) {
			InjuryModification back = InjuryModification.valueOf(v.name());
			assertEquals(v, back);
		}
	}
}
