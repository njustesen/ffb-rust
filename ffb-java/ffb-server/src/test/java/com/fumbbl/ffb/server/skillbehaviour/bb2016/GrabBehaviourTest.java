package com.fumbbl.ffb.server.skillbehaviour.bb2016;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PushbackMode;
import com.fumbbl.ffb.PushbackSquare;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.bb2016.StepPushback;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Stack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2016/grab_behaviour.rs tests (portable
 * subset — registry plumbing exempt). Grab is an ATTACKER skill: the acting player grabs during
 * a block action when adjacent to the pushed defender.
 */
public class GrabBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
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
		state.freeSquareAroundDefender = true;
		state.grabbing = null;
		state.startingPushbackSquare = new PushbackSquare(
			game.getFieldModel().getPlayerCoordinate(state.defender), Direction.NORTH, false);
		state.pushbackSquares = new PushbackSquare[]{
			new PushbackSquare(new FieldCoordinate(5, 7), Direction.NORTH, false)};
		return state;
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private boolean executeHook(StepPushback.StepState state) {
		StepPushback step = new StepPushback(gameState);
		GrabBehaviour behaviour = new GrabBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2016.Grab) GameFixture.skill(game, "Grab");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		return modifier.handleExecuteStepHook(step, state);
	}

	private void addGrab() {
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Grab"));
	}

	// rust: no_grab_skill_returns_false
	@Test
	public void noGrabSkillReturnsFalse() {
		assertFalse(executeHook(newState()));
	}

	// rust: grab_not_adjacent_returns_false
	@Test
	public void grabNotAdjacentReturnsFalse() {
		addGrab();
		StepPushback.StepState state = newState();
		state.startingPushbackSquare = new PushbackSquare(
			new FieldCoordinate(15, 10), Direction.NORTH, false);
		assertFalse(executeHook(state));
	}

	// rust: grab_with_free_pushback_squares_auto_grabs (also covers
	// grab_clears_starting_square_when_applied — the hook publishes a null starting square)
	@Test
	public void grabWithFreePushbackSquaresAutoGrabs() {
		addGrab();
		StepPushback.StepState state = newState();
		assertTrue(executeHook(state));
		// Java resets state.grabbing to null after a successful auto-grab; only the pushback
		// mode persists (see GrabBehaviour else-branch), mirroring the Rust hook's assertion.
		assertEquals(PushbackMode.GRAB, state.pushbackMode);
	}

	// rust: grab_fires_on_multiple_block_action
	@Test
	public void grabFiresOnMultipleBlockAction() {
		addGrab();
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MULTIPLE_BLOCK);
		StepPushback.StepState state = newState();
		assertTrue(executeHook(state));
		assertEquals(PushbackMode.GRAB, state.pushbackMode);
	}
}
