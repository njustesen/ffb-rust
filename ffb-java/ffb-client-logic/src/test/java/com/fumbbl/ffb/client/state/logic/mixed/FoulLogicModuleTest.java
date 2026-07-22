package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.util.UtilPlayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/foul_logic_module.rs}
 * against the real mixed {@link FoulLogicModule}.
 *
 * <p>Three Rust tests were pruned rather than ported (kept the suites 1:1):
 * {@code bloodlust_action_context_empty_without_flag} (the private {@code bloodlustActionContext}
 * helper is only ever reached in Java when {@code isSufferingBloodLust()} is true, so the
 * "empty without flag" branch is unreachable/unobservable here); and
 * {@code player_interaction_ignores_without_game} / {@code end_turn_no_op_without_game} (the Rust
 * {@code client.game()?} no-game short-circuits have no Java counterpart — Java dereferences
 * {@code client.getGame()} unconditionally).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class FoulLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock
	private Game game;

	@Mock
	private LogicPluginFactory logicPluginFactory;

	@Mock
	private MoveLogicPlugin moveLogicPlugin;

	@Mock
	private ActingPlayer actingPlayer;

	@Mock
	private FieldModel fieldModel;

	@Mock
	private Team teamAway;

	@Mock
	private ClientCommunication communication;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player actor;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	private FoulLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(game.getTeamAway()).thenReturn(teamAway);
		when(client.getCommunication()).thenReturn(communication);
		when(actingPlayer.getPlayer()).thenReturn(actor);
		module = new FoulLogicModule(client);
	}

	/** Makes {@link #defender} foulable: prone, on the away team, adjacent to the actor, unprotected. */
	private void makeFoulable() {
		when(fieldModel.getPlayerState(defender)).thenReturn(new PlayerState(PlayerState.PRONE));
		when(fieldModel.getPlayerCoordinate(defender)).thenReturn(new FieldCoordinate(1, 1));
		when(fieldModel.getPlayerCoordinate(actor)).thenReturn(new FieldCoordinate(1, 2));
		when(teamAway.hasPlayer(defender)).thenReturn(true);
		when(defender.hasSkillProperty(NamedProperties.preventBeingFouled)).thenReturn(false);
	}

	// rust: available_actions_contains_foul_and_chainsaw
	@Test
	void availableActionsContainsFoulAndChainsaw() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.FOUL));
		assertTrue(actions.contains(ClientAction.CHAINSAW));
		assertEquals(15, actions.size());
	}

	// rust: is_foulable_false_without_adjacency
	@Test
	void isFoulableFalseWithoutAdjacency() {
		when(fieldModel.getPlayerState(defender)).thenReturn(new PlayerState(PlayerState.PRONE));
		when(fieldModel.getPlayerCoordinate(defender)).thenReturn(new FieldCoordinate(10, 10));
		when(fieldModel.getPlayerCoordinate(actor)).thenReturn(new FieldCoordinate(1, 1));
		when(teamAway.hasPlayer(defender)).thenReturn(true);
		assertFalse(UtilPlayer.isFoulable(game, defender));
	}

	// rust: is_foulable_false_for_standing_player
	@Test
	void isFoulableFalseForStandingPlayer() {
		when(fieldModel.getPlayerState(defender)).thenReturn(new PlayerState(PlayerState.STANDING));
		when(fieldModel.getPlayerCoordinate(defender)).thenReturn(new FieldCoordinate(1, 1));
		when(fieldModel.getPlayerCoordinate(actor)).thenReturn(new FieldCoordinate(1, 2));
		when(teamAway.hasPlayer(defender)).thenReturn(true);
		assertFalse(UtilPlayer.isFoulable(game, defender));
	}

	// rust: bloodlust_action_context_adds_move_and_end_move (via playerInteraction on the acting player)
	@Test
	void bloodlustActionContextAddsMoveAndEndMove() {
		when(actingPlayer.isSufferingBloodLust()).thenReturn(true);
		InteractionResult result = module.playerInteraction(actor);
		assertEquals(InteractionResult.Kind.SELECT_ACTION, result.getKind());
		assertTrue(result.getActionContext().getActions().contains(ClientAction.MOVE));
		assertTrue(result.getActionContext().getActions().contains(ClientAction.END_MOVE));
	}

	// rust: foul_action_context_always_adds_foul (via playerSelected with a fouling-alternative skill)
	@Test
	void foulActionContextAlwaysAddsFoul() {
		makeFoulable();
		when(actor.hasSkillProperty(NamedProperties.providesFoulingAlternative)).thenReturn(true);
		when(actor.hasSkillProperty(NamedProperties.providesChainsawFoulingAlternative)).thenReturn(false);
		InteractionResult result = module.playerSelected(defender);
		assertEquals(InteractionResult.Kind.SELECT_ACTION, result.getKind());
		assertTrue(result.getActionContext().getActions().contains(ClientAction.FOUL));
		assertFalse(result.getActionContext().getActions().contains(ClientAction.CHAINSAW));
	}

	// rust: player_peek_ignores_when_not_foulable
	@Test
	void playerPeekIgnoresWhenNotFoulable() {
		when(fieldModel.getPlayerState(defender)).thenReturn(new PlayerState(PlayerState.STANDING));
		InteractionResult result = module.playerPeek(defender);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: perform_available_action_foul_sends_command
	@Test
	void performAvailableActionFoulSendsCommand() {
		when(actingPlayer.getPlayerId()).thenReturn("attacker");
		module.performAvailableAction(defender, ClientAction.FOUL);
		verify(communication).sendFoul("attacker", defender, false);
	}
}
