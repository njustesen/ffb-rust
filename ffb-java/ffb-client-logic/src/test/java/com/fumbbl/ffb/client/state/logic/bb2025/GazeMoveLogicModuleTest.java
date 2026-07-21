package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/gaze_move_logic_module.rs
// (Rust: mod tests). GazeMoveLogicModule extends GazeLogicModule extends MoveLogicModule, whose
// constructor eagerly resolves a MoveLogicPlugin; game/factory are mocked explicitly (not
// deep-stub cascaded) and wired to a real MoveLogicPlugin mock so construction succeeds.
//
// SKIPPED (with reasons):
// - available_actions_delegates_to_gaze_logic: GazeMoveLogicModule does not override
//   availableActions() in Java, so this would only be re-testing MoveLogicModule's own hardcoded
//   set plus a mocked plugin.availableActions() — no unique behavior to verify beyond what
//   GazeLogicModuleTest already covers.
// - player_interaction_ignores_without_game / player_peek_ignores_without_game /
//   perform_available_action_no_op_without_game: not overridden in Java (inherited from
//   GazeLogicModule/MoveLogicModule), and require a live Game/ActingPlayer object graph to
//   exercise meaningfully — out of scope.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class GazeMoveLogicModuleTest {

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
	}

	@Test
	void getIdReturnsGazeMove() {
		GazeMoveLogicModule module = new GazeMoveLogicModule(client);
		assertEquals(ClientStateId.GAZE_MOVE, module.getId());
	}
}
