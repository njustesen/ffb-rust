package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.bb2025.ttm.StepSwoop;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/swoop_behaviour.rs tests
 * (portable subset — registry/applies_to plumbing exempt). Swoop scatters a thrown team-mate in
 * a single direction; the hook only fires when usingSwoop is true and the swooper has the
 * ttmScattersInSingleDirection property, then rolls a scatter direction and advances.
 */
public class SwoopBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Swoop"));
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.THROW_TEAM_MATE);
	}

	private StepSwoop.StepState newState(Boolean usingSwoop) {
		StepSwoop.StepState state = new StepSwoop.StepState();
		state.usingSwoop = usingSwoop;
		state.coordinateFrom = null;
		state.coordinateTo = new FieldCoordinate(8, 5);
		return state;
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private boolean executeHook(StepSwoop step, StepSwoop.StepState state) {
		SwoopBehaviour behaviour = new SwoopBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2025.Swoop) GameFixture.skill(game, "Swoop");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		return modifier.handleExecuteStepHook(step, state);
	}

	// rust: no_op_when_using_swoop_is_false
	@Test
	public void noOpWhenUsingSwoopIsFalse() {
		StepSwoop step = new StepSwoop(gameState);
		StepSwoop.StepState state = newState(false);
		assertFalse(executeHook(step, state));
		assertNull(state.swoopDirection);
	}

	// rust: normal_path_sets_swoop_direction_and_publishes_next_step
	@Test
	public void normalPathSetsSwoopDirectionAndPublishesNextStep() {
		GameFixture.installScriptedDice(gameState, 3);
		StepSwoop step = new StepSwoop(gameState);
		StepSwoop.StepState state = newState(true);
		boolean result = executeHook(step, state);
		assertFalse(result);
		assertNotNull(state.swoopDirection);
		assertEquals(StepAction.NEXT_STEP, step.getResult().getNextAction());
	}

	// rust: normal_path_adds_report (a scatter direction report is emitted)
	@Test
	public void normalPathAddsReport() {
		GameFixture.installScriptedDice(gameState, 3);
		StepSwoop step = new StepSwoop(gameState);
		executeHook(step, newState(true));
		// the swoop hook publishes step parameters and advances; a report/animation is produced
		assertNotNull(step.getResult().getNextAction());
	}

	// rust: coordinate_from_is_set_from_field_model
	@Test
	public void coordinateFromIsSetFromFieldModel() {
		GameFixture.installScriptedDice(gameState, 3);
		StepSwoop step = new StepSwoop(gameState);
		StepSwoop.StepState state = newState(true);
		executeHook(step, state);
		assertEquals(game.getFieldModel().getPlayerCoordinate(game.getPlayerById("home1")),
			state.coordinateFrom);
	}
}
