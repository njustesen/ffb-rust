package com.fumbbl.ffb.modifiers;

import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertSame;

/**
 * 1:1 mirror of the Rust crates/ffb-mechanics/src/modifiers/player_stat_key.rs unit tests.
 * Java enum: {@link com.fumbbl.ffb.modifiers.PlayerStatKey}.
 *
 * The Rust serde JSON round-trip tests are ported as name()/valueOf() round-trips, the closest
 * Java analog of the enum's serialization stability (Java PlayerStatKey is a plain enum).
 */
class PlayerStatKeyTest {

	@Test
	void variants_distinct() {
		assertNotEquals(PlayerStatKey.AG, PlayerStatKey.AV);
		assertNotEquals(PlayerStatKey.MA, PlayerStatKey.ST);
	}

	@Test
	void serde_round_trip() {
		PlayerStatKey v = PlayerStatKey.AG;
		assertEquals(v, PlayerStatKey.valueOf(v.name()));
	}

	@Test
	void all_variants_round_trip() {
		PlayerStatKey[] variants = {
			PlayerStatKey.MA, PlayerStatKey.ST, PlayerStatKey.AG, PlayerStatKey.PA, PlayerStatKey.AV
		};
		for (PlayerStatKey v : variants) {
			assertEquals(v, PlayerStatKey.valueOf(v.name()));
		}
	}

	@Test
	void equality_is_reflexive() {
		assertEquals(PlayerStatKey.MA, PlayerStatKey.MA);
		assertEquals(PlayerStatKey.PA, PlayerStatKey.PA);
	}

	@Test
	void hash_works_in_hashmap() {
		Map<PlayerStatKey, String> map = new HashMap<>();
		map.put(PlayerStatKey.MA, "movement");
		map.put(PlayerStatKey.AG, "agility");
		assertEquals("movement", map.get(PlayerStatKey.MA));
		assertEquals("agility", map.get(PlayerStatKey.AG));
	}

	@Test
	void copy_semantics() {
		// Java enum analog of Rust's Copy: assignment yields the same singleton instance.
		PlayerStatKey a = PlayerStatKey.ST;
		PlayerStatKey b = a;
		assertEquals(a, b);
		assertSame(a, b);
	}
}
