package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Stab;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/brutal_block_modification.rs tests.
 * Test lives in the modification package so it can reach the protected skillUse()/tryInjuryModification().
 * <p>
 * EXEMPT: the Rust try_injury_modification_false_when_no_active_player test asserts false when the
 * game has no acting player; Java's tryInjuryModification dereferences game.getActingPlayer().getPlayer()
 * (there is always an acting player during injury resolution), so the no-acting-player scenario has no
 * faithful Java path. The casualty branch below covers the false path via short-circuit.
 */
public class BrutalBlockModificationTest {

	private BrutalBlockModification make() {
		return new BrutalBlockModification();
	}

	// rust: valid_type_is_block
	@Test
	public void validTypeIsBlock() {
		assertTrue(make().isValidType(new Block()));
		assertFalse(make().isValidType(new Stab()));
	}

	// rust: skill_use_is_add_injury_modifier
	@Test
	public void skillUseIsAddInjuryModifier() {
		assertEquals(SkillUse.ADD_INJURY_MODIFIER, make().skillUse());
	}

	// rust: try_injury_modification_false_when_casualty
	@Test
	public void tryInjuryModificationFalseWhenCasualty() {
		Game game = GameFixture.createGameState().getGame();
		InjuryContext ctx = new InjuryContext();
		ctx.setInjury(new PlayerState(PlayerState.SERIOUS_INJURY));
		assertFalse(make().tryInjuryModification(game, ctx, new Block()));
	}

	// rust: skill_id_starts_as_none
	@Test
	public void skillIdStartsAsNone() {
		assertNull(make().getSkill());
	}
}
