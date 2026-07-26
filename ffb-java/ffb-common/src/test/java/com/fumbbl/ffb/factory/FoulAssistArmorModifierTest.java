package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.ArmorModifierContext;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/foul_assist_armor_modifier.rs tests.
 * FoulAssistArmorModifier (com.fumbbl.ffb.factory, ffb-common) extends StaticArmourModifier;
 * appliesToContext = context.isFoul() && context.getFoulAssists() == modifier.
 */
public class FoulAssistArmorModifierTest {

	private Game game;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
	}

	private RosterPlayer player(String id) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		return p;
	}

	private ArmorModifierContext ctx(boolean isFoul, int foulAssists) {
		return new ArmorModifierContext(game, player("a"), player("d"), false, isFoul, foulAssists);
	}

	// rust: applies_when_foul_and_matching_assists
	@Test
	public void appliesWhenFoulAndMatchingAssists() {
		FoulAssistArmorModifier m = new FoulAssistArmorModifier("2 Offensive Assists", 2, true);
		assertTrue(m.appliesToContext(ctx(true, 2)));
	}

	// rust: does_not_apply_when_foul_but_wrong_assists
	@Test
	public void doesNotApplyWhenFoulButWrongAssists() {
		FoulAssistArmorModifier m = new FoulAssistArmorModifier("2 Offensive Assists", 2, true);
		assertFalse(m.appliesToContext(ctx(true, 3)));
	}

	// rust: does_not_apply_when_not_foul
	@Test
	public void doesNotApplyWhenNotFoul() {
		FoulAssistArmorModifier m = new FoulAssistArmorModifier("2 Offensive Assists", 2, true);
		assertFalse(m.appliesToContext(ctx(false, 2)));
	}

	// rust: get_modifier_returns_stored_value
	@Test
	public void getModifierReturnsStoredValue() {
		FoulAssistArmorModifier m = new FoulAssistArmorModifier("test", 3, false);
		assertEquals(3, m.getModifier(null, null));
	}

	// rust: is_foul_assist_modifier_reflects_constructor_arg
	@Test
	public void isFoulAssistModifierReflectsConstructorArg() {
		assertTrue(new FoulAssistArmorModifier("x", 1, true).isFoulAssistModifier());
		assertFalse(new FoulAssistArmorModifier("x", 1, false).isFoulAssistModifier());
	}
}
