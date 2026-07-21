package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.model.Game;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/select_blitz_target_logic_module.rs
// (Rust: mod tests). SelectBlitzTargetLogicModule extends MoveLogicModule and also builds a
// BlockLogicExtension field; both constructors eagerly resolve a plugin
// (MoveLogicPlugin/BlockLogicExtensionPlugin respectively) via
// client.getGame().getFactory(FactoryType.Factory.LOGIC_PLUGIN).forType(...); game/factory are
// mocked explicitly (not deep-stub cascaded) and wired to real plugin mocks so construction
// succeeds.
//
// SKIPPED (with reasons):
// - action_context_always_contains_end_move: `actionContext(ActingPlayer)` fans out into many
//   isXAvailable(actingPlayer) helpers on LogicModule which read `client.getGame()` and, for a
//   default/null-player ActingPlayer, would dereference a null Player — risks NPEs, out of scope.
// - can_be_blitzed_false_when_has_blocked: `canBeBlitzed(Player, ActingPlayer, Game)` is `private`
//   in Java, not callable from a test even in the same package.
// - player_peek_invalid_without_game / player_peek_invalid_when_not_blitzable: Java's playerPeek
//   calls `client.getUserInterface().getFieldComponent()...clearMovePath()` unconditionally before
//   any game-state branching, and canBeBlitzed needs a live Game — out of scope.
// - player_interaction_ignores_without_game / perform_available_action_end_move_sends_target_selected:
//   Java calls `client.getGame()` unconditionally and requires a live Player/ActingPlayer graph.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SelectBlitzTargetLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	MoveLogicPlugin moveLogicPlugin;

	@Mock
	BlockLogicExtensionPlugin blockLogicExtensionPlugin;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
	}

	@Test
	void getIdReturnsSelectBlitzTarget() {
		SelectBlitzTargetLogicModule module = new SelectBlitzTargetLogicModule(client);
		assertEquals(ClientStateId.SELECT_BLITZ_TARGET, module.getId());
	}

	@Test
	void availableActionsContainsEndMoveAndIncorporeal() {
		SelectBlitzTargetLogicModule module = new SelectBlitzTargetLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertTrue(actions.contains(ClientAction.INCORPOREAL));
		assertEquals(12, actions.size());
	}
}
