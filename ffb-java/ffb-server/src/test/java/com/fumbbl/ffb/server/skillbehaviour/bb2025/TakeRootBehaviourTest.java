package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import com.fumbbl.ffb.server.step.StepParameterSet;
import com.fumbbl.ffb.server.step.bb2025.shared.StepTakeRoot;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/take_root_behaviour.rs tests
 * (portable subset — registry plumbing exempt). Take Root only rolls when the player STARTED
 * the action standing and is not already rooted; a failed roll roots the player in place.
 */
public class TakeRootBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.setTurnMode(gameState, TurnMode.REGULAR);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Take Root"));
		game.getTurnDataHome().setReRolls(0);
		game.getTurnDataAway().setReRolls(0);
	}

	private StepTakeRoot createStep() {
		StepTakeRoot step = new StepTakeRoot(gameState);
		StepParameterSet params = new StepParameterSet();
		params.add(new StepParameter(StepParameterKey.GOTO_LABEL_ON_FAILURE, "failLabel"));
		step.init(params);
		return step;
	}

	private Player<?> player() {
		return game.getPlayerById("home1");
	}

	private boolean rooted() {
		return game.getFieldModel().getPlayerState(player()).isRooted();
	}

	// rust: modifier_started_prone_skips_roll
	@Test
	public void startedProneSkipsRoll() {
		PlayerState state = game.getFieldModel().getPlayerState(player());
		game.getFieldModel().setPlayerState(player(), state.changeBase(PlayerState.PRONE));
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		createStep().start();
		assertFalse(rooted());
	}

	// rust: modifier_already_rooted_skips_roll
	@Test
	public void alreadyRootedSkipsRoll() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		PlayerState state = game.getFieldModel().getPlayerState(player());
		game.getFieldModel().setPlayerState(player(), state.changeRooted(true));
		createStep().start();
		assertTrue(rooted());
	}

	// rust: modifier_failed_roll_roots_player
	@Test
	public void failedRollRootsPlayer() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertTrue(rooted());
	}

	// rust: modifier_successful_roll_not_rooted
	@Test
	public void successfulRollNotRooted() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		GameFixture.installScriptedDice(gameState, 6);
		createStep().start();
		assertFalse(rooted());
	}

	// rust: modifier_failed_with_trr_sets_waiting (team reroll available -> reroll dialog is
	// asked and the player is NOT yet rooted)
	@Test
	public void failedWithTeamRerollAsksAndDoesNotRootYet() {
		game.getTurnDataHome().setReRolls(1);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		GameFixture.installScriptedDice(gameState, 1);
		createStep().start();
		assertFalse(rooted());
		assertNotNull(game.getDialogParameter());
	}
}
