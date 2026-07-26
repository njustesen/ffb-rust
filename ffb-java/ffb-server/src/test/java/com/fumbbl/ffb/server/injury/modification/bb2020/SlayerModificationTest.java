package com.fumbbl.ffb.server.injury.modification.bb2020;

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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/bb2020/slayer_modification.rs tests.
 * bb2020 SlayerModification extends AvOrInjModification and gates on the defender having ST >= 5; the
 * GameFixture linemen are ST 3, so a placed acting attacker + lineman defender make both tryArmour
 * and tryInjury false (same outcome as the Rust ST-4 defender). EXEMPT: skillUse (declared protected
 * in base-package AvOrInj, not accessible here) and try_armour_false_when_no_defender (Java derefs
 * getPlayerById(null); Rust-defensive).
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
		assertFalse(make().isValidType(new Stab()));
	}

	// rust: try_armour_false_when_defender_st4 (lineman defender is ST 3, below the ST-5 threshold)
	@Test
	public void tryArmourFalseWhenDefenderBelowSt5() {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		ctx.setDefenderId("away1");
		ModificationParams params = new ModificationParams(gs, ctx, new Block());
		assertFalse(make().tryArmourRollModification(params));
	}

	// rust: try_injury_false_when_defender_st4
	@Test
	public void tryInjuryFalseWhenDefenderBelowSt5() {
		InjuryContext ctx = new InjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		ctx.setDefenderId("away1");
		assertFalse(make().tryInjuryModification(gs.getGame(), ctx, new Block()));
	}
}
