package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.interaction.ActionContext;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Port of the {@code #[cfg(test)]} module in logic_module.rs (ffb-rust crate
 * ffb-client/src/client/state/logic/logic_module.rs).
 *
 * Only the handful of Rust tests that exercise genuinely pure predicate methods (no
 * {@code client.getGame()} access) are ported here. The vast majority of Rust tests in that
 * module exercise free functions taking a fully-built {@code &Game}/{@code &Player} graph
 * directly; the corresponding Java {@code LogicModule} methods instead read
 * {@code client.getGame()} (often unconditionally, before any null-guard), which would require
 * either building a live {@code Game}/{@code FieldModel}/factory object graph or brittle
 * deep-stub mocking of factory/mechanic lookups that risks {@code ClassCastException} at
 * runtime. Per the porting brief those are skipped; see the final report for the itemized list.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class LogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private LogicModule logicModule;

	/**
	 * Minimal concrete subclass — {@code LogicModule} is abstract with three abstract members
	 * unrelated to the predicate methods under test here; they are stubbed trivially since none
	 * of the ported tests exercise them.
	 */
	private static final class TestLogicModule extends LogicModule {
		TestLogicModule(FantasyFootballClient client) {
			super(client);
		}

		@Override
		public ClientStateId getId() {
			return ClientStateId.MOVE;
		}

		@Override
		public Set<ClientAction> availableActions() {
			return Collections.emptySet();
		}

		@Override
		protected ActionContext actionContext(ActingPlayer actingPlayer) {
			return null;
		}

		@Override
		protected void performAvailableAction(Player<?> player, ClientAction action) {
			// no-op, unused by the ported tests
		}
	}

	@BeforeEach
	void setUp() {
		logicModule = new TestLogicModule(client);
	}

	// rust: is_putrid_regurgitation_available_is_always_false
	@Test
	void isPutridRegurgitationAvailableIsAlwaysFalse() {
		assertFalse(logicModule.isPutridRegurgitationAvailable());
	}

	// rust: is_move_available_matches_gaze_action_only
	@Test
	void isMoveAvailableMatchesGazeActionOnly() {
		ActingPlayer actingPlayer = new ActingPlayer(null);

		actingPlayer.setPlayerAction(PlayerAction.GAZE);
		assertTrue(logicModule.isMoveAvailable(actingPlayer));

		actingPlayer.setPlayerAction(PlayerAction.MOVE);
		assertFalse(logicModule.isMoveAvailable(actingPlayer));
	}

	// rust: is_frenzied_rush_available_false_without_skill
	@Test
	void isFrenziedRushAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isFrenziedRushAvailable(player));
	}

	// rust: is_slashing_nails_available_false_without_skill
	@Test
	void isSlashingNailsAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isSlashingNailsAvailable(player));
	}

	// rust: is_incorporeal_available_false_without_skill
	@Test
	void isIncorporealAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isIncorporealAvailable(player));
	}
}
