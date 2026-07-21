package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.model.RosterPlayer;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of the Rust crates/ffb-mechanics/src/modifiers/stat_based_roll_modifier_factory.rs unit
 * tests. The factory ({@link StatBasedRollModifierFactory}) is a lightweight, stateless helper:
 * given a stat key it reads the corresponding player stat and wraps it in a
 * {@link StatBasedRollModifier}. A bare {@link RosterPlayer} (no temporary stat modifiers) returns
 * its base stat, so these assertions mirror the Rust helper's fixed-stat players 1:1.
 */
class StatBasedRollModifierFactoryTest {

	private RosterPlayer playerWithStats(int agility, int strength, int armour, int movement, int passing) {
		RosterPlayer player = new RosterPlayer();
		player.setAgility(agility);
		player.setStrength(strength);
		player.setArmour(armour);
		player.setMovement(movement);
		player.setPassing(passing);
		return player;
	}

	@Test
	void create_with_ag_reads_agility() {
		StatBasedRollModifierFactory factory = new StatBasedRollModifierFactory("Agility", PlayerStatKey.AG);
		StatBasedRollModifier m = factory.create(playerWithStats(3, 4, 8, 6, 4));
		assertEquals(3, m.getModifier());
		assertEquals("Agility", m.getReportString());
	}

	@Test
	void create_with_st_reads_strength() {
		StatBasedRollModifierFactory factory = new StatBasedRollModifierFactory("Strength", PlayerStatKey.ST);
		StatBasedRollModifier m = factory.create(playerWithStats(3, 5, 8, 6, 4));
		assertEquals(5, m.getModifier());
	}

	@Test
	void create_with_av_reads_armour() {
		StatBasedRollModifierFactory factory = new StatBasedRollModifierFactory("Armour", PlayerStatKey.AV);
		StatBasedRollModifier m = factory.create(playerWithStats(3, 4, 9, 6, 4));
		assertEquals(9, m.getModifier());
	}

	@Test
	void create_with_ma_reads_movement() {
		StatBasedRollModifierFactory factory = new StatBasedRollModifierFactory("Movement", PlayerStatKey.MA);
		StatBasedRollModifier m = factory.create(playerWithStats(3, 4, 8, 6, 4));
		assertEquals(6, m.getModifier());
	}

	@Test
	void create_with_pa_reads_passing() {
		StatBasedRollModifierFactory factory = new StatBasedRollModifierFactory("Passing", PlayerStatKey.PA);
		StatBasedRollModifier m = factory.create(playerWithStats(3, 4, 8, 6, 4));
		assertEquals(4, m.getModifier());
	}
}
