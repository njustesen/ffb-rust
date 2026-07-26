package com.fumbbl.ffb.server.injury.modification.bb2025;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Stab;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.injury.context.ModifiedInjuryContext;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.injury.modification.ModificationParams;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/bb2025/slayer_modification.rs tests.
 * bb2025 SlayerModification extends AvOrInjModification and adds a "defender is a Big Guy" gate, so a
 * placed acting attacker + a non-big-guy (lineman) defender make both tryArmour and tryInjury false.
 * The skillUse test is EXEMPT here — skillUse() is declared protected in the base-package
 * AvOrInjModification, so it is not accessible from this bb2025-package test.
 */
public class SlayerModificationTest {

	private GameState gs;

	@BeforeEach
	void setUp() {
		gs = GameFixture.createGameState();
		GameFixture.placePlayer(gs, "home1", 5, 5);
		GameFixture.setActingPlayer(gs, "home1", PlayerAction.BLOCK);
		GameFixture.placePlayer(gs, "away1", 6, 5);
	}

	private SlayerModification make() {
		return new SlayerModification();
	}

	// rust: valid_type_is_block
	@Test
	public void validTypeIsBlock() {
		assertTrue(make().isValidType(new Block()));
	}

	// rust: stab_is_not_valid_type
	@Test
	public void stabIsNotValidType() {
		assertFalse(make().isValidType(new Stab()));
	}

	// rust: try_armour_false_without_big_guy_defender
	@Test
	public void tryArmourFalseWithoutBigGuyDefender() {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		ctx.setDefenderId("away1");
		ModificationParams params = new ModificationParams(gs, ctx, new Block());
		assertFalse(make().tryArmourRollModification(params));
	}

	// rust: try_injury_false_without_big_guy_defender
	@Test
	public void tryInjuryFalseWithoutBigGuyDefender() {
		InjuryContext ctx = new InjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		ctx.setDefenderId("away1");
		assertFalse(make().tryInjuryModification(gs.getGame(), ctx, new Block()));
	}
}
