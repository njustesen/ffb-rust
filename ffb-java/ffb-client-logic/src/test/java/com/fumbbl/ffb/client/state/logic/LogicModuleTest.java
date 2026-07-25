package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.TurnMode;
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
import static org.mockito.BDDMockito.given;

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

	// The following predicates read client.getGame() but short-circuit to false on the
	// skill/used-flag gate before touching any live game state, so a plain (skill-less) RosterPlayer
	// plus the deep-stub client reproduces the Rust "false without skill / when used" case faithfully.
	// (Methods that evaluate an unconditional local before the boolean gate remain skipped: a
	// (GameMechanic) cast — isBlockActionAvailable, isFoulActionAvailable, isViciousVinesAvailable,
	// isThrowBombActionAvailable, isAllYouCanEatAvailable — ClassCastExceptions under deep stubs; and
	// isSecureTheBallActionAvailable computes a UtilPlayer tacklezone sweep over getTeamAway() that
	// NPEs on the deep-stub null player array. See the class doc.)

	// rust: is_treacherous_available_false_without_skill
	@Test
	void isTreacherousAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isTreacherousAvailable(player));
	}

	// rust: is_black_ink_available_false_without_skill
	@Test
	void isBlackInkAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isBlackInkAvailable(player));
	}

	// rust: is_raiding_party_available_false_without_skill
	@Test
	void isRaidingPartyAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isRaidingPartyAvailable(player));
	}

	// rust: is_look_into_my_eyes_available_false_without_skill
	@Test
	void isLookIntoMyEyesAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isLookIntoMyEyesAvailable(player));
	}

	// rust: is_baleful_hex_available_false_without_skill
	@Test
	void isBalefulHexAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isBalefulHexAvailable(player));
	}

	// rust: is_then_i_started_blastin_available_false_without_skill
	@Test
	void isThenIStartedBlastinAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isThenIStartedBlastinAvailable(player));
	}

	// rust: is_chomp_available_false_without_pin_players_skill
	@Test
	void isChompAvailableFalseWithoutPinPlayersSkill() {
		RosterPlayer player = new RosterPlayer();
		RosterPlayer target = new RosterPlayer();
		assertFalse(logicModule.isChompAvailable(player, target));
	}

	// rust: is_punt_action_available_false_without_skill
	@Test
	void isPuntActionAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		assertFalse(logicModule.isPuntActionAvailable(player, false));
	}

	// rust: is_blitz_action_available_false_when_blitz_used
	@Test
	void isBlitzActionAvailableFalseWhenBlitzUsed() {
		RosterPlayer player = new RosterPlayer();
		given(client.getGame().getTurnData().isBlitzUsed()).willReturn(true);
		assertFalse(logicModule.isBlitzActionAvailable(player));
	}

	// rust: is_pass_action_available_false_when_pass_used
	@Test
	void isPassActionAvailableFalseWhenPassUsed() {
		RosterPlayer player = new RosterPlayer();
		given(client.getGame().getTurnData().isPassUsed()).willReturn(true);
		assertFalse(logicModule.isPassActionAvailable(player, false));
	}

	// rust: is_hand_over_action_available_false_when_used
	@Test
	void isHandOverActionAvailableFalseWhenUsed() {
		RosterPlayer player = new RosterPlayer();
		given(client.getGame().getTurnData().isHandOverUsed()).willReturn(true);
		assertFalse(logicModule.isHandOverActionAvailable(player, false));
	}

	// The following predicates gate on live PlayerState/turn-mode terms before the skill check; the
	// deep-stub PlayerState reached via getFieldModel().getPlayerState(player) is stubbed so the
	// skill/state gate is the genuine reason for the result (isolating the Rust intent), without
	// building a live FieldModel graph. Each stub targets the same cached deep-stub instance.

	// rust: is_multi_block_action_available_false_without_skill
	@Test
	void isMultiBlockActionAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		given(client.getGame().getFieldModel().getPlayerState(player).isActive()).willReturn(true);
		assertFalse(logicModule.isMultiBlockActionAvailable(player));
	}

	// rust: is_furious_outburst_available_false_without_skill
	@Test
	void isFuriousOutburstAvailableFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		given(client.getGame().getFieldModel().getPlayerState(player).isActive()).willReturn(true);
		given(client.getGame().getFieldModel().getPlayerState(player).getBase()).willReturn(PlayerState.STANDING);
		assertFalse(logicModule.isFuriousOutburstAvailable(player));
	}

	// rust: is_kick_em_block_and_blitz_false_without_skill
	@Test
	void isKickEmBlockAndBlitzFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		given(client.getGame().getFieldModel().getPlayerState(player).isActive()).willReturn(true);
		assertFalse(logicModule.isKickEmBlockAvailable(player));
		assertFalse(logicModule.isKickEmBlitzAvailable(player));
	}

	// rust: is_beer_barrel_bash_false_without_skill
	@Test
	void isBeerBarrelBashFalseWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		given(client.getGame().getTurnMode()).willReturn(TurnMode.REGULAR);
		given(client.getGame().getFieldModel().getPlayerState(player).getBase()).willReturn(PlayerState.STANDING);
		assertFalse(logicModule.isBeerBarrelBashAvailable(player));
	}

	// rust: is_stand_up_action_available_requires_prone_and_active
	@Test
	void isStandUpActionAvailableRequiresProneAndActive() {
		RosterPlayer player = new RosterPlayer();
		// default (standing) -> false
		assertFalse(logicModule.isStandUpActionAvailable(player));
		// prone + active -> true
		given(client.getGame().getFieldModel().getPlayerState(player).getBase()).willReturn(PlayerState.PRONE);
		given(client.getGame().getFieldModel().getPlayerState(player).isActive()).willReturn(true);
		assertTrue(logicModule.isStandUpActionAvailable(player));
	}

	// rust: is_recover_from_confusion_action_available_requires_confused_state
	@Test
	void isRecoverFromConfusionActionAvailableRequiresConfusedState() {
		RosterPlayer player = new RosterPlayer();
		// not confused -> false
		assertFalse(logicModule.isRecoverFromConfusionActionAvailable(player));
		// confused + active + non-prone -> true
		given(client.getGame().getFieldModel().getPlayerState(player).isConfused()).willReturn(true);
		given(client.getGame().getFieldModel().getPlayerState(player).isActive()).willReturn(true);
		assertTrue(logicModule.isRecoverFromConfusionActionAvailable(player));
	}

	// rust: is_recover_from_gaze_action_available_requires_hypnotized_state
	@Test
	void isRecoverFromGazeActionAvailableRequiresHypnotizedState() {
		RosterPlayer player = new RosterPlayer();
		// not hypnotized -> false
		assertFalse(logicModule.isRecoverFromGazeActionAvailable(player));
		// hypnotized + non-prone -> true
		given(client.getGame().getFieldModel().getPlayerState(player).isHypnotized()).willReturn(true);
		assertTrue(logicModule.isRecoverFromGazeActionAvailable(player));
	}

	// rust: is_recover_from_eye_gouge_action_available_requires_eye_gouged_state
	@Test
	void isRecoverFromEyeGougeActionAvailableRequiresEyeGougedState() {
		RosterPlayer player = new RosterPlayer();
		// not eye-gouged -> false
		assertFalse(logicModule.isRecoverFromEyeGougeActionAvailable(player));
		// eye-gouged + active + non-prone -> true
		given(client.getGame().getFieldModel().getPlayerState(player).isEyeGouged()).willReturn(true);
		given(client.getGame().getFieldModel().getPlayerState(player).isActive()).willReturn(true);
		assertTrue(logicModule.isRecoverFromEyeGougeActionAvailable(player));
	}

	// rust: is_recover_from_confusion_and_stand_up_are_mutually_exclusive_prone_gate
	@Test
	void isRecoverFromConfusionAndStandUpAreMutuallyExclusiveProneGate() {
		RosterPlayer player = new RosterPlayer();
		// prone + active + confused: cannot recover from confusion yet (must stand up first)
		given(client.getGame().getFieldModel().getPlayerState(player).isConfused()).willReturn(true);
		given(client.getGame().getFieldModel().getPlayerState(player).isActive()).willReturn(true);
		given(client.getGame().getFieldModel().getPlayerState(player).getBase()).willReturn(PlayerState.PRONE);
		assertFalse(logicModule.isRecoverFromConfusionActionAvailable(player));
	}
}
