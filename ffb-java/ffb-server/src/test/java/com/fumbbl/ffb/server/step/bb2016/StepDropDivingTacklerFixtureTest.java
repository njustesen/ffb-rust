package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_drop_diving_tackler.rs} (state subset).
 * COORDINATE_FROM / USING_DIVING_TACKLE are stored via setParameter. The step always clears the
 * defender id; when diving tackle is used it moves the defender to COORDINATE_FROM and drops it prone.
 * The does_not_publish_a_stray_player_id test inspects published parameters and is deferred.
 */
public class StepDropDivingTacklerFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DROP_DIVING_TACKLER);
	}

	private int defenderState() {
		Player<?> def = gameState.getGame().getPlayerById("away1");
		return gameState.getGame().getFieldModel().getPlayerState(def).getBase();
	}

	// rust: without_diving_tackle_clears_defender
	@Test
	public void withoutDivingTackleClearsDefender() {
		GameFixture.startStep(newStep());
		assertNull(gameState.getGame().getDefenderId());
		assertEquals(PlayerState.STANDING, defenderState());
	}

	// rust: with_diving_tackle_drops_defender
	@Test
	public void withDivingTackleDropsDefender() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.USING_DIVING_TACKLE, true));
		GameFixture.startStep(step);
		assertEquals(PlayerState.PRONE, defenderState());
		assertNull(gameState.getGame().getDefenderId());
	}

	// rust: coordinate_from_moves_defender
	@Test
	public void coordinateFromMovesDefender() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.USING_DIVING_TACKLE, true));
		step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(10, 7)));
		GameFixture.startStep(step);
		Game game = gameState.getGame();
		assertEquals(new FieldCoordinate(10, 7), game.getFieldModel().getPlayerCoordinate(game.getPlayerById("away1")));
	}

	// rust: no_defender_clears_defender_id
	@Test
	public void noDefenderClearsDefenderId() {
		gameState.getGame().setDefenderId(null);
		GameFixture.startStep(newStep());
		assertNull(gameState.getGame().getDefenderId());
	}
}
