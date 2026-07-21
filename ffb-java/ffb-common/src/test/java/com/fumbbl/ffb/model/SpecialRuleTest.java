package com.fumbbl.ffb.model;

import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/special_rule.rs for {@link SpecialRule}.
 */
public class SpecialRuleTest {

	@Test
	public void fromCaseInsensitive() {
		assertEquals(SpecialRule.SWARMING, SpecialRule.from("swarming"));
		assertEquals(SpecialRule.SWARMING, SpecialRule.from("SWARMING"));
	}

	@Test
	public void fromUnknown() {
		assertNull(SpecialRule.from("unknown"));
	}

	@Test
	public void allVariantsRoundTripThroughGetRuleName() {
		SpecialRule[] variants = {
			SpecialRule.BADLANDS_BRAWL, SpecialRule.ELVEN_KINGDOMS_LEAGUE,
			SpecialRule.HALFLING_THIMBLE_CUP, SpecialRule.LUSTRIAN_SUPERLEAGUE,
			SpecialRule.OLD_WORLD_CLASSIC, SpecialRule.SYLVANIAN_SPOTLIGHT,
			SpecialRule.UNDERWORLD_CHALLENGE, SpecialRule.WORLDS_EDGE_SUPERLEAGUE,
			SpecialRule.BRIBERY_AND_CORRUPTION, SpecialRule.FAVOURED_OF_UNDIVIDED,
			SpecialRule.FAVOURED_OF_KHORNE, SpecialRule.FAVOURED_OF_NURGLE,
			SpecialRule.FAVOURED_OF_TZEENTCH, SpecialRule.FAVOURED_OF_SLAANESH,
			SpecialRule.LOW_COST_LINEMEN, SpecialRule.SWARMING,
			SpecialRule.MASTERS_OF_UNDEATH, SpecialRule.BRAWLIN_BRUTES
		};
		for (SpecialRule variant : variants) {
			assertEquals(variant, SpecialRule.from(variant.getRuleName()));
		}
	}

	@Test
	public void favouredOfVariantsHaveDistinctNames() {
		String[] names = {
			SpecialRule.FAVOURED_OF_UNDIVIDED.getRuleName(),
			SpecialRule.FAVOURED_OF_KHORNE.getRuleName(),
			SpecialRule.FAVOURED_OF_NURGLE.getRuleName(),
			SpecialRule.FAVOURED_OF_TZEENTCH.getRuleName(),
			SpecialRule.FAVOURED_OF_SLAANESH.getRuleName()
		};
		Set<String> unique = new HashSet<>();
		for (String name : names) {
			unique.add(name);
		}
		assertEquals(names.length, unique.size());
	}

}
