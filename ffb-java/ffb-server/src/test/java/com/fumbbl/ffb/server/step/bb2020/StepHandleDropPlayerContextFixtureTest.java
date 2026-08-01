package com.fumbbl.ffb.server.step.bb2020;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/step_handle_drop_player_context.rs (guard +
 * param subset). With no drop-player context, start() → NEXT_STEP. SUCCESSFUL_PRO accepted via
 * setParameter; unknown → false. The DROP_PLAYER_CONTEXT param + injury-result publish / drop-player /
 * goto-label / use-skill-command report tests need a built DropPlayerContext with an injury result and
 * are deferred (published-param + command). SUCCESSFUL_PRO also is not a pure param-store — setting it
 * true immediately marks the pro skill used on the acting player (NPE without a playerId+context) — so
 * that setParameter twin is deferred too.
 */
public class StepHandleDropPlayerContextFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.HANDLE_DROP_PLAYER_CONTEXT);
	}

	// rust: no_context_returns_next
	@Test
	public void noContextReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
