package com.fumbbl.ffb.model;

import com.fumbbl.ffb.factory.AnimationTypeFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/animation_type.rs for {@link AnimationType}.
 */
public class AnimationTypeTest {

	private final AnimationTypeFactory factory = new AnimationTypeFactory();

	@Test
	public void forNameCaseInsensitive() {
		assertEquals(AnimationType.PASS, factory.forName("pass"));
		assertEquals(AnimationType.PASS, factory.forName("PASS"));
		assertNull(factory.forName("invalid"));
	}

	@Test
	public void allVariantsHaveNonEmptyNames() {
		for (AnimationType v : AnimationType.values()) {
			assertFalse(v.getName().isEmpty(), v + " has empty name");
		}
	}

	@Test
	public void forNameRoundTripsAllVariants() {
		for (AnimationType v : AnimationType.values()) {
			assertEquals(v, factory.forName(v.getName()), "round trip failed for " + v);
		}
	}

}
