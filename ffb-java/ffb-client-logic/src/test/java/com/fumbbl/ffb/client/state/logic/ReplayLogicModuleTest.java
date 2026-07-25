package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.client.ActionKey;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.ActingPlayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

/**
 * Partial port of the {@code #[cfg(test)]} module in replay_logic_module.rs (ffb-rust crate
 * ffb-client/src/client/state/logic/replay_logic_module.rs).
 *
 * Only the two tests that exercise pure, collaborator-independent behavior are ported here:
 * {@code replayStopped} (a bare {@code client.getMode()} + key comparison) and the
 * {@code actionContext} UnsupportedOperationException.
 *
 * The other seven Rust tests are a structural divergence and remain Rust-only: the Rust module
 * asserts on its private {@code replay_list} field (length after setUp / handleCommand /
 * replayMode), which the Java {@code ReplayLogicModule} does not expose — its equivalent
 * behavior is only observable as interactions on {@code ClientReplayer}, {@code getCommunication()},
 * {@code getOverlays()}, {@code getClientData()} and the {@code ReplayCallbacks} collaborators
 * (setUp/handleCommand/evaluateControl/replayMode), so a faithful 1:1 would be Mockito
 * {@code verify()} assertions of a different shape than the Rust state checks (and getOverlays()
 * .stream() NPEs under deep stubs). {@code isOnline} is a documented Rust stub-gap (always false);
 * the Java delegate would only re-assert a deep-stub default. These are left to the replay
 * infra pass; see docs/TEST_PARITY_PORT.md.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ReplayLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ReplayLogicModule module;

	@BeforeEach
	void setUp() {
		module = new ReplayLogicModule(client);
	}

	// rust: replay_stopped_true_only_for_spectator_mode_and_menu_replay_key
	@Test
	void replayStoppedTrueOnlyForSpectatorModeAndMenuReplayKey() {
		given(client.getMode()).willReturn(ClientMode.SPECTATOR);
		assertTrue(module.replayStopped(ActionKey.MENU_REPLAY));
		assertFalse(module.replayStopped(ActionKey.TOOLBAR_TURN_END));

		given(client.getMode()).willReturn(ClientMode.PLAYER);
		assertFalse(module.replayStopped(ActionKey.MENU_REPLAY));
	}

	// rust: action_context_panics
	@Test
	void actionContextThrows() {
		assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(new ActingPlayer(null)));
	}
}
