package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.SpecialEffectFactory;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/special_effect.rs for {@link SpecialEffect}.
 */
public class SpecialEffectTest {

	private final SpecialEffectFactory factory = new SpecialEffectFactory();

	@Test
	public void forNameRoundTrip() {
		assertEquals(SpecialEffect.LIGHTNING, factory.forName("lightning"));
	}

	@Test
	public void allVariantsGetNameRoundTrip() {
		SpecialEffect[] variants = {SpecialEffect.LIGHTNING, SpecialEffect.FIREBALL, SpecialEffect.ZAP, SpecialEffect.BOMB};
		for (SpecialEffect variant : variants) {
			assertEquals(variant, factory.forName(variant.getName()),
				variant + " did not round-trip through getName/forName");
		}
	}

}
