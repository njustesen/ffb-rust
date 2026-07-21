package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertSame;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/modifier_type.rs unit tests.
 * Java enum: {@link com.fumbbl.ffb.modifiers.ModifierType}.
 *
 * The Rust serde JSON round-trip tests are ported as name()/valueOf() round-trips, the closest
 * Java analog of the enum's serialization stability (Java ModifierType is a plain enum with no
 * serde/Gson binding of its own).
 */
class ModifierTypeTest {

	@Test
	void variants_distinct() {
		assertNotEquals(ModifierType.REGULAR, ModifierType.TACKLEZONE);
		assertNotEquals(ModifierType.DISTURBING_PRESENCE, ModifierType.DIVING_TACKLE);
	}

	@Test
	void serde_round_trip() {
		ModifierType v = ModifierType.DISTURBING_PRESENCE;
		assertEquals(v, ModifierType.valueOf(v.name()));
	}

	@Test
	void all_variants_serialize_and_deserialize() {
		ModifierType[] variants = {
			ModifierType.DEPENDS_ON_SUM_OF_OTHERS,
			ModifierType.DISTURBING_PRESENCE,
			ModifierType.DIVING_TACKLE,
			ModifierType.PREHENSILE_TAIL,
			ModifierType.REGULAR,
			ModifierType.TACKLEZONE,
			ModifierType.STAT_BASED,
		};
		for (ModifierType v : variants) {
			assertEquals(v, ModifierType.valueOf(v.name()));
		}
	}

	@Test
	void equality_is_reflexive() {
		assertEquals(ModifierType.REGULAR, ModifierType.REGULAR);
		assertEquals(ModifierType.TACKLEZONE, ModifierType.TACKLEZONE);
		assertEquals(ModifierType.STAT_BASED, ModifierType.STAT_BASED);
	}

	@Test
	void copy_semantics() {
		// Java enum analog of Rust's Copy: assignment yields the same singleton instance.
		ModifierType a = ModifierType.TACKLEZONE;
		ModifierType b = a;
		assertEquals(a, b);
		assertSame(a, b);
	}

	@Test
	void hash_works_in_hashmap() {
		Map<ModifierType, Integer> map = new HashMap<>();
		map.put(ModifierType.REGULAR, 1);
		map.put(ModifierType.TACKLEZONE, 2);
		assertEquals(1, map.get(ModifierType.REGULAR));
		assertEquals(2, map.get(ModifierType.TACKLEZONE));
	}
}
