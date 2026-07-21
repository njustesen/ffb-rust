package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
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
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/punt_logic_module.rs
// (Rust: mod tests). PuntLogicModule extends MoveLogicModule, whose constructor eagerly resolves
// a MoveLogicPlugin; game/factory are mocked explicitly (not deep-stub cascaded) and wired to a
// real MoveLogicPlugin mock so construction succeeds.
//
// actionAvailable's has_acted/PUNT branches short-circuit before evaluating UtilPlayer.hasBall(...),
// so it is safely testable with a mocked Game/Player/ActingPlayer that never has hasBall invoked.
//
// SKIPPED (with reasons):
// - action_context_always_adds_end_move / action_context_adds_punt_when_punt_move_and_has_ball:
//   `actionContext(ActingPlayer)` calls many isXAvailable helpers plus `UtilPlayer.hasBall`/
//   `UtilPlayer.hasMoveLeft` against a live Game — out of scope.
// - player_interaction_ignores_without_game / perform_available_action_punt_sends_command:
//   Java calls `client.getGame()` unconditionally and dispatches through a live Player/ActingPlayer
//   graph — out of scope.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class PuntLogicModuleTest {

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
	Player<?> player;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
	}

	@Test
	void getIdReturnsPunt() {
		PuntLogicModule module = new PuntLogicModule(client);
		assertEquals(ClientStateId.PUNT, module.getId());
	}

	@Test
	void availableActionsIncludesPunt() {
		when(moveLogicPlugin.availableActions()).thenReturn(Collections.emptySet());
		PuntLogicModule module = new PuntLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.PUNT));
	}

	@Test
	void actionAvailableTrueWhenHasActed() {
		when(actingPlayer.hasActed()).thenReturn(true);
		PuntLogicModule module = new PuntLogicModule(client);
		assertTrue(module.actionAvailable(player, actingPlayer, null, game, null));
	}

	@Test
	void actionAvailableTrueForPuntAction() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.PUNT);
		PuntLogicModule module = new PuntLogicModule(client);
		assertTrue(module.actionAvailable(player, actingPlayer, null, game, null));
	}

	@Test
	void actionAvailableFalseOtherwise() {
		PuntLogicModule module = new PuntLogicModule(client);
		assertFalse(module.actionAvailable(player, actingPlayer, null, game, null));
	}

	@Test
	void playerPeekAlwaysIgnores() {
		PuntLogicModule module = new PuntLogicModule(client);
		InteractionResult result = module.playerPeek(player);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	@Test
	void fieldPeekDelegatesOnPuntMove() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.PUNT_MOVE);
		PuntLogicModule module = new PuntLogicModule(client);
		InteractionResult result = module.fieldPeek(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.DELEGATE, result.getKind());
		assertEquals(ClientStateId.MOVE, result.getDelegate());
	}

	@Test
	void fieldPeekIgnoresWithoutMoveSquare() {
		PuntLogicModule module = new PuntLogicModule(client);
		InteractionResult result = module.fieldPeek(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}
}
