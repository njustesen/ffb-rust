package com.fumbbl.ffb.mechanics.bb2016;

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
 * Mirror of ffb-rust crates/ffb-mechanics/src/bb2016/pass_mechanic.rs tests.
 * evaluatePass short-circuits on roll 6/1 and isModifiedFumble (roll+distance), so a bare
 * RosterPlayer suffices; the Safe Pass skill (grants dontDropFumbles) comes from a GameFixture game.
 */
public class PassMechanicTest {

	private final PassMechanic m = new PassMechanic();

	private RosterPlayer thrower() {
		RosterPlayer p = new RosterPlayer();
		p.setId("t");
		return p;
	}

	private Skill skill(String name) {
		Game game = GameFixture.createGameState().getGame();
		return (Skill) game.getFactory(FactoryType.Factory.SKILL).forName(name);
	}

	// rust: natural_one_without_safe_pass_is_fumble
	@Test
	public void naturalOneWithoutSafePassIsFumble() {
		assertEquals(PassResult.FUMBLE,
			m.evaluatePass(thrower(), 1, PassingDistance.SHORT_PASS, Collections.emptyList(), false));
	}

	// rust: modified_fumble_with_safe_pass_is_saved_fumble
	@Test
	public void modifiedFumbleWithSafePassIsSavedFumble() {
		RosterPlayer t = thrower();
		t.addSkill(skill("Safe Pass"));
		assertEquals(PassResult.SAVED_FUMBLE,
			m.evaluatePass(t, 2, PassingDistance.LONG_PASS, Collections.emptyList(), false));
	}

	// rust: modified_fumble_without_safe_pass_is_fumble
	@Test
	public void modifiedFumbleWithoutSafePassIsFumble() {
		assertEquals(PassResult.FUMBLE,
			m.evaluatePass(thrower(), 2, PassingDistance.LONG_PASS, Collections.emptyList(), false));
	}

	// rust: bomb_action_modified_fumble_is_fumble_even_with_safe_pass
	@Test
	public void bombActionModifiedFumbleIsFumbleEvenWithSafePass() {
		RosterPlayer t = thrower();
		t.addSkill(skill("Safe Pass"));
		assertEquals(PassResult.FUMBLE,
			m.evaluatePass(t, 2, PassingDistance.LONG_PASS, Collections.emptyList(), true));
	}

	// rust: high_roll_is_accurate
	@Test
	public void highRollIsAccurate() {
		assertEquals(PassResult.ACCURATE,
			m.evaluatePass(thrower(), 6, PassingDistance.SHORT_PASS, Collections.emptyList(), false));
	}
}
