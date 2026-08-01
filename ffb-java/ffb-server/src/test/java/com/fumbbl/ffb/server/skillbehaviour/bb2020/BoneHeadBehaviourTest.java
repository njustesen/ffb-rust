package com.fumbbl.ffb.server.skillbehaviour.bb2020;

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
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2020/bone_head_behaviour.rs tests
 * (portable subset — Rust registry/applies_to plumbing tests are exempt). The behaviour is
 * exercised through a real StepBoneHead: the fixture registers all skill behaviours, so
 * step.start() runs the BoneHeadBehaviour execute hook.
 */
public class BoneHeadBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Bone Head"));
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

	private boolean confused(String playerId) {
		return game.getFieldModel().getPlayerState(game.getPlayerById(playerId)).isConfused();
	}

	// rust: step_modifier_no_negatraits_gives_next
	@Test
	public void noNegatraitsTurnModeGivesNextStep() {
		GameFixture.setTurnMode(gameState, TurnMode.KICKOFF_RETURN);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		StepBoneHead step = createStep();
		step.start();
		assertEquals(StepAction.NEXT_STEP, step.getResult().getNextAction());
		assertFalse(confused("home1"));
	}

	// rust: already_used_this_action_skips_second_roll_silently
	@Test
	public void alreadyUsedSkillSkipsSecondRollSilently() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		game.getActingPlayer().markSkillUsed(GameFixture.skill(game, "Bone Head"));
		StepBoneHead step = createStep();
		step.start();
		assertEquals(StepAction.NEXT_STEP, step.getResult().getNextAction());
		assertFalse(confused("home1"));
	}

	// rust: successful roll path (roll >= minimumRollConfusion(true) == 2)
	@Test
	public void successfulRollGivesNextStep() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		GameFixture.installScriptedDice(gameState, 2);
		StepBoneHead step = createStep();
		step.start();
		assertEquals(StepAction.NEXT_STEP, step.getResult().getNextAction());
		assertFalse(confused("home1"));
	}

	// rust: failed roll confuses the player and jumps to the failure label
	@Test
	public void failedRollConfusesPlayerAndGoesToFailureLabel() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		GameFixture.installScriptedDice(gameState, 1);
		StepBoneHead step = createStep();
		step.start();
		assertEquals(StepAction.GOTO_LABEL, step.getResult().getNextAction());
		assertEquals("failLabel", step.getResult().getNextActionParameter());
		assertTrue(confused("home1"));
	}

	// rust: cancel_bb2020_ttm_sets_pass_used_not_ttm_used
	@Test
	public void failedThrowTeamMateActionSetsPassUsed() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.THROW_TEAM_MATE);
		GameFixture.installScriptedDice(gameState, 1);
		StepBoneHead step = createStep();
		step.start();
		assertTrue(game.getTurnDataHome().isPassUsed());
	}
}
