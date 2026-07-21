package com.fumbbl.ffb.server.step.game.start;

import com.fumbbl.ffb.GameStatus;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.commands.ClientCommandCoinChoice;
import com.fumbbl.ffb.net.commands.ClientCommandStartGame;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepCommandStatus;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.lang.reflect.Field;
import java.util.Date;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Proof-of-concept for the {@link GameFixture} test harness. Mirrors the 11
 * Rust unit tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/game/start/step_init_start_game.rs}.
 *
 * <p>Mapping notes (Rust -> Java):
 * <ul>
 *   <li>Rust {@code StepAction::Continue} == Java {@link StepAction#CONTINUE}
 *       (the default of a fresh {@code StepResult}).</li>
 *   <li>Rust tracks "both coaches ready" in the step's own flags; Java stores
 *       it on the model as {@code game.getStarted() != null} (set inside
 *       {@code handleCommand} once both flags are up). The
 *       "start after both coaches already ready" test therefore presets
 *       {@code game.setStarted(...)}, which is the state Java's
 *       {@code executeStep()} actually gates on.</li>
 *   <li>Rust {@code GameStatus::Starting/Active} live on the Game struct; Java
 *       keeps them on the {@link GameState} ({@code getStatus()}).</li>
 * </ul>
 */
public class StepInitStartGameFixtureTest {

	private GameState gameState;
	private Game game;
	private StepInitStartGame step;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = new StepInitStartGame(gameState);
	}

	// Rust: id_is_init_start_game
	@Test
	public void idIsInitStartGame() {
		assertEquals(StepId.INIT_START_GAME, step.getId());
	}

	// Rust: default_fields_are_false
	@Test
	public void defaultFieldsAreFalse() {
		assertFalse(booleanField(step, "fFumbblGameCreated"));
		assertFalse(booleanField(step, "fStartedHome"));
		assertFalse(booleanField(step, "fStartedAway"));
	}

	// Rust: start_without_both_coaches_ready_returns_continue_and_leaves_game_starting
	@Test
	public void startWithoutBothCoachesReadyReturnsContinueAndLeavesGameStarting() {
		// Java: executeStep() only calls leaveStep() once game.getStarted() != null,
		// which requires BOTH coaches' CLIENT_START_GAME. On the initial start()
		// neither has signalled yet, so the step must wait.
		step.start();
		assertEquals(StepAction.CONTINUE, GameFixture.nextAction(step));
		assertEquals(GameStatus.STARTING, gameState.getStatus());
		assertNull(game.getStarted());
	}

	// Rust: start_after_both_coaches_already_ready_sets_game_active_and_returns_next_step
	@Test
	public void startAfterBothCoachesAlreadyReadySetsGameActiveAndReturnsNextStep() {
		// "Both coaches already ready" is game.getStarted() != null in Java.
		game.setStarted(new Date());
		step.start();
		assertEquals(StepAction.NEXT_STEP, GameFixture.nextAction(step));
		assertEquals(GameStatus.ACTIVE, gameState.getStatus());
	}

	// Rust: start_initial_status_is_starting
	@Test
	public void startInitialStatusIsStarting() {
		assertEquals(GameStatus.STARTING, gameState.getStatus());
	}

	// Rust: handle_command_start_game_home_sets_started_home
	@Test
	public void handleCommandStartGameHomeSetsStartedHome() {
		GameFixture.handleCommand(step, new ClientCommandStartGame(), true);
		assertTrue(booleanField(step, "fStartedHome"));
		assertFalse(booleanField(step, "fStartedAway"));
	}

	// Rust: handle_command_start_game_away_sets_started_away
	@Test
	public void handleCommandStartGameAwaySetsStartedAway() {
		GameFixture.handleCommand(step, new ClientCommandStartGame(), false);
		assertFalse(booleanField(step, "fStartedHome"));
		assertTrue(booleanField(step, "fStartedAway"));
	}

	// Rust: handle_command_start_game_from_only_one_coach_does_not_activate_game
	@Test
	public void handleCommandStartGameFromOnlyOneCoachDoesNotActivateGame() {
		// game.setStarted() (and thus leaveStep()) only fires once BOTH coaches
		// are ready. A single coach's ready command must leave the game waiting.
		StepCommandStatus status = GameFixture.handleCommand(step, new ClientCommandStartGame(), true);
		assertEquals(StepCommandStatus.EXECUTE_STEP, status);
		assertEquals(StepAction.CONTINUE, GameFixture.nextAction(step));
		assertEquals(GameStatus.STARTING, gameState.getStatus());
		assertNull(game.getStarted());
	}

	// Rust: handle_command_start_game_from_both_coaches_activates_game
	@Test
	public void handleCommandStartGameFromBothCoachesActivatesGame() {
		GameFixture.handleCommand(step, new ClientCommandStartGame(), true);
		GameFixture.handleCommand(step, new ClientCommandStartGame(), false);
		assertEquals(StepAction.NEXT_STEP, GameFixture.nextAction(step));
		assertEquals(GameStatus.ACTIVE, gameState.getStatus());
		assertNotNull(game.getStarted());
	}

	// Rust: handle_command_unknown_action_returns_continue
	@Test
	public void handleCommandUnknownActionReturnsContinue() {
		// A command this step does not handle (coin choice belongs to StepCoinChoice).
		StepCommandStatus status = GameFixture.handleCommand(step, new ClientCommandCoinChoice(true), true);
		assertEquals(StepCommandStatus.UNHANDLED_COMMAND, status);
		assertEquals(StepAction.CONTINUE, GameFixture.nextAction(step));
		assertEquals(GameStatus.STARTING, gameState.getStatus());
	}

	// Rust: default_creates_same_as_new - Java analogue: the StepFactory
	// constructs an equivalent instance with the same id.
	@Test
	public void stepFactoryCreatesSameAsConstructor() {
		assertEquals(StepId.INIT_START_GAME,
			GameFixture.createStep(gameState, StepId.INIT_START_GAME).getId());
	}

	// ── helpers ──────────────────────────────────────────────────────────────

	private static boolean booleanField(Object target, String fieldName) {
		try {
			Field field = target.getClass().getDeclaredField(fieldName);
			field.setAccessible(true);
			return field.getBoolean(target);
		} catch (ReflectiveOperationException e) {
			throw new IllegalStateException("Cannot read field " + fieldName, e);
		}
	}
}
