package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.bb2020.SeriousInjury;
import com.fumbbl.ffb.factory.mixed.CasualtyModifierFactory;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.bb2020.CasualtyModifier;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of the portable subset of the Rust
 * crates/ffb-mechanics/src/modifiers/casualty_modifier_factory.rs unit tests, covering
 * {@link CasualtyModifierFactory#findModifiers(com.fumbbl.ffb.model.Player)}.
 *
 * <p>In Java the niggling count is derived from the player's lasting injuries carrying the NI
 * ({@code InjuryAttribute.NI}) attribute rather than from a plain integer field, so each Niggling
 * Injury is expressed as a {@code SeriousInjury.SERIOUS_INJURY} lasting injury. {@code findModifiers}
 * needs no game/aggregator initialisation, so this mirrors the Rust behaviour with lightweight
 * inputs.
 *
 * <p>The Rust tests for {@code for_number}, {@code from_name}, {@code for_name},
 * {@code find_registered_modifiers} (Decay) and beyond-static-table pluralisation are NOT ported:
 * those Java methods are private ({@code fromName}/{@code forNumber}) or require
 * {@code initialize(Game)} to populate the {@code ModifierAggregator} ({@code forName}), and Decay is
 * sourced from skills/aggregator in Java rather than a static table.
 */
class CasualtyModifierFactoryTest {

	private RosterPlayer playerWithNigglings(int count) {
		RosterPlayer player = new RosterPlayer();
		for (int i = 0; i < count; i++) {
			player.addLastingInjury(SeriousInjury.SERIOUS_INJURY);
		}
		return player;
	}

	@Test
	void find_modifiers_empty_when_no_nigglings() {
		CasualtyModifierFactory factory = new CasualtyModifierFactory();
		assertTrue(factory.findModifiers(playerWithNigglings(0)).isEmpty());
	}

	@Test
	void find_modifiers_single_niggling() {
		CasualtyModifierFactory factory = new CasualtyModifierFactory();
		Set<CasualtyModifier> mods = factory.findModifiers(playerWithNigglings(1));
		assertEquals(1, mods.size());
		CasualtyModifier modifier = mods.iterator().next();
		assertEquals("1 Niggling Injury", modifier.getName());
		assertEquals(1, modifier.getModifier());
	}

	@Test
	void find_modifiers_pluralizes_multiple_nigglings() {
		CasualtyModifierFactory factory = new CasualtyModifierFactory();
		Set<CasualtyModifier> mods = factory.findModifiers(playerWithNigglings(3));
		assertEquals(1, mods.size());
		CasualtyModifier modifier = mods.iterator().next();
		assertEquals("3 Niggling Injuries", modifier.getName());
		assertEquals(3, modifier.getModifier());
	}
}
