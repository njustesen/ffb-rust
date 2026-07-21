package com.fumbbl.ffb.server.step.game.end;

import com.fumbbl.ffb.GameStatus;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.commands.ClientCommandStartGame;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.StepAction;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirrors the Rust unit tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/game/end/step_end_game.rs}
 * ({@code StepEndGame}).
 *
 * <p>Rust sets {@code game.status = Finished} and returns {@code NextStep}; Java
 * keeps status on the {@link GameState} and additionally records a finished
 * timestamp. Replay/upload branches are skipped because {@code game.isTesting()}
 * is true in the fixture.
 */
public class StepEndGameFixtureTest {

	private GameState gameState;
	private Game game;
	private StepEndGame step;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = new StepEndGame(gameState);
	}

	// Rust: start_sets_game_finished_and_returns_next_step
	@Test
	public void startSetsGameFinishedAndReturnsNextStep() {
		StepAction action = GameFixture.startStep(step);
		assertEquals(StepAction.NEXT_STEP, action);
		assertEquals(GameStatus.FINISHED, gameState.getStatus());
		assertNotNull(game.getFinished());
	}

	// Rust: start_initial_status_is_starting
	@Test
	public void startInitialStatusIsStarting() {
		assertEquals(GameStatus.STARTING, gameState.getStatus());
	}

	// Rust: handle_command_returns_continue
	@Test
	public void handleCommandReturnsContinue() {
		GameFixture.handleCommand(step, new ClientCommandStartGame(), true);
		assertEquals(StepAction.CONTINUE, GameFixture.nextAction(step));
	}
}
