package com.fumbbl.ffb.server.step.bb2016.block;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/block/step_both_down.rs}.
 * BOTH_DOWN knocks down any player lacking the Block property; default GameFixture linemen have no
 * Block skill, so both the acting attacker and the defender fall to PlayerState.FALLING. Java
 * dereferences both game.getDefender() and actingPlayer.getPlayer(), so both are placed.
 * <p>
 * EXEMPT: the Rust no_defender_no_panic test — Java's StepBothDown has no null-guard (a both-down
 * resolution always has both an attacker and a defender in production); the missing-defender
 * scenario has no faithful Java path.
 */
public class StepBothDownFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BOTH_DOWN);
	}

	private int stateBase(String playerId) {
		Game game = gameState.getGame();
		return game.getFieldModel().getPlayerState(game.getPlayerById(playerId)).getBase();
	}

	// rust: returns_next_step
	@Test
	public void returnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: defender_without_property_falls
	@Test
	public void defenderWithoutPropertyFalls() {
		GameFixture.startStep(newStep());
		assertEquals(PlayerState.FALLING, stateBase("away1"));
	}

	// rust: attacker_without_property_falls
	@Test
	public void attackerWithoutPropertyFalls() {
		GameFixture.startStep(newStep());
		assertEquals(PlayerState.FALLING, stateBase("home1"));
	}

	// rust: set_parameter_old_defender_state
	@Test
	public void setParameterOldDefenderState() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.PRONE))));
	}
}
