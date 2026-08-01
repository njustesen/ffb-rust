package com.fumbbl.ffb.server.step.mixed;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/step_drop_acting_player.rs. start() drops the
 * acting player prone (own-skill injury) when in-bounds and not already stunned, else leaves them;
 * NEXT_STEP. Exempt: no_acting_player_is_noop (Rust-defensive — Java derefs getPlayerCoordinate(null));
 * ball_carrier_triggers_scatter_param (published CATCH_SCATTER_THROW_IN_MODE param — deferred).
 */
public class StepDropActingPlayerFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DROP_ACTING_PLAYER);
	}

	private PlayerState stateOfHome1() {
		return game.getFieldModel().getPlayerState(game.getPlayerById("home1"));
	}

	// rust: standing_acting_player_becomes_prone
	@Test
	public void standingActingPlayerBecomesProne() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertEquals(PlayerState.PRONE, stateOfHome1().getBase());
	}

	// rust: stunned_acting_player_not_dropped
	@Test
	public void stunnedActingPlayerNotDropped() {
		game.getFieldModel().setPlayerState(game.getPlayerById("home1"), new PlayerState(PlayerState.STUNNED));
		GameFixture.startStep(newStep());
		assertEquals(PlayerState.STUNNED, stateOfHome1().getBase());
	}
}
