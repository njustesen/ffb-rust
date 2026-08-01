package com.fumbbl.ffb.server.skillbehaviour.bb2016;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import com.fumbbl.ffb.server.step.StepParameterSet;
import com.fumbbl.ffb.server.step.action.common.StepReallyStupid;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2016/really_stupid_behaviour.rs tests
 * (portable subset — registry/flag-helper plumbing exempt).
 */
public class ReallyStupidBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Really Stupid"));
		game.getTurnDataHome().setReRolls(0);
		game.getTurnDataAway().setReRolls(0);
	}

	private StepReallyStupid createStep() {
		StepReallyStupid step = new StepReallyStupid(gameState);
		StepParameterSet params = new StepParameterSet();
		params.add(new StepParameter(StepParameterKey.GOTO_LABEL_ON_FAILURE, "failLabel"));
		step.init(params);
		return step;
	}

	// rust: step_modifier_no_negatraits_gives_next
	@Test
	public void noNegatraitsTurnModeGivesNextStep() {
		GameFixture.setTurnMode(gameState, TurnMode.KICKOFF_RETURN);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		StepReallyStupid step = createStep();
		step.start();
		assertEquals(StepAction.NEXT_STEP, step.getResult().getNextAction());
		assertFalse(game.getFieldModel().getPlayerState(game.getPlayerById("home1")).isConfused());
	}

	// rust: cancel_bb2016_ktm_sets_blitz_used_not_ktm_used
	@Test
	public void cancelKtmSetsBlitzUsedNotKtmUsed() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.KICK_TEAM_MATE);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertTrue(game.getTurnDataHome().isBlitzUsed());
		assertFalse(game.getTurnDataHome().isKtmUsed());
	}

	// rust: cancel_bb2016_ttm_sets_pass_used_not_ttm_used
	@Test
	public void cancelTtmSetsPassUsed() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.THROW_TEAM_MATE);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertTrue(game.getTurnDataHome().isPassUsed());
	}

	// rust: cancel_bb2016_hand_over_sets_hand_over_used
	@Test
	public void cancelHandOverSetsHandOverUsed() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.HAND_OVER);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertTrue(game.getTurnDataHome().isHandOverUsed());
	}
}
