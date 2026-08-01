package com.fumbbl.ffb.server.skillbehaviour.bb2020;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PushbackMode;
import com.fumbbl.ffb.PushbackSquare;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.bb2020.block.StepPushback;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Stack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2020/side_step_behaviour.rs tests (portable subset —
 * registry plumbing exempt; Rust headless auto-decline for undecided is a documented
 * divergence, Java shows the skill-use dialog).
 */
public class SideStepBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 10, 10);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private StepPushback.StepState newState() {
		StepPushback.StepState state = new StepPushback.StepState();
		state.defender = game.getPlayerById("away1");
		state.sideStepping = new HashMap<>();
		state.pushbackStack = new Stack<>();
		state.oldDefenderState = game.getFieldModel().getPlayerState(state.defender);
		state.freeSquareAroundDefender = true;
		state.pushbackSquares = new PushbackSquare[0];
		return state;
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private boolean executeHook(StepPushback.StepState state) {
		StepPushback step = new StepPushback(gameState);
		SideStepBehaviour behaviour = new SideStepBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2020.SideStep) GameFixture.skill(game, "Side Step");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		return modifier.handleExecuteStepHook(step, state);
	}

	private void addSideStep() {
		((RosterPlayer) game.getPlayerById("away1")).addSkill(GameFixture.skill(game, "Side Step"));
	}

	// rust: no_side_step_skill_returns_false
	@Test
	public void noSideStepSkillReturnsFalse() {
		assertFalse(executeHook(newState()));
	}

	// rust: side_step_with_no_free_square_returns_false
	@Test
	public void sideStepWithNoFreeSquareReturnsFalse() {
		addSideStep();
		StepPushback.StepState state = newState();
		state.freeSquareAroundDefender = false;
		assertFalse(executeHook(state));
	}

	// rust: side_step_headless_auto_declines (Java variant: shows skill-use dialog)
	@Test
	public void sideStepNotDecidedShowsDialog() {
		addSideStep();
		StepPushback.StepState state = newState();
		assertTrue(executeHook(state));
		assertNotNull(game.getDialogParameter());
	}

	// rust: side_step_declined_in_map_returns_false
	@Test
	public void sideStepDeclinedInMapReturnsFalse() {
		addSideStep();
		StepPushback.StepState state = newState();
		state.sideStepping.put("away1", false);
		assertFalse(executeHook(state));
	}

	// rust: side_step_accepted_switches_to_side_step_mode (also covers
	// side_step_accepted_clears_starting_square — the hook publishes a null starting square)
	@Test
	public void sideStepAcceptedSwitchesToSideStepMode() {
		addSideStep();
		StepPushback.StepState state = newState();
		state.sideStepping.put("away1", true);
		state.startingPushbackSquare = new PushbackSquare(
			game.getFieldModel().getPlayerCoordinate(state.defender), Direction.NORTH, false);
		assertTrue(executeHook(state));
		assertEquals(PushbackMode.SIDE_STEP, state.pushbackMode);
	}

}
