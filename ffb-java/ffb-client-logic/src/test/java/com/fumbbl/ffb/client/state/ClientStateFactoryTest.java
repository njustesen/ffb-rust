package com.fumbbl.ffb.client.state;

import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PushbackSquare;
import com.fumbbl.ffb.SpecialEffect;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.LogicModule;
import com.fumbbl.ffb.client.state.logic.interaction.ActionContext;
import com.fumbbl.ffb.factory.INamedObjectFactory;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.TtmMechanic;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.property.NamedProperties;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Collections;
import java.util.Date;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.mockito.Mockito.when;

/**
 * Port of the {@code #[cfg(test)]} module in
 * {@code ffb-rust/crates/ffb-client/src/client/state/client_state_factory.rs}, against the real
 * {@link ClientStateFactory#getStateForGame()} switch logic.
 *
 * <p>The Rust engine extracted the {@code ClientStateId} computation into a returnable value; the
 * Java {@code getStateForGame()} instead returns the registered {@code ClientState} object via
 * {@code getStateForId(id)}. The concrete {@code ClientStateFactory} subclasses (and real
 * {@code ClientState} instances) live in the AWT {@code ffb-client} module, which is not part of
 * this mirror and is not headless-buildable. So this test stands up a minimal test-local concrete
 * factory that registers one lightweight stub {@code ClientState} per {@link ClientStateId} (each
 * carrying its id through a stub {@link LogicModule}), and asserts on
 * {@code factory.getStateForGame().getId()} — exercising the exact production switch.
 *
 * <p>The Rust {@code get_state_for_id_and_register_are_documented_no_ops} test is not ported: it
 * asserts that the Rust factory's {@code register*}/{@code get_state_for_id} stubs are no-ops,
 * which is Rust-only plumbing (the real Java members are not no-ops). It was pruned on the Rust
 * side to keep the suites 1:1.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientStateFactoryTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private Game game;

	@Mock
	private TtmMechanic ttmMechanic;

	private ClientStateFactory<FantasyFootballClient> factory;

	/** Stub logic module: carries a fixed {@link ClientStateId}; all other members are inert. */
	private static final class StubLogicModule extends LogicModule {
		private final ClientStateId id;

		StubLogicModule(FantasyFootballClient client, ClientStateId id) {
			super(client);
			this.id = id;
		}

		@Override
		public ClientStateId getId() {
			return id;
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
		}
	}

	/** Stub state: {@code getId()} (final) delegates to the stub logic module's id. */
	private static final class StubState extends ClientState<LogicModule, FantasyFootballClient> {
		StubState(FantasyFootballClient client, LogicModule logicModule) {
			super(client, logicModule);
		}

		@Override
		protected void drawSelectSquare() {
		}
	}

	private static final class TestFactory extends ClientStateFactory<FantasyFootballClient> {
		TestFactory(FantasyFootballClient client) {
			super(client);
		}

		@Override
		public void registerStates() {
			for (ClientStateId id : ClientStateId.values()) {
				register(new StubState(getClient(), new StubLogicModule(getClient(), id)));
			}
		}

		@Override
		public void registerStatesForRules() {
		}
	}

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(client.getMode()).thenReturn(ClientMode.PLAYER);
		when(game.getTeamHome().getName()).thenReturn("Home");
		when(game.isHomePlaying()).thenReturn(true);
		// RETURNS_DEEP_STUBS hands back a non-null mock for Date-returning getFinished(); force the
		// "not finished" default so we reach the turn-mode switch (individual tests override).
		when(game.getFinished()).thenReturn(null);
		factory = new TestFactory(client);
	}

	private ClientStateId stateFor(TurnMode turnMode) {
		when(game.getTurnMode()).thenReturn(turnMode);
		ClientState<? extends LogicModule, FantasyFootballClient> state = factory.getStateForGame();
		return state == null ? null : state.getId();
	}

	private ClientStateId stateForAction(PlayerAction action) {
		when(game.getTurnMode()).thenReturn(TurnMode.REGULAR);
		when(game.getActingPlayer().getPlayerAction()).thenReturn(action);
		ClientState<? extends LogicModule, FantasyFootballClient> state = factory.getStateForGame();
		return state == null ? null : state.getId();
	}

	// rust: replay_mode_returns_replay
	@Test
	void replayModeReturnsReplay() {
		when(client.getMode()).thenReturn(ClientMode.REPLAY);
		assertEquals(ClientStateId.REPLAY, stateFor(TurnMode.REGULAR));
	}

	// rust: missing_team_home_name_returns_login
	@Test
	void missingTeamHomeNameReturnsLogin() {
		when(game.getTeamHome().getName()).thenReturn("");
		assertEquals(ClientStateId.LOGIN, stateFor(TurnMode.REGULAR));
	}

	// rust: spectator_mode_returns_spectate
	@Test
	void spectatorModeReturnsSpectate() {
		when(client.getMode()).thenReturn(ClientMode.SPECTATOR);
		assertEquals(ClientStateId.SPECTATE, stateFor(TurnMode.REGULAR));
	}

	// rust: finished_game_returns_spectate
	@Test
	void finishedGameReturnsSpectate() {
		when(game.getFinished()).thenReturn(new Date());
		assertEquals(ClientStateId.SPECTATE, stateFor(TurnMode.REGULAR));
	}

	// rust: home_playing_and_waiting_for_opponent_returns_wait_for_opponent
	@Test
	void homePlayingAndWaitingForOpponentReturnsWaitForOpponent() {
		when(game.isHomePlaying()).thenReturn(true);
		when(game.isWaitingForOpponent()).thenReturn(true);
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, stateFor(TurnMode.REGULAR));
	}

	// rust: hit_and_run_home_playing
	@Test
	void hitAndRunHomePlaying() {
		assertEquals(ClientStateId.HIT_AND_RUN, stateFor(TurnMode.HIT_AND_RUN));
	}

	// rust: hit_and_run_opponent_playing
	@Test
	void hitAndRunOpponentPlaying() {
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, stateFor(TurnMode.HIT_AND_RUN));
	}

	// rust: select_blitz_target_home_playing
	@Test
	void selectBlitzTargetHomePlaying() {
		assertEquals(ClientStateId.SELECT_BLITZ_TARGET, stateFor(TurnMode.SELECT_BLITZ_TARGET));
	}

	// rust: select_gaze_target_home_playing
	@Test
	void selectGazeTargetHomePlaying() {
		assertEquals(ClientStateId.SELECT_GAZE_TARGET, stateFor(TurnMode.SELECT_GAZE_TARGET));
	}

	// rust: regular_no_acting_player_selects_player
	@Test
	void regularNoActingPlayerSelectsPlayer() {
		when(game.getActingPlayer().getPlayer()).thenReturn(null);
		assertEquals(ClientStateId.SELECT_PLAYER, stateFor(TurnMode.REGULAR));
	}

	// rust: regular_with_pushback_squares_returns_pushback
	@Test
	void regularWithPushbackSquaresReturnsPushback() {
		when(game.getFieldModel().getPushbackSquares()).thenReturn(new PushbackSquare[1]);
		assertEquals(ClientStateId.PUSHBACK, stateFor(TurnMode.REGULAR));
	}

	// rust: player_action_move_variants
	@Test
	void playerActionMoveVariants() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.MOVE, PlayerAction.STAND_UP,
			PlayerAction.STAND_UP_BLITZ, PlayerAction.SECURE_THE_BALL}) {
			assertEquals(ClientStateId.MOVE, stateForAction(action), action.name());
		}
	}

	// rust: player_action_blitz_move_is_blitz
	@Test
	void playerActionBlitzMoveIsBlitz() {
		assertEquals(ClientStateId.BLITZ, stateForAction(PlayerAction.BLITZ_MOVE));
	}

	// rust: player_action_block_family_is_block
	@Test
	void playerActionBlockFamilyIsBlock() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.BREATHE_FIRE, PlayerAction.BLITZ,
			PlayerAction.BLOCK, PlayerAction.CHAINSAW, PlayerAction.PROJECTILE_VOMIT, PlayerAction.STAB,
			PlayerAction.VICIOUS_VINES, PlayerAction.CHOMP}) {
			assertEquals(ClientStateId.BLOCK, stateForAction(action), action.name());
		}
	}

	// rust: player_action_multiple_block_without_skill_is_block
	@Test
	void playerActionMultipleBlockWithoutSkillIsBlock() {
		when(game.getActingPlayer().getPlayer().hasSkillProperty(NamedProperties.canBlockTwoAtOnce))
			.thenReturn(false);
		assertEquals(ClientStateId.BLOCK, stateForAction(PlayerAction.MULTIPLE_BLOCK));
	}

	// rust: player_action_multiple_block_with_skill_is_synchronous
	@Test
	void playerActionMultipleBlockWithSkillIsSynchronous() {
		when(game.getActingPlayer().getPlayer().hasSkillProperty(NamedProperties.canBlockTwoAtOnce))
			.thenReturn(true);
		assertEquals(ClientStateId.SYNCHRONOUS_MULTI_BLOCK, stateForAction(PlayerAction.MULTIPLE_BLOCK));
	}

	// rust: player_action_foul_family
	@Test
	void playerActionFoulFamily() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.FOUL, PlayerAction.FOUL_MOVE}) {
			assertEquals(ClientStateId.FOUL, stateForAction(action), action.name());
		}
	}

	// rust: player_action_hand_over_family
	@Test
	void playerActionHandOverFamily() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.HAND_OVER, PlayerAction.HAND_OVER_MOVE}) {
			assertEquals(ClientStateId.HAND_OVER, stateForAction(action), action.name());
		}
	}

	// rust: player_action_pass_family
	@Test
	void playerActionPassFamily() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.PASS, PlayerAction.PASS_MOVE,
			PlayerAction.HAIL_MARY_PASS}) {
			assertEquals(ClientStateId.PASS, stateForAction(action), action.name());
		}
	}

	// rust: player_action_punt_family
	@Test
	void playerActionPuntFamily() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.PUNT, PlayerAction.PUNT_MOVE}) {
			assertEquals(ClientStateId.PUNT, stateForAction(action), action.name());
		}
	}

	// rust: player_action_throw_team_mate_family
	@Test
	void playerActionThrowTeamMateFamily() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.THROW_TEAM_MATE,
			PlayerAction.THROW_TEAM_MATE_MOVE}) {
			assertEquals(ClientStateId.THROW_TEAM_MATE, stateForAction(action), action.name());
		}
	}

	// rust: player_action_kick_team_mate_family
	@Test
	void playerActionKickTeamMateFamily() {
		INamedObjectFactory factory = game.getFactory(FactoryType.Factory.MECHANIC);
		when(factory.forName(Mechanic.Type.TTM.name())).thenReturn(ttmMechanic);
		when(ttmMechanic.handleKickLikeThrow()).thenReturn(true);
		for (PlayerAction action : new PlayerAction[]{PlayerAction.KICK_TEAM_MATE,
			PlayerAction.KICK_TEAM_MATE_MOVE}) {
			assertEquals(ClientStateId.KICK_TEAM_MATE_THROW, stateForAction(action), action.name());
		}
	}

	// rust: player_action_swoop
	@Test
	void playerActionSwoop() {
		assertEquals(ClientStateId.SWOOP, stateForAction(PlayerAction.SWOOP));
	}

	// rust: player_action_gaze
	@Test
	void playerActionGaze() {
		assertEquals(ClientStateId.GAZE, stateForAction(PlayerAction.GAZE));
	}

	// rust: player_action_bomb_family
	@Test
	void playerActionBombFamily() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.THROW_BOMB, PlayerAction.HAIL_MARY_BOMB}) {
			assertEquals(ClientStateId.BOMB, stateForAction(action), action.name());
		}
	}

	// rust: player_action_gaze_move
	@Test
	void playerActionGazeMove() {
		assertEquals(ClientStateId.GAZE_MOVE, stateForAction(PlayerAction.GAZE_MOVE));
	}

	// rust: player_action_throw_keg
	@Test
	void playerActionThrowKeg() {
		assertEquals(ClientStateId.THROW_KEG, stateForAction(PlayerAction.THROW_KEG));
	}

	// rust: player_action_maximum_carnage
	@Test
	void playerActionMaximumCarnage() {
		assertEquals(ClientStateId.MAXIMUM_CARNAGE, stateForAction(PlayerAction.MAXIMUM_CARNAGE));
	}

	// rust: player_action_putrid_regurgitation_blitz_family
	@Test
	void playerActionPutridRegurgitationBlitzFamily() {
		for (PlayerAction action : new PlayerAction[]{PlayerAction.PUTRID_REGURGITATION_BLITZ,
			PlayerAction.PUTRID_REGURGITATION_MOVE}) {
			assertEquals(ClientStateId.PUTRID_REGURGITATION_BLITZ, stateForAction(action), action.name());
		}
	}

	// rust: player_action_putrid_regurgitation_block
	@Test
	void playerActionPutridRegurgitationBlock() {
		assertEquals(ClientStateId.PUTRID_REGURGITATION_BLOCK,
			stateForAction(PlayerAction.PUTRID_REGURGITATION_BLOCK));
	}

	// rust: player_action_kick_em_blitz
	@Test
	void playerActionKickEmBlitz() {
		assertEquals(ClientStateId.KICK_EM_BLITZ, stateForAction(PlayerAction.KICK_EM_BLITZ));
	}

	// rust: player_action_kick_em_block
	@Test
	void playerActionKickEmBlock() {
		assertEquals(ClientStateId.KICK_EM_BLOCK, stateForAction(PlayerAction.KICK_EM_BLOCK));
	}

	// rust: player_action_the_flashing_blade_is_stab
	@Test
	void playerActionTheFlashingBladeIsStab() {
		assertEquals(ClientStateId.STAB, stateForAction(PlayerAction.THE_FLASHING_BLADE));
	}

	// rust: player_action_furious_outburst (Java enum constant is misspelled FURIOUS_OUTPBURST)
	@Test
	void playerActionFuriousOutburst() {
		assertEquals(ClientStateId.FURIOUS_OUTBURST, stateForAction(PlayerAction.FURIOUS_OUTPBURST));
	}

	// rust: player_action_unmatched_returns_none
	@Test
	void playerActionUnmatchedReturnsNone() {
		assertNull(stateForAction(PlayerAction.DUMP_OFF));
	}

	// rust: opponent_playing_regular_finds_passive_state
	@Test
	void opponentPlayingRegularFindsPassiveState() {
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, stateFor(TurnMode.REGULAR));
	}

	// rust: opponent_playing_with_pushback_and_waiting_finds_pushback
	@Test
	void opponentPlayingWithPushbackAndWaitingFindsPushback() {
		when(game.isHomePlaying()).thenReturn(false);
		when(game.isWaitingForOpponent()).thenReturn(true);
		when(game.getFieldModel().getPushbackSquares()).thenReturn(new PushbackSquare[1]);
		assertEquals(ClientStateId.PUSHBACK, stateFor(TurnMode.REGULAR));
	}

	// rust: kickoff_home_and_away
	@Test
	void kickoffHomeAndAway() {
		assertEquals(ClientStateId.KICKOFF, stateFor(TurnMode.KICKOFF));
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, stateFor(TurnMode.KICKOFF));
	}

	// rust: kickoff_return
	@Test
	void kickoffReturn() {
		assertEquals(ClientStateId.KICKOFF_RETURN, stateFor(TurnMode.KICKOFF_RETURN));
	}

	// rust: swarming
	@Test
	void swarming() {
		assertEquals(ClientStateId.SWARMING, stateFor(TurnMode.SWARMING));
	}

	// rust: pass_block
	@Test
	void passBlock() {
		assertEquals(ClientStateId.PASS_BLOCK, stateFor(TurnMode.PASS_BLOCK));
	}

	// rust: start_game_is_unconditional
	@Test
	void startGameIsUnconditional() {
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.START_GAME, stateFor(TurnMode.START_GAME));
	}

	// rust: setup_and_perfect_defence_share_a_branch
	@Test
	void setupAndPerfectDefenceShareABranch() {
		for (TurnMode turnMode : new TurnMode[]{TurnMode.SETUP, TurnMode.PERFECT_DEFENCE}) {
			when(game.isHomePlaying()).thenReturn(true);
			assertEquals(ClientStateId.SETUP, stateFor(turnMode), turnMode.name());
			when(game.isHomePlaying()).thenReturn(false);
			assertEquals(ClientStateId.WAIT_FOR_SETUP, stateFor(turnMode), turnMode.name());
		}
	}

	// rust: solid_defence
	@Test
	void solidDefence() {
		assertEquals(ClientStateId.SOLID_DEFENCE, stateFor(TurnMode.SOLID_DEFENCE));
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.WAIT_FOR_SETUP, stateFor(TurnMode.SOLID_DEFENCE));
	}

	// rust: high_kick
	@Test
	void highKick() {
		assertEquals(ClientStateId.HIGH_KICK, stateFor(TurnMode.HIGH_KICK));
	}

	// rust: quick_snap
	@Test
	void quickSnap() {
		assertEquals(ClientStateId.QUICK_SNAP, stateFor(TurnMode.QUICK_SNAP));
	}

	// rust: illegal_substitution
	@Test
	void illegalSubstitution() {
		assertEquals(ClientStateId.ILLEGAL_SUBSTITUTION, stateFor(TurnMode.ILLEGAL_SUBSTITUTION));
	}

	// rust: touchback_requires_away_playing
	@Test
	void touchbackRequiresAwayPlaying() {
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.TOUCHBACK, stateFor(TurnMode.TOUCHBACK));
		when(game.isHomePlaying()).thenReturn(true);
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, stateFor(TurnMode.TOUCHBACK));
	}

	// rust: interception_away_playing_without_dump_off
	@Test
	void interceptionAwayPlayingWithoutDumpOff() {
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.INTERCEPTION, stateFor(TurnMode.INTERCEPTION));
	}

	// rust: interception_home_playing_with_dump_off
	@Test
	void interceptionHomePlayingWithDumpOff() {
		when(game.getThrowerAction()).thenReturn(PlayerAction.DUMP_OFF);
		assertEquals(ClientStateId.INTERCEPTION, stateFor(TurnMode.INTERCEPTION));
	}

	// rust: interception_home_playing_without_dump_off_waits
	@Test
	void interceptionHomePlayingWithoutDumpOffWaits() {
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, stateFor(TurnMode.INTERCEPTION));
	}

	// rust: dump_off_requires_away_playing
	@Test
	void dumpOffRequiresAwayPlaying() {
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.DUMP_OFF, stateFor(TurnMode.DUMP_OFF));
	}

	// rust: wizard_home_playing
	@Test
	void wizardHomePlaying() {
		assertEquals(ClientStateId.WIZARD, stateFor(TurnMode.WIZARD));
	}

	// rust: wizard_away_playing_with_spell_selected
	@Test
	void wizardAwayPlayingWithSpellSelected() {
		when(game.isHomePlaying()).thenReturn(false);
		when(client.getClientData().getWizardSpell()).thenReturn(SpecialEffect.FIREBALL);
		assertEquals(ClientStateId.WIZARD, stateFor(TurnMode.WIZARD));
	}

	// rust: wizard_away_playing_without_spell_waits
	@Test
	void wizardAwayPlayingWithoutSpellWaits() {
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, stateFor(TurnMode.WIZARD));
	}

	// rust: bomb_turn_modes
	@Test
	void bombTurnModes() {
		for (TurnMode turnMode : new TurnMode[]{TurnMode.BOMB_HOME, TurnMode.BOMB_HOME_BLITZ,
			TurnMode.BOMB_AWAY, TurnMode.BOMB_AWAY_BLITZ}) {
			assertEquals(ClientStateId.BOMB, stateFor(turnMode), turnMode.name());
		}
	}

	// rust: safe_pair_of_hands
	@Test
	void safePairOfHands() {
		assertEquals(ClientStateId.PLACE_BALL, stateFor(TurnMode.SAFE_PAIR_OF_HANDS));
	}

	// rust: raiding_party
	@Test
	void raidingParty() {
		assertEquals(ClientStateId.RAIDING_PARTY, stateFor(TurnMode.RAIDING_PARTY));
	}

	// rust: select_block_kind
	@Test
	void selectBlockKind() {
		assertEquals(ClientStateId.SELECT_BLOCK_KIND, stateFor(TurnMode.SELECT_BLOCK_KIND));
	}

	// rust: trickster
	@Test
	void trickster() {
		assertEquals(ClientStateId.TRICKSTER, stateFor(TurnMode.TRICKSTER));
	}

	// rust: then_i_started_blastin
	@Test
	void thenIStartedBlastin() {
		assertEquals(ClientStateId.THEN_I_STARTED_BLASTIN, stateFor(TurnMode.THEN_I_STARTED_BLASTIN));
	}

	// rust: end_game_is_unconditional
	@Test
	void endGameIsUnconditional() {
		when(game.isHomePlaying()).thenReturn(false);
		assertEquals(ClientStateId.WAIT_FOR_OPPONENT, stateFor(TurnMode.END_GAME));
	}

	// rust: between_turns_and_no_players_to_field_are_unmatched
	@Test
	void betweenTurnsAndNoPlayersToFieldAreUnmatched() {
		assertNull(stateFor(TurnMode.BETWEEN_TURNS));
		assertNull(stateFor(TurnMode.NO_PLAYERS_TO_FIELD));
	}
}
