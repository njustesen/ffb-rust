package com.fumbbl.ffb.server.skillbehaviour.bb2020;

import com.fumbbl.ffb.ReRolledActions;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.bb2020.shared.StepCatchScatterThrowIn;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2020/catch_behaviour.rs tests
 * (portable subset — registry plumbing exempt). Catch grants a catch reroll.
 */
public class CatchBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private boolean executeHook(StepCatchScatterThrowIn step, StepCatchScatterThrowIn.StepState state) {
		CatchBehaviour behaviour = new CatchBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.common.Catch) GameFixture.skill(game, "Catch");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		return modifier.handleExecuteStepHook(step, state);
	}

	// rust: catcher_without_catch_returns_false
	@Test
	public void catcherWithoutCatchReturnsFalse() {
		StepCatchScatterThrowIn step = new StepCatchScatterThrowIn(gameState);
		StepCatchScatterThrowIn.StepState state = new StepCatchScatterThrowIn.StepState();
		state.catcher = game.getPlayerById("home1");
		assertFalse(executeHook(step, state));
		assertFalse(state.rerollCatch);
	}

	// rust: catcher_with_catch_grants_reroll
	@Test
	public void catcherWithCatchGrantsReroll() {
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Catch"));
		StepCatchScatterThrowIn step = new StepCatchScatterThrowIn(gameState);
		StepCatchScatterThrowIn.StepState state = new StepCatchScatterThrowIn.StepState();
		state.catcher = game.getPlayerById("home1");
		assertTrue(executeHook(step, state));
		assertTrue(state.rerollCatch);
		assertEquals(ReRolledActions.CATCH, step.getReRolledAction());
	}
}
