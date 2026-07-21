package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.ClientData;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.ClientAction;
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

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/select_logic_module.rs
// (Rust: mod tests). SelectLogicModule extends LogicModule directly (no plugin/mixin fields), so a
// plain explicitly-wired Game/FieldModel mock suffices for construction.
//
// SKIPPED (with reasons):
// - action_context_for_player_adds_move_for_standing_player: Java's `actionContext(Player)` fans
//   out into dozens of isXAvailable(player) helpers (isTreacherousAvailable, isBlockActionAvailable,
//   etc.), each reading a live Game/Team/FieldModel graph — out of scope.
// - can_declare_skill_action_false_without_skill: mirrors `Player.canDeclareSkillAction(property,
//   PlayerState)`, a method on the abstract `Player<T extends Position>` class that walks real
//   `Skill` objects; `Player` cannot be trivially instantiated (abstract, needs a concrete roster
//   position subclass) without building more live model state than is safe here.
// - player_interaction_ignores_without_game / player_interaction_selects_action_for_active_home_player /
//   perform_available_action_move_sends_command / perform_available_action_no_op_without_game:
//   these all require a live Game/Team/Player object graph or route through the skipped
//   actionContext(Player) — out of scope.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SelectLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	FieldModel fieldModel;

	@Mock
	ClientData clientData;

	@Mock
	@SuppressWarnings("rawtypes")
	Player player;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(client.getClientData()).thenReturn(clientData);
	}

	@Test
	void getIdReturnsSelectPlayer() {
		SelectLogicModule module = new SelectLogicModule(client);
		assertEquals(ClientStateId.SELECT_PLAYER, module.getId());
	}

	@Test
	void availableActionsContainsMoveAndPunt() {
		SelectLogicModule module = new SelectLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.PUNT));
		assertEquals(32, actions.size());
	}

	@Test
	void actionContextForActingPlayerThrows() {
		SelectLogicModule module = new SelectLogicModule(client);
		ActingPlayer ap = new ActingPlayer(null);
		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(ap));
	}

	@Test
	@SuppressWarnings("unchecked")
	void isMoveActionAvailableTrueForStandingActivePlayer() {
		PlayerState state = new PlayerState(PlayerState.STANDING).changeActive(true);
		when(fieldModel.getPlayerState(player)).thenReturn(state);
		SelectLogicModule module = new SelectLogicModule(client);
		assertTrue(module.isMoveActionAvailable(player));
	}

	@Test
	@SuppressWarnings("unchecked")
	void isMoveActionAvailableFalseWithoutState() {
		when(fieldModel.getPlayerState(player)).thenReturn(null);
		SelectLogicModule module = new SelectLogicModule(client);
		assertFalse(module.isMoveActionAvailable(player));
	}

	@Test
	void setUpClearsDefenderAndDiceResult() {
		SelectLogicModule module = new SelectLogicModule(client);
		module.setUp();
		verify(game).setDefenderId(null);
		verify(clientData).clearBlockDiceResult();
	}
}
