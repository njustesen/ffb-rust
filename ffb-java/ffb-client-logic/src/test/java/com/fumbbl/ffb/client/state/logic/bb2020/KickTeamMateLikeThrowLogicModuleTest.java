package com.fumbbl.ffb.client.state.logic.bb2020;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.ClientData;
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
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

/**
 * Ported from
 * {@code ffb-rust/crates/ffb-client/src/client/state/logic/bb2020/kick_team_mate_like_throw_logic_module.rs}
 * against the real bb2020 {@link KickTeamMateLikeThrowLogicModule} (extends MoveLogicModule).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code can_be_kicked_false_without_game} / {@code player_interaction_ignores_without_game}
 * (Rust no-game short-circuits); {@code find_kickable_players_empty_without_kickable_teammates}
 * and {@code show_grid_for_ktm_false_without_player} (both route through the TTM GAME-mechanic
 * factory / findAdjacentCoordinates over a live field — fixture-inexpressible).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class KickTeamMateLikeThrowLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	MoveLogicPlugin moveLogicPlugin;

	@Mock
	ActingPlayer actingPlayer;

	@Mock
	FieldModel fieldModel;

	@Mock
	ClientCommunication communication;

	@Mock
	ClientData clientData;

	@SuppressWarnings("rawtypes")
	@Mock
	Player defender;

	@SuppressWarnings("rawtypes")
	@Mock
	Player thrower;

	private KickTeamMateLikeThrowLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(client.getCommunication()).thenReturn(communication);
		when(client.getClientData()).thenReturn(clientData);
		module = new KickTeamMateLikeThrowLogicModule(client);
	}

	// rust: available_actions_matches_move_logic_module
	@Test
	void availableActionsMatchesMoveLogicModule() {
		when(moveLogicPlugin.availableActions()).thenReturn(Collections.emptySet());
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
	}

	// rust: find_kickable_players_none_with_defender
	@Test
	void findKickablePlayersNoneWithDefender() {
		when(game.getDefender()).thenReturn(defender);
		assertNull(module.findKickablePlayers(game, thrower));
	}

	// rust: field_peek_preview_throw_when_defender_present
	@Test
	void fieldPeekPreviewThrowWhenDefenderPresent() {
		when(game.getDefender()).thenReturn(defender);
		when(game.getPassCoordinate()).thenReturn(null);
		InteractionResult result = module.fieldPeek(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.PREVIEW_THROW, result.getKind());
	}

	// rust: field_interaction_delegates_when_ktm_move
	@Test
	void fieldInteractionDelegatesWhenKtmMove() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.KICK_TEAM_MATE_MOVE);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.DELEGATE, result.getKind());
		assertEquals(ClientStateId.MOVE, result.getDelegate());
	}
}
