package com.fumbbl.ffb.model;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/keyword.rs for {@link Keyword}.
 */
public class KeywordTest {

	@Test
	public void forNameDwarf() {
		assertEquals(Keyword.DWARF, Keyword.forName("Dwarf"));
		assertEquals(Keyword.DWARF, Keyword.forName("dwarf"));
	}

	@Test
	public void forNameUnknownFallback() {
		assertEquals(Keyword.UNKNOWN, Keyword.forName("nonexistent"));
	}

	@Test
	public void bigGuyCannotGetEvenWith() {
		assertFalse(Keyword.BIG_GUY.isCanGetEvenWith());
	}

	@Test
	public void dwarfCanGetEvenWith() {
		assertTrue(Keyword.DWARF.isCanGetEvenWith());
	}

	@Test
	public void forNameVampireReturnsVampire() {
		assertEquals(Keyword.VAMPIRE, Keyword.forName("Vampire"));
	}

}
