package com.fumbbl.ffb.client.state.logic.bb2020;

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

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/bb2020/stab_logic_module.rs}
 * against the real bb2020 {@link StabLogicModule} (extends the mixed BlockLogicModule; construction
 * resolves the MOVE + BLOCK plugins).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code get_targets_empty_by_default} (Java {@code targets} is null before setUp, not an empty
 * array — a Rust-vs-Java default-representation detail); {@code set_up_populates_targets_from_adjacent_opponents}
 * / {@code player_peek_performs_for_target} / {@code player_peek_resets_for_non_target} (need a live
 * field graph: setUp's findAdjacentBlockablePlayers, and playerPeek streams the null-until-setUp
 * targets array); {@code action_context_empty_without_special_ability_or_acted} (actionContext
 * fan-out); {@code player_interaction_ignores_without_game} /
 * {@code perform_available_action_no_op_without_game_for_move} (Rust no-game short-circuits).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class StabLogicModuleTest {

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

	private StabLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		module = new StabLogicModule(client);
	}

	// rust: available_actions_includes_extension_and_own_actions
	@Test
	void availableActionsIncludesExtensionAndOwnActions() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertTrue(actions.contains(ClientAction.BLOCK));
		assertTrue(actions.contains(ClientAction.STAB));
	}
}
