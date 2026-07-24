package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.ClientData;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.model.ActingPlayer;
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
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/throw_team_mate_logic_module.rs}
 * against the real {@link ThrowTeamMateLogicModule} (extends MoveLogicModule).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code can_be_thrown_false_without_thrower} (Java {@code canBeThrown} is private and casts the
 * TTM GAME-mechanic from the factory); {@code player_peek_sets_selected_player_and_resets_when_not_throwable}
 * (the reset branch needs canBeThrown → the TTM mechanic + adjacency over a live Game); and
 * {@code player_interaction_ignores_without_game} (Rust no-game short-circuit). The defender-present
 * preview-throw branches, the field-interaction delegate, and availableActions port cleanly.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ThrowTeamMateLogicModuleTest {

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
	ClientData clientData;

	@SuppressWarnings("rawtypes")
	@Mock
	Player defender;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private ThrowTeamMateLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(client.getClientData()).thenReturn(clientData);
		module = new ThrowTeamMateLogicModule(client);
	}

	// rust: available_actions_matches_move_logic_module
	@Test
	void availableActionsMatchesMoveLogicModule() {
		when(moveLogicPlugin.availableActions()).thenReturn(Collections.emptySet());
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.END_MOVE));
	}

	// rust: field_interaction_delegates_when_throw_team_mate_move
	@Test
	void fieldInteractionDelegatesWhenThrowTeamMateMove() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.THROW_TEAM_MATE_MOVE);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.DELEGATE, result.getKind());
		assertEquals(ClientStateId.MOVE, result.getDelegate());
	}

	// rust: field_peek_preview_throw_when_defender_present_and_no_pass_coordinate
	@Test
	void fieldPeekPreviewThrowWhenDefenderPresentAndNoPassCoordinate() {
		when(game.getDefender()).thenReturn(defender);
		when(game.getPassCoordinate()).thenReturn(null);
		InteractionResult result = module.fieldPeek(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.PREVIEW_THROW, result.getKind());
	}

	// rust: player_peek_preview_throw_when_defender_present
	@Test
	void playerPeekPreviewThrowWhenDefenderPresent() {
		when(game.getDefender()).thenReturn(defender);
		when(game.getPassCoordinate()).thenReturn(null);
		InteractionResult result = module.playerPeek(player);
		assertEquals(InteractionResult.Kind.PREVIEW_THROW, result.getKind());
	}
}
