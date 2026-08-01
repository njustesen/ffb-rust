package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.dialog.DialogReRollParameter;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_dialog.rs tests. showDialog stores the
 * dialog parameter (and sets waitingForOpponent when the turn timer is stopped); hideDialog clears
 * both.
 */
public class UtilServerDialogTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
	}

	// rust: show_dialog_sets_dialog_id
	@Test
	public void showDialogSetsDialogParameter() {
		DialogReRollParameter param = new DialogReRollParameter();
		UtilServerDialog.showDialog(gameState, param, false);
		assertSame(param, game.getDialogParameter());
	}

	// rust: show_dialog_with_stop_timer_sets_waiting_for_opponent
	@Test
	public void showDialogWithStopTimerSetsWaitingForOpponent() {
		UtilServerDialog.showDialog(gameState, new DialogReRollParameter(), true);
		assertTrue(game.isWaitingForOpponent());
	}

	// rust: show_dialog_without_stop_timer_leaves_waiting_for_opponent_unchanged
	@Test
	public void showDialogWithoutStopTimerLeavesWaitingUnchanged() {
		UtilServerDialog.showDialog(gameState, new DialogReRollParameter(), false);
		assertFalse(game.isWaitingForOpponent());
	}

	// rust: hide_dialog_clears_dialog_id_and_waiting
	@Test
	public void hideDialogClearsDialogAndWaiting() {
		UtilServerDialog.showDialog(gameState, new DialogReRollParameter(), true);
		UtilServerDialog.hideDialog(gameState);
		assertNull(game.getDialogParameter());
		assertFalse(game.isWaitingForOpponent());
	}

	// rust: hide_dialog_idempotent_when_already_hidden
	@Test
	public void hideDialogIdempotentWhenAlreadyHidden() {
		UtilServerDialog.hideDialog(gameState);
		assertNull(game.getDialogParameter());
		assertFalse(game.isWaitingForOpponent());
	}

	// rust: show_then_hide_round_trips
	@Test
	public void showThenHideRoundTrips() {
		UtilServerDialog.showDialog(gameState, new DialogReRollParameter(), false);
		assertTrue(game.getDialogParameter() != null);
		UtilServerDialog.hideDialog(gameState);
		assertNull(game.getDialogParameter());
	}
}
