package com.fumbbl.ffb;

import com.fumbbl.ffb.modifiers.PlayerStatKey;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/injury_attribute.rs for {@link InjuryAttribute}.
 */
public class InjuryAttributeTest {

	@Test
	public void forNameRoundTrip() {
		assertEquals(InjuryAttribute.MA, InjuryAttribute.forName("MA"));
		assertEquals(InjuryAttribute.AG, InjuryAttribute.forName("AG"));
	}

	@Test
	public void forNameStripsPrefix() {
		assertEquals(InjuryAttribute.MA, InjuryAttribute.forName("-MA"));
		assertEquals(InjuryAttribute.AG, InjuryAttribute.forName("+AG"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(InjuryAttribute.forName("XX"));
	}

	@Test
	public void getIdIsUniquePerVariant() {
		assertNotEquals(InjuryAttribute.MA.getId(), InjuryAttribute.AG.getId());
	}

	@Test
	public void forStatKeyMaReturnsMa() {
		assertEquals(InjuryAttribute.MA, InjuryAttribute.forStatKey(PlayerStatKey.MA));
	}

	@Test
	public void forNameCaseInsensitive() {
		assertEquals(InjuryAttribute.MA, InjuryAttribute.forName("ma"));
	}

}
