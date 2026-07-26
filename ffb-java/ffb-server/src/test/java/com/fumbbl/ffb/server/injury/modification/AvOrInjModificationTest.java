package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Stab;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.injury.context.ModifiedInjuryContext;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/av_or_inj_modification.rs tests.
 * Both tryArmourRollModification and tryInjuryModification check the acting player's tacklezones,
 * so a standing acting player is placed (placePlayer sets STANDING, setActingPlayer wires it).
 * Same-package test for protected skillUse()/tryArmour/tryInjury.
 * <p>
 * EXEMPT: the Rust modify_injury_sets_add_injury_modifier_skill_use test — Java modifyInjuryInternal
 * dereferences getSkill().getArmorModifiers() (a registered skill is present in production; there is
 * no no-skill path).
 */
public class AvOrInjModificationTest {

	private static GameState gs;

	@BeforeAll
	static void setUp() {
		gs = GameFixture.createGameState();
		GameFixture.placePlayer(gs, "home1", 5, 5);
		GameFixture.setActingPlayer(gs, "home1", PlayerAction.MOVE);
	}

	private AvOrInjModification make() {
		return new AvOrInjModification();
	}

	private ModificationParams params(boolean armorBroken) {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		ctx.setArmorBroken(armorBroken);
		return new ModificationParams(gs, ctx, new Block());
	}

	// rust: valid_type_is_block
	@Test
	public void validTypeIsBlock() {
		assertTrue(make().isValidType(new Block()));
		assertFalse(make().isValidType(new Stab()));
	}

	// rust: skill_use_is_add_armour_modifier
	@Test
	public void skillUseIsAddArmourModifier() {
		assertEquals(SkillUse.ADD_ARMOUR_MODIFIER, make().skillUse());
	}

	// rust: try_armour_false_when_armor_broken
	@Test
	public void tryArmourFalseWhenArmorBroken() {
		assertFalse(make().tryArmourRollModification(params(true)));
	}

	// rust: try_armour_true_when_not_broken
	@Test
	public void tryArmourTrueWhenNotBroken() {
		assertTrue(make().tryArmourRollModification(params(false)));
	}

	// rust: try_injury_false_when_casualty
	@Test
	public void tryInjuryFalseWhenCasualty() {
		InjuryContext ctx = new InjuryContext();
		ctx.setInjury(new PlayerState(PlayerState.BADLY_HURT));
		assertFalse(make().tryInjuryModification(gs.getGame(), ctx, new Block()));
	}

	// rust: try_injury_true_when_not_casualty
	@Test
	public void tryInjuryTrueWhenNotCasualty() {
		InjuryContext ctx = new InjuryContext();
		ctx.setInjury(new PlayerState(PlayerState.STUNNED));
		assertTrue(make().tryInjuryModification(gs.getGame(), ctx, new Block()));
	}

	// rust: skill_id_starts_as_none
	@Test
	public void skillIdStartsAsNone() {
		assertNull(make().getSkill());
	}
}
