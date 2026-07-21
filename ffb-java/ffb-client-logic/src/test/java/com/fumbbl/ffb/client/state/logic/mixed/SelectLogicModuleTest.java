package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.ClientData;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Game;
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
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/mixed/select_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - action_context_for_player_empty_by_default: Java's actionContext(Player) fetches the Game
//   via client.getGame() internally (unlike Rust's explicit `game: &Game` parameter) and walks
//   many is*Available(player) helpers that themselves read client.getGame() -- a live
//   ActingPlayer/Player/Game object graph, out of scope per task instructions.
// - is_move_action_available_reflects_player_state: the Rust test asserts nothing (`let _ = ...`)
//   and Java's isMoveActionAvailable(Player) reads client.getGame().getFieldModel() internally,
//   so there is no meaningful, safely-mockable behavior to assert here.
// - find_alternate_block_actions_empty_without_skill: Java's findAlternateBlockActions(Player)
//   is a private method, not accessible from a test in the same package.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SelectLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Mock
	Player<?> player;

	@Test
	void getIdIsSelectPlayer() {
		SelectLogicModule module = new SelectLogicModule(client);
		assertEquals(ClientStateId.SELECT_PLAYER, module.getId());
	}

	@Test
	void availableActionsContainsExpectedSet() {
		SelectLogicModule module = new SelectLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.BLOCK));
		assertTrue(actions.contains(ClientAction.VICIOUS_VINES));
		assertEquals(25, actions.size());
	}

	@Test
	void actionContextForActingPlayerPanics() {
		SelectLogicModule module = new SelectLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);
		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}

	@Test
	void playerInteractionIgnoresWithoutGame() {
		SelectLogicModule module = new SelectLogicModule(client);
		InteractionResult result = module.playerInteraction(player);
		assertEquals(InteractionResult.ignore().getKind(), result.getKind());
	}

	@Test
	void setUpClearsDefenderAndBlockDiceResult() {
		SelectLogicModule module = new SelectLogicModule(client);
		Game game = client.getGame();
		ClientData clientData = client.getClientData();

		module.setUp();

		verify(game).setDefenderId(null);
		verify(clientData).clearBlockDiceResult();
	}

	@Test
	void performAvailableActionSendsBlock() {
		SelectLogicModule module = new SelectLogicModule(client);
		ClientCommunication communication = client.getCommunication();

		module.performAvailableAction(player, ClientAction.BLOCK);

		verify(communication).sendActingPlayer(player, PlayerAction.BLOCK, false);
	}
}
