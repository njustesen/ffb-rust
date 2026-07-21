package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.MoveSquare;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.factory.INamedObjectFactory;
import com.fumbbl.ffb.mechanics.JumpMechanic;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/move_logic_module.rs}'s
 * {@code #[cfg(test)] mod tests}, against the real {@link MoveLogicModule}.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class MoveLogicModuleTest {

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
	private INamedObjectFactory<Object> mechanicFactory;

	@Mock
	private JumpMechanic jumpMechanic;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private MoveLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(game.<INamedObjectFactory<Object>>getFactory(FactoryType.Factory.MECHANIC)).thenReturn(mechanicFactory);
		when(mechanicFactory.forName(Mechanic.Type.JUMP.name())).thenReturn(jumpMechanic);

		module = new MoveLogicModule(client);
	}

	// Rust: available_actions_contains_expected_variants
	@Test
	void availableActionsContainsExpectedVariants() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertTrue(actions.contains(ClientAction.JUMP));
		assertTrue(actions.contains(ClientAction.AUTO_GAZE_ZOAT));
		assertEquals(20, actions.size());
	}

	// Rust: action_context_empty_without_any_special_availability
	// SKIPPED: MoveLogicModule#actionContext(ActingPlayer) exercises a long chain of
	// LogicModule availability helpers (isPassAnySquareAvailable, isSpecialAbilityAvailable,
	// isEndPlayerActionAvailable, ...) plus plugin.actionContext(...), each walking a real
	// Game/Team/FieldModel graph. Faithfully driving all of them to an "empty" result would
	// require a populated Game rather than plain mocks, per the task's skip list.

	// Rust: kind_returns_move_for_plain_square
	@Test
	void kindReturnsMoveForPlainSquare() {
		when(actingPlayer.isJumping()).thenReturn(false);
		MoveSquare square = new MoveSquare(new FieldCoordinate(3, 3), 0, 0);
		assertEquals(MoveSquare.Kind.MOVE, module.kind(square));
	}

	// Rust: kind_returns_dodge_when_dodging_and_not_jumping
	@Test
	void kindReturnsDodgeWhenDodgingAndNotJumping() {
		when(actingPlayer.isJumping()).thenReturn(false);
		MoveSquare square = new MoveSquare(new FieldCoordinate(3, 3), 3, 0);
		assertEquals(MoveSquare.Kind.DODGE, module.kind(square));
	}

	// Rust: move_square_none_without_game
	// SKIPPED: the Rust `move_square` short-circuits to `None` via `client.game()?` when no
	// game is set. Java's private `moveSquare(FieldCoordinate)` has no such null-safety check
	// (it dereferences `client.getGame()` unconditionally), so a "no game" scenario has no
	// faithful Java counterpart here — it would just NPE. Not a behavioral bug, just a gap
	// between Rust's `Option`-based client and Java's non-null assumption.

	// Rust: automove_path_empty_without_moving_action
	@Test
	void automovePathEmptyWithoutMovingAction() {
		when(actingPlayer.getPlayerAction()).thenReturn(null);
		FieldCoordinate[] path = module.automovePath(new FieldCoordinate(5, 5));
		assertEquals(0, path.length);
	}

	// Rust: find_shortest_path_empty_without_moving_action
	@Test
	void findShortestPathEmptyWithoutMovingAction() {
		when(actingPlayer.getPlayerAction()).thenReturn(null);
		FieldCoordinate[] path = module.findShortestPath(new FieldCoordinate(5, 5));
		assertEquals(0, path.length);
	}

	// Rust: field_peek_resets_when_no_move_square_or_path
	@Test
	void fieldPeekResetsWhenNoMoveSquareOrPath() {
		InteractionResult result = module.fieldPeek(new FieldCoordinate(5, 5));
		assertEquals(InteractionResult.Kind.RESET, result.getKind());
	}

	// Rust: field_interaction_ignores_without_move_square_or_path
	@Test
	void fieldInteractionIgnoresWithoutMoveSquareOrPath() {
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(5, 5));
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// Rust: player_interaction_ignores_without_game
	// ADAPTED: Java's `playerInteraction(Player<?>)` dereferences `client.getGame()`
	// unconditionally (no null-safety), so the Rust "no game" setup has no direct Java
	// equivalent. This instead exercises the equivalent real branch: the passed-in player is
	// not the acting player (the "B&C" / other-player path) and there is no move square for
	// them, so the result is still IGNORE.
	@Test
	void playerInteractionIgnoresForOtherPlayerWithoutMoveSquare() {
		InteractionResult result = module.playerInteraction(player);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// Rust: end_turn_sends_end_turn_command
	@Test
	void endTurnSendsEndTurnCommand() {
		when(actingPlayer.getPlayer()).thenReturn(player);
		when(actingPlayer.hasActed()).thenReturn(false);
		when(game.getTurnMode()).thenReturn(TurnMode.REGULAR);

		module.endTurn();

		verify(client.getCommunication()).sendEndTurn(TurnMode.REGULAR);
	}

	// Rust: perform_available_action_no_op_without_game
	// ADAPTED: Java's `performAvailableAction(Player<?>, ClientAction)` guards its whole body
	// with `if (player != null)` (unlike the Rust port, which instead early-returns when there
	// is no game). This exercises that real Java guard directly: a null player is a no-op.
	@Test
	void performAvailableActionNoOpWhenPlayerNull() {
		assertDoesNotThrow(() -> module.performAvailableAction(null, ClientAction.END_MOVE));
	}
}
