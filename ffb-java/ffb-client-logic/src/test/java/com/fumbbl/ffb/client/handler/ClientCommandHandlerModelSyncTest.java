package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.SoundId;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.Animation;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.change.ModelChange;
import com.fumbbl.ffb.model.change.ModelChangeId;
import com.fumbbl.ffb.model.change.ModelChangeList;
import com.fumbbl.ffb.net.commands.ServerCommandModelSync;
import com.fumbbl.ffb.report.ReportList;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.anyBoolean;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.Mockito.RETURNS_DEEP_STUBS;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;

/**
 * 1:1 port of the Rust {@code client_command_handler_model_sync.rs} test module.
 *
 * <p>The Rust module tests several free/inherent helper functions
 * ({@code should_sync_clock}, {@code find_updates}, {@code has_block_choice_report},
 * {@code should_wait_for_animation}, {@code prepare_for_animation}/{@code restore_after_animation})
 * that were invented in the Rust translation because it lacks a live {@code Game}/UI. Java has no
 * equivalent standalone methods -- the same logic is inlined in
 * {@link ClientCommandHandlerModelSync#handleNetCommand} and {@code animationFinished()}. Where the
 * inlined logic can be exercised safely through {@code handleNetCommand} with deep-stub mocks (no
 * NPEs, no dependence on un-mockable production code), these tests reframe the Rust scenario as a
 * behavioral test of the real handler, verifying the Java-observable side effect that corresponds to
 * the Rust assertion. Each handler/client pair is built fresh per scenario (rather than shared
 * {@code @Mock} fields) so independent {@code verify()} calls don't cross-contaminate.</p>
 */
class ClientCommandHandlerModelSyncTest {

	private static FantasyFootballClient newClient() {
		FantasyFootballClient client = mock(FantasyFootballClient.class, RETURNS_DEEP_STUBS);
		// ModelChangeList.applyTo(game) unconditionally resolves these two factories via the
		// generic game.getFactory(...), which a deep stub returns as an INamedObjectFactory mock
		// that fails the implicit checkcast; force concrete mocks (doReturn on the root game mock).
		com.fumbbl.ffb.model.Game game = client.getGame();
		org.mockito.Mockito.doReturn(mock(com.fumbbl.ffb.factory.SkillFactory.class))
			.when(game).getFactory(com.fumbbl.ffb.FactoryType.Factory.SKILL);
		org.mockito.Mockito.doReturn(mock(com.fumbbl.ffb.factory.PrayerFactory.class))
			.when(game).getFactory(com.fumbbl.ffb.FactoryType.Factory.PRAYER);
		return client;
	}

	@Test
	void shouldSyncClockTrueForQueuingAndPlaying() {
		// QUEUING: Java syncs the clock and short-circuits.
		FantasyFootballClient queuingClient = newClient();
		Game queuingGame = queuingClient.getGame();
		ClientCommandHandlerModelSync queuingHandler = new ClientCommandHandlerModelSync(queuingClient);
		ServerCommandModelSync queuingCmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(), null, null, 42L, 7L);
		assertTrue(queuingHandler.handleNetCommand(queuingCmd, ClientCommandHandlerMode.QUEUING));
		verify(queuingGame).setGameTime(42L);
		verify(queuingGame).setTurnTime(7L);

		// PLAYING: Java also syncs the clock, then continues (and still returns true since there's
		// no animation to wait for).
		FantasyFootballClient playingClient = newClient();
		Game playingGame = playingClient.getGame();
		ClientCommandHandlerModelSync playingHandler = new ClientCommandHandlerModelSync(playingClient);
		ServerCommandModelSync playingCmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(), null, null, 42L, 7L);
		assertTrue(playingHandler.handleNetCommand(playingCmd, ClientCommandHandlerMode.PLAYING));
		verify(playingGame).setGameTime(42L);
		verify(playingGame).setTurnTime(7L);

		// REPLAYING: Java does NOT sync the clock.
		FantasyFootballClient replayingClient = newClient();
		Game replayingGame = replayingClient.getGame();
		ClientCommandHandlerModelSync replayingHandler = new ClientCommandHandlerModelSync(replayingClient);
		ServerCommandModelSync replayingCmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(), null, null, 42L, 7L);
		assertTrue(replayingHandler.handleNetCommand(replayingCmd, ClientCommandHandlerMode.REPLAYING));
		verify(replayingGame, never()).setGameTime(anyLong());
		verify(replayingGame, never()).setTurnTime(anyLong());
	}

	@Test
	void findUpdatesDetectsActingPlayerChange() {
		FantasyFootballClient client = newClient();
		ClientCommandHandlerModelSync handler = new ClientCommandHandlerModelSync(client);

		ModelChangeList changes = new ModelChangeList();
		changes.add(new ModelChange(ModelChangeId.ACTING_PLAYER_SET_HAS_MOVED, null, Boolean.TRUE));
		ServerCommandModelSync cmd = new ServerCommandModelSync(changes, new ReportList(), null, null, 0, 0);

		handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING);

		// Java-observable equivalent of `updates.update_acting_player`: the
		// updateUserinterface() branch gated on fUpdateActingPlayer.
		verify(client.getClientData()).setActingPlayerUpdated(true);
		verify(client.getUserInterface().getFieldComponent().getLayerUnderPlayers()).clearMovePath();
	}

	@Test
	void findUpdatesDetectsTurnNrAndTurnMode() {
		FantasyFootballClient client = newClient();
		ClientCommandHandlerModelSync handler = new ClientCommandHandlerModelSync(client);

		ModelChangeList changes = new ModelChangeList();
		changes.add(new ModelChange(ModelChangeId.TURN_DATA_SET_TURN_NR, ModelChange.HOME, 3));
		changes.add(new ModelChange(ModelChangeId.GAME_SET_TURN_MODE, null, TurnMode.KICKOFF));
		ServerCommandModelSync cmd = new ServerCommandModelSync(changes, new ReportList(), null, null, 0, 0);

		handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING);

		// Java-observable equivalent of `updates.update_turn_nr || updates.update_turn_mode`: the
		// updateUserinterface() branch `if (fUpdateTurnNr || (fUpdateTurnMode && ...)) clientData.clear();`.
		verify(client.getClientData()).clear();
	}

	@Test
	void findUpdatesIgnoresUnrelatedChanges() {
		FantasyFootballClient client = newClient();
		ClientCommandHandlerModelSync handler = new ClientCommandHandlerModelSync(client);

		ModelChangeList changes = new ModelChangeList();
		changes.add(new ModelChange(ModelChangeId.FIELD_MODEL_SET_BALL_COORDINATE, null, null));
		ServerCommandModelSync cmd = new ServerCommandModelSync(changes, new ReportList(), null, null, 0, 0);

		handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING);

		verify(client.getClientData(), never()).setActingPlayerUpdated(anyBoolean());
		verify(client.getClientData(), never()).clear();
	}

	@Test
	void hasBlockChoiceReportFalseForEmptyList() {
		FantasyFootballClient client = newClient();
		ClientCommandHandlerModelSync handler = new ClientCommandHandlerModelSync(client);

		ServerCommandModelSync cmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(), null, null, 0, 0);
		handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING);

		// Java-observable equivalent of `has_block_choice_report` being false: handleExtraEffects()
		// never finds a BLOCK_CHOICE report, so it never populates the client's block-dice result.
		verify(client.getClientData(), never()).setBlockDiceResult(anyList());
	}

	// Rust: should_wait_for_animation_true_when_playing
	// Rust: should_wait_for_animation_respects_replaying_single_speed_flag
	// Rust: prepare_and_restore_pass_animation_round_trips_ball_coordinate
	// Rust: prepare_and_restore_bomb_animation_round_trips_bomb_coordinate
	// SKIPPED: `should_wait_for_animation`/`prepare_for_animation`/`restore_after_animation` are
	// Rust-invented pure helpers with no standalone Java method; in Java this logic only runs when
	// handleNetCommand's `waitForAnimation` is true, which drives production
	// AnimationSequenceFactory/AnimationSequenceThrowing code (real Swing Timer, AnimationProjector,
	// coordinate-based stepping) that isn't reproducible through a mockable seam without either a
	// compiler to verify against or inventing coordinates/icon data the Rust test never needed.

	@Test
	void shouldWaitForAnimationFalseWithoutAnimation() {
		FantasyFootballClient client = newClient();
		ClientCommandHandlerModelSync handler = new ClientCommandHandlerModelSync(client);

		ServerCommandModelSync cmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(), null, null, 0, 0);

		// No animation -> waitForAnimation is false -> handleNetCommand returns true immediately.
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));
	}

	@Test
	void handleNetCommandShortCircuitsWhenQueuing() {
		FantasyFootballClient client = newClient();
		ClientCommandHandlerModelSync handler = new ClientCommandHandlerModelSync(client);

		ServerCommandModelSync cmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(),
			new Animation(), SoundId.TOUCHDOWN, 0, 0);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.QUEUING));
	}

	// Rust: handle_net_command_is_a_no_op_for_a_mismatched_command_type
	// SKIPPED: Java casts unconditionally; wrong type throws CCE, not a no-op.
}
