package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Chainsaw;
import com.fumbbl.ffb.injury.Foul;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/old_pro_modification.rs tests (public subset).
 * The is_spotted_foul / is_self_inflicted tests are EXEMPT — those helpers are private in the Java
 * OldProModification (the Rust made them pub for unit testing), so they cannot be called from a test.
 */
public class OldProModificationTest {

	private static GameState gs;

	@BeforeAll
	static void setUp() {
		gs = GameFixture.createGameState();
		GameFixture.placePlayer(gs, "home1", 5, 5);
		com.fumbbl.ffb.server.fixture.GameFixture.setActingPlayer(gs, "home1", com.fumbbl.ffb.PlayerAction.BLOCK);
		GameFixture.placePlayer(gs, "away1", 6, 5);
	}

	private OldProModification make() {
		return new OldProModification();
	}

	// rust: requires_conditional_re_roll
	@Test
	public void requiresConditionalReRoll() {
		assertTrue(make().requiresConditionalReRollSkill());
	}

	// rust: valid_types_include_block_foul_chainsaw
	@Test
	public void validTypesIncludeBlockFoulChainsaw() {
		OldProModification m = make();
		assertTrue(m.isValidType(new Block()));
		assertTrue(m.isValidType(new Foul()));
		assertTrue(m.isValidType(new Chainsaw()));
	}

	// rust: modify_armour_returns_false_without_armor_roll
	@Test
	public void modifyArmourReturnsFalseWithoutArmorRoll() {
		InjuryContext ctx = new InjuryContext();
		ctx.setAttackerId("home1");
		ctx.setDefenderId("away1");
		assertFalse(make().modifyArmour(gs, ctx, new Block()));
	}
}
