package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.bb2025.block.StepPushback;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Stack;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/monstrous_mouth_behaviour.rs
 * tests (portable subset — registry plumbing exempt). BB2025 Monstrous Mouth: a CHOMPED
 * defender is force-pushed (chain cleared); a chomped ball-carrier's strip is prevented.
 */
public class MonstrousMouthBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 5, 6);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
	}

	private StepPushback.StepState newState() {
		StepPushback.StepState state = new StepPushback.StepState();
		state.defender = game.getPlayerById("away1");
		state.standingFirm = new HashMap<>();
		state.sideStepping = new HashMap<>();
		state.pushbackStack = new Stack<>();
		state.oldDefenderState = game.getFieldModel().getPlayerState(state.defender);
		return state;
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private boolean executeHook(StepPushback step, StepPushback.StepState state) {
		MonstrousMouthBehaviour behaviour = new MonstrousMouthBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2025.MonstrousMouth) GameFixture.skill(game, "Monstrous Mouth");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		return modifier.handleExecuteStepHook(step, state);
	}

	private void chompDefender() {
		PlayerState ps = game.getFieldModel().getPlayerState(game.getPlayerById("away1"));
		game.getFieldModel().setPlayerState(game.getPlayerById("away1"), ps.changeChomped(true));
	}

	// rust: not_chomped_returns_false
	@Test
	public void notChompedReturnsFalse() {
		StepPushback.StepState state = newState();
		assertFalse(executeHook(new StepPushback(gameState), state));
		assertFalse(state.doPush);
	}

	// rust: chomped_defender_forces_push
	@Test
	public void chompedDefenderForcesPush() {
		chompDefender();
		StepPushback.StepState state = newState();
		assertTrue(executeHook(new StepPushback(gameState), state));
		assertTrue(state.doPush);
		assertTrue(state.pushbackStack.isEmpty());
	}

	// rust: chomped_defender_with_ball_prevents_strip_and_reports
	@Test
	public void chompedDefenderWithBallPreventsStripAndReports() {
		chompDefender();
		game.getFieldModel().setBallCoordinate(game.getFieldModel().getPlayerCoordinate(game.getPlayerById("away1")));
		game.getFieldModel().setBallInPlay(true);
		game.getFieldModel().setBallMoving(false);
		StepPushback step = new StepPushback(gameState);
		StepPushback.StepState state = newState();
		assertTrue(executeHook(step, state));
		assertTrue(step.getResult().getReportList().size() > 0);
	}
}
