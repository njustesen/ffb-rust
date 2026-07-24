package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.net.ClientCommunication;
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
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/swoop_logic_module.rs}
 * against the real {@link SwoopLogicModule} (extends MoveLogicModule). All Rust tests port.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SwoopLogicModuleTest {

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

	@SuppressWarnings("rawtypes")
	@Mock
	Player actor;

	@SuppressWarnings("rawtypes")
	@Mock
	Player defender;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private SwoopLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(client.getCommunication()).thenReturn(communication);
		module = new SwoopLogicModule(client);
	}

	// rust: available_actions_matches_move_logic_module
	@Test
	void availableActionsMatchesMoveLogicModule() {
		when(moveLogicPlugin.availableActions()).thenReturn(Collections.emptySet());
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
	}

	// rust: field_interaction_ignores_without_swoop_action
	@Test
	void fieldInteractionIgnoresWithoutSwoopAction() {
		when(actingPlayer.getPlayerAction()).thenReturn(null);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: field_interaction_handled_when_swoop_action_set
	@Test
	void fieldInteractionHandledWhenSwoopActionSet() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.SWOOP);
		when(actingPlayer.getPlayer()).thenReturn(actor);
		when(fieldModel.getPlayerCoordinate(actor)).thenReturn(new FieldCoordinate(5, 5));
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(10, 10));
		assertEquals(InteractionResult.Kind.HANDLED, result.getKind());
	}

	// rust: player_peek_resets_without_defender_or_pass_coordinate
	@Test
	void playerPeekResetsWithoutDefenderOrPassCoordinate() {
		when(game.getDefender()).thenReturn(null);
		when(game.getPassCoordinate()).thenReturn(null);
		InteractionResult result = module.playerPeek(player);
		assertEquals(InteractionResult.Kind.RESET, result.getKind());
	}

	// rust: player_peek_ignores_when_defender_present
	@Test
	void playerPeekIgnoresWhenDefenderPresent() {
		when(game.getDefender()).thenReturn(defender);
		InteractionResult result = module.playerPeek(player);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: player_interaction_ignores_without_swoop_action
	@Test
	void playerInteractionIgnoresWithoutSwoopAction() {
		when(actingPlayer.getPlayerAction()).thenReturn(null);
		InteractionResult result = module.playerInteraction(player);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}
}
