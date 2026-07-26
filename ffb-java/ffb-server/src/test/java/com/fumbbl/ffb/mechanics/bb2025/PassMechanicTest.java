package com.fumbbl.ffb.mechanics.bb2025;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/bb2025/pass_mechanic.rs tests.
 * BB2025: no-PA is always a fumble even with Safe Pass; below-PA is inaccurate (not a fumble).
 */
public class PassMechanicTest {

	private final PassMechanic m = new PassMechanic();

	private RosterPlayer thrower(int passing) {
		RosterPlayer p = new RosterPlayer();
		p.setId("t");
		p.setPassing(passing);
		return p;
	}

	private Skill safePass() {
		Game game = GameFixture.createGameState().getGame();
		return (Skill) game.getFactory(FactoryType.Factory.SKILL).forName("Safe Pass");
	}

	// rust: roll_one_without_safe_pass_is_fumble
	@Test
	public void rollOneWithoutSafePassIsFumble() {
		assertEquals(PassResult.FUMBLE,
			m.evaluatePass(thrower(4), 1, PassingDistance.SHORT_PASS, Collections.emptyList(), false));
	}

	// rust: roll_one_with_safe_pass_is_saved_fumble
	@Test
	public void rollOneWithSafePassIsSavedFumble() {
		RosterPlayer t = thrower(4);
		t.addSkill(safePass());
		assertEquals(PassResult.SAVED_FUMBLE,
			m.evaluatePass(t, 1, PassingDistance.SHORT_PASS, Collections.emptyList(), false));
	}

	// rust: no_passing_ability_is_always_fumble_regardless_of_safe_pass
	@Test
	public void noPassingAbilityIsAlwaysFumbleRegardlessOfSafePass() {
		RosterPlayer t = thrower(-1);
		t.addSkill(safePass());
		assertEquals(PassResult.FUMBLE,
			m.evaluatePass(t, 3, PassingDistance.SHORT_PASS, Collections.emptyList(), false));
	}

	// rust: normal_roll_above_pa_is_accurate
	@Test
	public void normalRollAbovePaIsAccurate() {
		assertEquals(PassResult.ACCURATE,
			m.evaluatePass(thrower(4), 5, PassingDistance.SHORT_PASS, Collections.emptyList(), false));
	}

	// rust: roll_below_pa_is_inaccurate
	@Test
	public void rollBelowPaIsInaccurate() {
		assertEquals(PassResult.INACCURATE,
			m.evaluatePass(thrower(4), 3, PassingDistance.SHORT_PASS, Collections.emptyList(), false));
	}
}
