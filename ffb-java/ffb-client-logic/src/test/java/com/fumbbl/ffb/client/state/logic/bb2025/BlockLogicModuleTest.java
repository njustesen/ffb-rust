package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.model.Game;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/block_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// BlockLogicModule extends AbstractBlockLogicModule (extends LogicModule directly, no plugin
// factory lookup of its own), but its constructor also builds a BlockLogicExtension field, whose
// own constructor eagerly resolves a BlockLogicExtensionPlugin via
// client.getGame().getFactory(FactoryType.Factory.LOGIC_PLUGIN).forType(LogicPlugin.Type.BLOCK)
// and casts the result. Game/factory are mocked explicitly (not deep-stub cascaded) and wired to a
// real BlockLogicExtensionPlugin mock so construction succeeds.
//
// SKIPPED (with reasons):
// - action_context_empty_without_any_availability / action_context_adds_end_move_when_has_acted:
//   actionContext() delegates to BlockLogicExtension.actionContext(ActingPlayer), which walks a
//   live ActingPlayer/Player/Game object graph (skills, properties) that cannot be safely mocked
//   here without risking NPEs on unverified internals.
// - player_peek_resets_when_no_game / player_interaction_ignores_without_game /
//   perform_available_action_no_op_without_game: these all require a real Game via
//   client.getGame() feeding into extension.isBlockable(...)/actionContext(...), i.e. live game
//   state, which is out of scope per task instructions.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class BlockLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	BlockLogicExtensionPlugin blockLogicExtensionPlugin;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
	}

	@Test
	void availableActionsContainsBlockAndMove() {
		BlockLogicModule module = new BlockLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.BLOCK));
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.AUTO_GAZE_ZOAT));
	}
}
