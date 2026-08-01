package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
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
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/eye_gouge_behaviour.rs tests
 * (portable subset — registry plumbing exempt). Eye Gouge (canRemoveOpponentAssists) marks the
 * MAIN defender eye-gouged during a push; the hook always returns false (never stops the chain).
 */
public class EyeGougeBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 5, 6);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		game.setDefenderId("away1");
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
	private boolean executeHook(StepPushback.StepState state) {
		StepPushback step = new StepPushback(gameState);
		EyeGougeBehaviour behaviour = new EyeGougeBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2025.EyeGouge) GameFixture.skill(game, "Eye Gouge");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		return modifier.handleExecuteStepHook(step, state);
	}

	private void addEyeGouge() {
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Eye Gouge"));
	}

	private boolean gouged() {
		return game.getFieldModel().getPlayerState(game.getPlayerById("away1")).isEyeGouged();
	}

	// rust: no_eye_gouge_property_returns_false
	@Test
	public void noEyeGougePropertyReturnsFalse() {
		assertFalse(executeHook(newState()));
		assertFalse(gouged());
	}

	// rust: not_main_defender_returns_false_and_does_not_gouge
	@Test
	public void notMainDefenderReturnsFalseAndDoesNotGouge() {
		addEyeGouge();
		game.setDefenderId("otherId");
		assertFalse(executeHook(newState()));
		assertFalse(gouged());
	}

	// rust: main_defender_with_gouge_property_gets_gouged_and_reports
	@Test
	public void mainDefenderWithGougePropertyGetsGougedAndReports() {
		addEyeGouge();
		StepPushback step = new StepPushback(gameState);
		EyeGougeBehaviour behaviour = new EyeGougeBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2025.EyeGouge) GameFixture.skill(game, "Eye Gouge");
		@SuppressWarnings({"unchecked", "rawtypes"})
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		boolean result = modifier.handleExecuteStepHook(step, newState());
		assertFalse(result, "Eye Gouge never stops hook processing");
		assertTrue(gouged());
		assertTrue(step.getResult().getReportList().size() > 0);
	}
}
