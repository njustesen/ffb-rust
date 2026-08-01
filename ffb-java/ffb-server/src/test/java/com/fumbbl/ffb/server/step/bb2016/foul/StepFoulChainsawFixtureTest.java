package com.fumbbl.ffb.server.step.bb2016.foul;

import com.fumbbl.ffb.PlayerAction;
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

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2016/foul/step_foul_chainsaw.rs (guard subset).
 * A fouling player without the chainsaw property falls straight through to NEXT_STEP. The chainsaw
 * roll / report / USING_CHAINSAW publish tests are dice-driven and deferred; no_acting_player is
 * Rust-defensive (Java derefs getActingPlayer().getPlayer()); GOTO_LABEL_ON_FAILURE is init-consumed
 * (setParameter twin exempt).
 */
public class StepFoulChainsawFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.FOUL);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.FOUL_CHAINSAW);
	}

	// rust: player_without_chainsaw_returns_next
	@Test
	public void playerWithoutChainsawReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
