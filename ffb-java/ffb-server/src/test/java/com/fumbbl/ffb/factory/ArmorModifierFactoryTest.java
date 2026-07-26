package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.SpecialEffect;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.modifiers.ArmorModifier;
import com.fumbbl.ffb.modifiers.ArmorModifierContext;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/armor_modifier_factory.rs tests (subset).
 * The factory is initialized against a GameFixture (BB2025) game.
 */
public class ArmorModifierFactoryTest {

	private Game game;
	private ArmorModifierFactory factory;

	@BeforeEach
	void setUp() {
		game = GameFixture.createGameState().getGame();
		factory = new ArmorModifierFactory();
		factory.initialize(game);
	}

	private RosterPlayer player(String id) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		return p;
	}

	private RosterPlayer playerWithSkill(String id, String skillName) {
		RosterPlayer p = player(id);
		p.addSkill((Skill) game.getFactory(FactoryType.Factory.SKILL).forName(skillName));
		return p;
	}

	// rust: special_effect_returns_fireball_modifier
	@Test
	public void specialEffectReturnsFireballModifier() {
		Set<ArmorModifier> mods = factory.specialEffectArmourModifiers(SpecialEffect.FIREBALL, player("d"));
		assertEquals(1, mods.size());
		assertEquals("Fireball", mods.iterator().next().getName());
	}

	// rust: find_armor_modifiers_mighty_blow_applies_on_block
	@Test
	public void findArmorModifiersMightyBlowAppliesOnBlock() {
		RosterPlayer attacker = playerWithSkill("a", "Mighty Blow");
		RosterPlayer defender = player("d");
		Set<ArmorModifier> mods = factory.findArmorModifiers(game, attacker, defender, false, false);
		assertEquals(1, mods.size());
		assertEquals(1, mods.iterator().next().getModifier(attacker, defender));
	}

	// rust: find_armor_modifiers_mighty_blow_ignores_stab
	@Test
	public void findArmorModifiersMightyBlowIgnoresStab() {
		RosterPlayer attacker = playerWithSkill("a", "Mighty Blow");
		assertTrue(factory.findArmorModifiers(game, attacker, player("d"), true, false).isEmpty());
	}

	// rust: find_armor_modifiers_mighty_blow_ignores_foul
	@Test
	public void findArmorModifiersMightyBlowIgnoresFoul() {
		RosterPlayer attacker = playerWithSkill("a", "Mighty Blow");
		assertTrue(factory.findArmorModifiers(game, attacker, player("d"), false, true).isEmpty());
	}

	// rust: find_armor_modifiers_dirty_player_applies_on_foul
	@Test
	public void findArmorModifiersDirtyPlayerAppliesOnFoul() {
		RosterPlayer attacker = playerWithSkill("a", "Dirty Player");
		Set<ArmorModifier> mods = factory.findArmorModifiers(game, attacker, player("d"), false, true);
		assertEquals(1, mods.size());
	}

	// rust: find_armor_modifiers_dirty_player_ignores_block
	@Test
	public void findArmorModifiersDirtyPlayerIgnoresBlock() {
		RosterPlayer attacker = playerWithSkill("a", "Dirty Player");
		assertTrue(factory.findArmorModifiers(game, attacker, player("d"), false, false).isEmpty());
	}

	// rust: for_name_finds_foul_assist
	@Test
	public void forNameFindsFoulAssist() {
		assertNotNull(factory.forName("1 Offensive Assist"));
	}

	// rust: for_name_returns_none_for_unknown
	@Test
	public void forNameReturnsNoneForUnknown() {
		assertNull(factory.forName("Unknown Modifier"));
	}

	// rust: get_foul_assist_returns_matching_modifier
	@Test
	public void getFoulAssistReturnsMatchingModifier() {
		ArmorModifierContext ctx =
			new ArmorModifierContext(game, player("a"), player("d"), false, true, 3);
		Set<ArmorModifier> mods = factory.getFoulAssist(ctx);
		assertEquals(1, mods.size());
	}

	// rust: get_foul_assist_defensive_assist
	@Test
	public void getFoulAssistDefensiveAssist() {
		ArmorModifierContext ctx =
			new ArmorModifierContext(game, player("a"), player("d"), false, true, -2);
		Set<ArmorModifier> mods = factory.getFoulAssist(ctx);
		assertEquals(1, mods.size());
	}
}
