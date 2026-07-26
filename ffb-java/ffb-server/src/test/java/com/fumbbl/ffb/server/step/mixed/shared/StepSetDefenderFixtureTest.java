package com.fumbbl.ffb.server.step.mixed.shared;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import com.fumbbl.ffb.server.step.StepParameterSet;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/mixed/step_set_defender.rs} (defender-id subset).
 * BLOCK_DEFENDER_ID and GAZE_VICTIM_ID both set the step's defender id, which start() applies to the
 * game (unless the value is null and IGNORE_NULL_VALUE is set). The block-defender-id-not-consumed and
 * ignore-null-value-is-consumed tests exercise the StepParameter consumption mechanism and are
 * deferred.
 */
public class StepSetDefenderFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
	}

	private IStep newStep() {
		// SET_DEFENDER is not registered in the fixture's StepFactory, so instantiate directly.
		return new StepSetDefender(gameState);
	}

	// rust: sets_defender_from_block_defender_id
	@Test
	public void setsDefenderFromBlockDefenderId() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_DEFENDER_ID, "p01"));
		GameFixture.startStep(step);
		assertEquals("p01", gameState.getGame().getDefenderId());
	}

	// rust: sets_defender_from_gaze_victim_id
	@Test
	public void setsDefenderFromGazeVictimId() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.GAZE_VICTIM_ID, "p02"));
		GameFixture.startStep(step);
		assertEquals("p02", gameState.getGame().getDefenderId());
	}

	// rust: clears_defender_when_gaze_victim_is_none_and_not_ignoring_null
	@Test
	public void clearsDefenderWhenGazeVictimNoneAndNotIgnoring() {
		gameState.getGame().setDefenderId("old");
		IStep step = newStep(); // ignoreNullValue defaults false
		step.setParameter(StepParameter.from(StepParameterKey.GAZE_VICTIM_ID, null));
		GameFixture.startStep(step);
		assertNull(gameState.getGame().getDefenderId());
	}

	// rust: preserves_defender_when_null_and_ignore_null_value_set
	@Test
	public void preservesDefenderWhenNullAndIgnoreNullValueSet() {
		gameState.getGame().setDefenderId("old");
		IStep step = newStep();
		// IGNORE_NULL_VALUE is init-consumed (not stored via setParameter), so supply it via init().
		StepParameterSet set = new StepParameterSet();
		set.add(StepParameter.from(StepParameterKey.IGNORE_NULL_VALUE, true));
		step.init(set);
		step.setParameter(StepParameter.from(StepParameterKey.GAZE_VICTIM_ID, null));
		GameFixture.startStep(step);
		assertEquals("old", gameState.getGame().getDefenderId());
	}
}
