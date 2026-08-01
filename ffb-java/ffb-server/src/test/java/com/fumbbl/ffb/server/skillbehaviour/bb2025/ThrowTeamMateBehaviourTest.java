package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.bb2025.ttm.StepThrowTeamMate;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/throw_team_mate_behaviour.rs
 * tests (portable subset — registry/applies_to/hook-default plumbing exempt). The behaviour sets
 * hasPassed/throwerId, clears concession, marks TTM/KTM used, then rolls the throw. These twins
 * assert the early deterministic side-effects (the roll path itself is exercised elsewhere).
 */
public class ThrowTeamMateBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 13, 7);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.THROW_TEAM_MATE);
		game.setPassCoordinate(new FieldCoordinate(13, 10));
		game.setConcessionPossible(true);
		GameFixture.installScriptedDice(gameState, 4);
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private void executeHook(StepThrowTeamMate.StepState state) {
		StepThrowTeamMate step = new StepThrowTeamMate(gameState);
		ThrowTeamMateBehaviour behaviour = new ThrowTeamMateBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.mixed.ThrowTeamMate) GameFixture.skill(game, "Throw Team-Mate");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		modifier.handleExecuteStepHook(step, state);
	}

	private StepThrowTeamMate.StepState newState(boolean kicked) {
		StepThrowTeamMate.StepState state = new StepThrowTeamMate.StepState();
		state.kicked = kicked;
		state.thrownPlayerId = "home2";
		return state;
	}

	// rust: handle_execute_step_sets_has_passed_true
	@Test
	public void handleExecuteStepSetsHasPassedTrue() {
		executeHook(newState(false));
		assertTrue(game.getActingPlayer().hasPassed());
	}

	// rust: handle_execute_step_sets_thrower_id
	@Test
	public void handleExecuteStepSetsThrowerId() {
		executeHook(newState(false));
		assertEquals("home1", game.getThrowerId());
	}

	// rust: handle_execute_step_clears_concession_possible
	@Test
	public void handleExecuteStepClearsConcessionPossible() {
		executeHook(newState(false));
		assertFalse(game.isConcessionPossible());
	}

	// rust: handle_execute_step_marks_ttm_used_for_non_kicked
	@Test
	public void handleExecuteStepMarksTtmUsedForNonKicked() {
		executeHook(newState(false));
		assertTrue(game.getTurnDataHome().isTtmUsed());
		assertFalse(game.getTurnDataHome().isKtmUsed());
	}

	// rust: handle_execute_step_marks_ktm_used_for_kicked
	@Test
	public void handleExecuteStepMarksKtmUsedForKicked() {
		executeHook(newState(true));
		assertTrue(game.getTurnDataHome().isKtmUsed());
		assertFalse(game.getTurnDataHome().isTtmUsed());
	}
}
