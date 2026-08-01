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
import com.fumbbl.ffb.server.step.action.common.StepBoneHead;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2016/bone_head_behaviour.rs tests
 * (portable subset — registry/applies_to plumbing exempt). BB2016 skill name is "Bone-Head";
 * BB2016 cancel maps KICK_TEAM_MATE to blitzUsed.
 */
public class BoneHeadBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Bone-Head"));
		game.getTurnDataHome().setReRolls(0);
		game.getTurnDataAway().setReRolls(0);
	}

	private StepBoneHead createStep() {
		StepBoneHead step = new StepBoneHead(gameState);
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
		StepBoneHead step = createStep();
		step.start();
		assertEquals(StepAction.NEXT_STEP, step.getResult().getNextAction());
		assertFalse(game.getFieldModel().getPlayerState(game.getPlayerById("home1")).isConfused());
	}

	// rust: cancel_bb2016_ktm_sets_blitz_used
	@Test
	public void cancelKtmSetsBlitzUsed() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.KICK_TEAM_MATE);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertTrue(game.getTurnDataHome().isBlitzUsed());
	}

	// rust: cancel_bb2016_ttm_sets_pass_used
	@Test
	public void cancelTtmSetsPassUsed() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.THROW_TEAM_MATE);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertTrue(game.getTurnDataHome().isPassUsed());
	}

	// rust: cancel_bb2016_foul_sets_foul_used
	@Test
	public void cancelFoulSetsFoulUsed() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.FOUL);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertTrue(game.getTurnDataHome().isFoulUsed());
	}
}
