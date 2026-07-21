package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
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

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/gaze_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// GazeLogicModule extends MoveLogicModule, whose constructor eagerly resolves a MoveLogicPlugin
// via client.getGame().getFactory(FactoryType.Factory.LOGIC_PLUGIN).forType(LogicPlugin.Type.MOVE)
// and casts the result to MoveLogicPlugin; game/factory are mocked explicitly (not deep-stub
// cascaded) and wired to a real MoveLogicPlugin mock so construction and availableActions() (which
// merges in plugin.availableActions()) both work correctly.
//
// SKIPPED (with reasons):
// - can_be_gazed_false_without_victim / can_be_gazed_false_without_adjacency /
//   can_be_gazed_false_for_own_team_member: Java's `canBeGazed(Player)` needs a live Game (teams,
//   field model with player coordinates/states) via client.getGame() — out of scope.
// - player_interaction_ignores_without_game / player_peek_ignores_without_game: these call
//   client.getGame() unconditionally in Java and require a live Game/ActingPlayer graph.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class GazeLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	MoveLogicPlugin moveLogicPlugin;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(moveLogicPlugin.availableActions()).thenReturn(Collections.emptySet());
	}

	@Test
	void getIdReturnsGaze() {
		GazeLogicModule module = new GazeLogicModule(client);
		assertEquals(ClientStateId.GAZE, module.getId());
	}

	@Test
	void availableActionsDelegatesToMoveLogicAndContainsGaze() {
		GazeLogicModule module = new GazeLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.GAZE));
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.END_MOVE));
	}
}
