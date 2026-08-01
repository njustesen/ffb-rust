package com.fumbbl.ffb.server.skillbehaviour.bb2016;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
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
import com.fumbbl.ffb.server.step.bb2016.StepWildAnimal;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2016/wild_animal_behaviour.rs tests
 * (portable subset — registry plumbing exempt). Wild Animal failure leaves the player STANDING
 * and inactive (NOT confused, unlike Bone Head / Really Stupid); standing-up players drop PRONE.
 */
public class WildAnimalBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Wild Animal"));
		game.getTurnDataHome().setReRolls(0);
		game.getTurnDataAway().setReRolls(0);
	}

	private StepWildAnimal createStep() {
		StepWildAnimal step = new StepWildAnimal(gameState);
		StepParameterSet params = new StepParameterSet();
		params.add(new StepParameter(StepParameterKey.GOTO_LABEL_ON_FAILURE, "failLabel"));
		step.init(params);
		return step;
	}

	private PlayerState state() {
		return game.getFieldModel().getPlayerState(game.getPlayerById("home1"));
	}

	// rust: step_modifier_no_negatraits_gives_next
	@Test
	public void noNegatraitsTurnModeGivesNextStep() {
		GameFixture.setTurnMode(gameState, TurnMode.KICKOFF_RETURN);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		StepWildAnimal step = createStep();
		step.start();
		assertEquals(StepAction.NEXT_STEP, step.getResult().getNextAction());
	}

	// rust: cancel_wild_animal_sets_standing_not_confused
	@Test
	public void cancelSetsStandingNotConfused() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertEquals(PlayerState.STANDING, state().getBase());
		assertFalse(state().isConfused());
		assertFalse(state().isActive());
	}

	// rust: cancel_wild_animal_ktm_sets_blitz_used
	@Test
	public void cancelKtmSetsBlitzUsed() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.KICK_TEAM_MATE);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertTrue(game.getTurnDataHome().isBlitzUsed());
	}

	// rust: cancel_wild_animal_standing_up_sets_prone
	@Test
	public void cancelStandingUpSetsProne() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		game.getActingPlayer().setStandingUp(true);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertEquals(PlayerState.PRONE, state().getBase());
	}
}
