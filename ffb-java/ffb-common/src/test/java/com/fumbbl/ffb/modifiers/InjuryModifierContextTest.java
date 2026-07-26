package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/injury_modifier_context.rs tests.
 * Java ctor carries an InjuryContext (Rust omits it); passed null here — the getters/flags/mode
 * under test do not touch it.
 */
public class InjuryModifierContextTest {

	private Game game;
	private RosterPlayer defender;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
		defender = new RosterPlayer();
		defender.setId("d");
	}

	// rust: new_has_expected_fields
	@Test
	public void newHasExpectedFields() {
		InjuryModifierContext ctx = new InjuryModifierContext(game, null, null, defender, false, false, false, false);
		assertFalse(ctx.isStab());
		assertFalse(ctx.isFoul());
		assertFalse(ctx.isVomitLike());
		assertFalse(ctx.isChainsaw());
		assertFalse(ctx.isTtm());
		assertTrue(ctx.isAttackerMode());
	}

	// rust: getters_return_set_values
	@Test
	public void gettersReturnSetValues() {
		InjuryModifierContext ctx = new InjuryModifierContext(game, null, null, defender, true, true, true, true);
		assertTrue(ctx.isStab());
		assertTrue(ctx.isFoul());
		assertTrue(ctx.isVomitLike());
		assertTrue(ctx.isChainsaw());
		assertNull(ctx.getAttacker());
	}

	// rust: variant_constructor_sets_ttm
	@Test
	public void variantConstructorSetsTtm() {
		InjuryModifierContext ctx = new InjuryModifierContext(game, null, null, defender, false, false, false, false, true);
		assertTrue(ctx.isTtm());
		assertTrue(ctx.isAttackerMode());
		assertFalse(ctx.isDefenderMode());
	}

	// rust: attacker_none_when_not_provided
	@Test
	public void attackerNoneWhenNotProvided() {
		InjuryModifierContext ctx = new InjuryModifierContext(game, null, null, defender, false, false, false, false);
		assertNull(ctx.getAttacker());
	}

	// rust: set_defender_mode_switches_mode
	@Test
	public void setDefenderModeSwitchesMode() {
		InjuryModifierContext ctx = new InjuryModifierContext(game, null, null, defender, false, false, false, false);
		assertTrue(ctx.isAttackerMode());
		ctx.setDefenderMode();
		assertTrue(ctx.isDefenderMode());
		assertFalse(ctx.isAttackerMode());
	}
}
