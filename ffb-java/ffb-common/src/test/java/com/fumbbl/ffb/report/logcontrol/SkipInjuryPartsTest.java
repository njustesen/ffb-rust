package com.fumbbl.ffb.report.logcontrol;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/skip_injury_parts.rs tests.
 */
public class SkipInjuryPartsTest {

	// rust: none_skips_nothing
	@Test
	public void noneSkipsNothing() {
		assertFalse(SkipInjuryParts.NONE.isArmour());
		assertFalse(SkipInjuryParts.NONE.isInjury());
		assertFalse(SkipInjuryParts.NONE.isCas());
	}

	// rust: armour_and_cas_skips_armour_and_cas
	@Test
	public void armourAndCasSkipsArmourAndCas() {
		assertTrue(SkipInjuryParts.ARMOUR_AND_CAS.isArmour());
		assertFalse(SkipInjuryParts.ARMOUR_AND_CAS.isInjury());
		assertTrue(SkipInjuryParts.ARMOUR_AND_CAS.isCas());
	}

	// rust: everything_but_cas_skips_armour_and_injury
	@Test
	public void everythingButCasSkipsArmourAndInjury() {
		assertTrue(SkipInjuryParts.EVERYTHING_BUT_CAS.isArmour());
		assertTrue(SkipInjuryParts.EVERYTHING_BUT_CAS.isInjury());
		assertFalse(SkipInjuryParts.EVERYTHING_BUT_CAS.isCas());
	}

	// rust: injury_only_skips_injury (two-arg ctor delegates cas = injury = true)
	@Test
	public void injuryOnlySkipsInjury() {
		assertFalse(SkipInjuryParts.INJURY.isArmour());
		assertTrue(SkipInjuryParts.INJURY.isInjury());
		assertTrue(SkipInjuryParts.INJURY.isCas());
	}

	// rust: armour_and_injury_also_skips_cas (two-arg ctor delegates cas = injury = true)
	@Test
	public void armourAndInjuryAlsoSkipsCas() {
		assertTrue(SkipInjuryParts.ARMOUR_AND_INJURY.isArmour());
		assertTrue(SkipInjuryParts.ARMOUR_AND_INJURY.isInjury());
		assertTrue(SkipInjuryParts.ARMOUR_AND_INJURY.isCas());
	}

	// rust: display_works
	@Test
	public void displayWorks() {
		assertEquals("NONE", SkipInjuryParts.NONE.name());
		assertEquals("ARMOUR_AND_CAS", SkipInjuryParts.ARMOUR_AND_CAS.name());
	}
}
