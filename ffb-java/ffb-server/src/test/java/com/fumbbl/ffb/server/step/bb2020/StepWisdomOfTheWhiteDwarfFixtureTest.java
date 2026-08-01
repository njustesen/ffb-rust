package com.fumbbl.ffb.server.step.bb2020;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/step_wisdom_of_the_white_dwarf.rs
 * (setParameter subset). Wisdom of the White Dwarf grants a chosen skill via player-selection /
 * skill-selection COMMANDS, so the step has no setParameter keys — any key returns false. The
 * start / select-player / select-skill / report tests are command-driven and deferred.
 */
public class StepWisdomOfTheWhiteDwarfFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	// rust: set_parameter_returns_false
	@Test
	public void setParameterReturnsFalse() {
		IStep step = GameFixture.createStep(gameState, StepId.WISDOM_OF_THE_WHITE_DWARF);
		assertFalse(step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
