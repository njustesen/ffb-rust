package com.fumbbl.ffb.client.state.logic.bb2020;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.ActionContext;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2020/trickster_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// This module's logic never fans out into the live-Game/Player-graph-walking isXAvailable()
// helpers used elsewhere in this batch (actionContext ignores its ActingPlayer argument
// entirely), so every Rust test here has a safe, direct Java equivalent.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class TricksterLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void getIdReturnsTrickster() {
		TricksterLogicModule module = new TricksterLogicModule(client);
		assertEquals(ClientStateId.TRICKSTER, module.getId());
	}

	@Test
	void availableActionsIsEndMoveOnly() {
		Set<ClientAction> actions = new TricksterLogicModule(client).availableActions();
		assertEquals(1, actions.size());
		assertTrue(actions.contains(ClientAction.END_MOVE));
	}

	@Test
	void actionContextAlwaysHasEndMove() {
		TricksterLogicModule module = new TricksterLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);

		ActionContext ctx = module.actionContext(actingPlayer);

		assertEquals(1, ctx.getActions().size());
		assertTrue(ctx.getActions().contains(ClientAction.END_MOVE));
	}

	@Test
	void fieldInteractionIgnoresWithoutMoveSquare() {
		when(client.getGame().getFieldModel().getMoveSquare(org.mockito.ArgumentMatchers.any())).thenReturn(null);
		TricksterLogicModule module = new TricksterLogicModule(client);

		InteractionResult result = module.fieldInteraction(new FieldCoordinate(1, 1));

		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	@Test
	void fieldPeekInvalidWithoutMoveSquare() {
		when(client.getGame().getFieldModel().getMoveSquare(org.mockito.ArgumentMatchers.any())).thenReturn(null);
		TricksterLogicModule module = new TricksterLogicModule(client);

		InteractionResult result = module.fieldPeek(new FieldCoordinate(1, 1));

		assertEquals(InteractionResult.Kind.INVALID, result.getKind());
	}

	@Test
	void playerInteractionSelectsActionForDefender() {
		Player<?> defender = mock(Player.class);
		when(client.getGame().getDefender()).thenReturn((com.fumbbl.ffb.model.Player) defender);
		TricksterLogicModule module = new TricksterLogicModule(client);

		InteractionResult result = module.playerInteraction(defender);

		assertEquals(InteractionResult.Kind.SELECT_ACTION, result.getKind());
	}

	@Test
	void playerInteractionIgnoresNonDefender() {
		Player<?> defender = mock(Player.class);
		Player<?> other = mock(Player.class);
		when(client.getGame().getDefender()).thenReturn((com.fumbbl.ffb.model.Player) defender);
		TricksterLogicModule module = new TricksterLogicModule(client);

		InteractionResult result = module.playerInteraction(other);

		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	@Test
	void performAvailableActionSendsEndTurnForEndMove() {
		TricksterLogicModule module = new TricksterLogicModule(client);
		Player<?> player = mock(Player.class);

		module.performAvailableAction(player, ClientAction.END_MOVE);

		verify(client.getCommunication()).sendEndTurn(TurnMode.TRICKSTER);
	}

	@Test
	void performAvailableActionNoOpForOtherActions() {
		TricksterLogicModule module = new TricksterLogicModule(client);
		Player<?> player = mock(Player.class);

		module.performAvailableAction(player, ClientAction.MOVE);

		verify(client.getCommunication(), never()).sendEndTurn(org.mockito.ArgumentMatchers.any());
	}
}
