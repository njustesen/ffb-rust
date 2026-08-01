package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.bb2025.shared.StepDropFallingPlayers;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/saboteur_behaviour.rs tests
 * (portable subset — registry/applies_to plumbing and no-acting-player guard exempt). A falling
 * attacker with Saboteur (canSabotageBlockerOnKnockdown) during a block may roll to knock the
 * defender down too; on 4+ the defender is set FALLING.
 */
public class SaboteurBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		game.setDefenderId("away1");
		fallAttacker();
	}

	private void fallAttacker() {
		PlayerState ps = game.getFieldModel().getPlayerState(game.getPlayerById("home1"));
		game.getFieldModel().setPlayerState(game.getPlayerById("home1"), ps.changeBase(PlayerState.FALLING));
	}

	private void addSaboteur() {
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Saboteur"));
	}

	private StepDropFallingPlayers.StepState newState(Boolean usingAttacker) {
		StepDropFallingPlayers.StepState state = new StepDropFallingPlayers.StepState();
		state.usingSaboteurAttacker = usingAttacker;
		state.oldDefenderState = game.getFieldModel().getPlayerState(game.getPlayerById("away1"));
		return state;
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private boolean executeHook(StepDropFallingPlayers step, StepDropFallingPlayers.StepState state) {
		SaboteurBehaviour behaviour = new SaboteurBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2025.Saboteur) GameFixture.skill(game, "Saboteur");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		return modifier.handleExecuteStepHook(step, state);
	}

	private boolean defenderFalling() {
		return game.getFieldModel().getPlayerState(game.getPlayerById("away1")).getBase() == PlayerState.FALLING;
	}

	// rust: returns_false_when_no_defender
	@Test
	public void returnsFalseWhenNoDefender() {
		addSaboteur();
		game.setDefenderId(null);
		assertFalse(executeHook(new StepDropFallingPlayers(gameState), newState(true)));
	}

	// rust: attacker_without_saboteur_skill_returns_false
	@Test
	public void attackerWithoutSaboteurSkillReturnsFalse() {
		assertFalse(executeHook(new StepDropFallingPlayers(gameState), newState(true)));
	}

	// rust: attacker_saboteur_not_block_action_returns_false
	@Test
	public void attackerSaboteurNotBlockActionReturnsFalse() {
		addSaboteur();
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		fallAttacker();
		assertFalse(executeHook(new StepDropFallingPlayers(gameState), newState(true)));
	}

	// rust: attacker_saboteur_accepted_roll_report_added
	@Test
	public void attackerSaboteurAcceptedRollReportAdded() {
		addSaboteur();
		GameFixture.installScriptedDice(gameState, 3);
		StepDropFallingPlayers step = new StepDropFallingPlayers(gameState);
		executeHook(step, newState(true));
		assertTrue(step.getResult().getReportList().size() > 0);
	}

	// rust: attacker_saboteur_declined_no_roll
	@Test
	public void attackerSaboteurDeclinedNoRoll() {
		addSaboteur();
		StepDropFallingPlayers step = new StepDropFallingPlayers(gameState);
		executeHook(step, newState(false));
		assertEquals(0, step.getResult().getReportList().size());
		assertFalse(defenderFalling());
	}

	// rust: attacker_saboteur_success_triggers_defender_falling
	@Test
	public void attackerSaboteurSuccessTriggersDefenderFalling() {
		addSaboteur();
		GameFixture.installScriptedDice(gameState, 6);
		executeHook(new StepDropFallingPlayers(gameState), newState(true));
		assertTrue(defenderFalling());
	}
}
