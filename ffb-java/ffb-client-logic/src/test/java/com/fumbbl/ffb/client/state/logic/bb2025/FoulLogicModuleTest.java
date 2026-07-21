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

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/foul_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// FoulLogicModule extends MoveLogicModule, whose constructor eagerly resolves a MoveLogicPlugin
// via client.getGame().getFactory(FactoryType.Factory.LOGIC_PLUGIN).forType(LogicPlugin.Type.MOVE)
// and casts the result to MoveLogicPlugin. A plain RETURNS_DEEP_STUBS client/game mock would
// auto-create a mock of the erased generic factory bound, which fails that cast; game/factory are
// therefore mocked explicitly (not deep-stub-cascaded) and wired to a real MoveLogicPlugin mock so
// construction succeeds.
//
// SKIPPED (with reasons):
// - is_foulable_false_without_player / is_foulable_true_for_prone_adjacent_away_player /
//   is_foulable_false_for_standing_player: the Rust `is_foulable` free function mirrors Java's
//   `com.fumbbl.ffb.util.UtilPlayer.isFoulable(Game, Player)`, which needs a fully-populated
//   `Game` (teams, field model with player coordinates/states) — live game-state construction,
//   out of scope per task instructions.
// - bloodlust_action_context_empty_when_not_suffering / bloodlust_action_context_has_move_and_end_move:
//   Java's `bloodlustActionContext(ActingPlayer)` is a `private` method on `FoulLogicModule`, not
//   callable from a test even in the same package.
// - player_peek_ignores_without_game / player_interaction_ignores_without_game /
//   perform_available_action_no_op_without_game: these call `client.getGame()` unconditionally in
//   Java and otherwise require a live Game/ActingPlayer object graph — out of scope.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class FoulLogicModuleTest {

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
	void getIdReturnsFoul() {
		FoulLogicModule module = new FoulLogicModule(client);
		assertEquals(ClientStateId.FOUL, module.getId());
	}

	@Test
	void availableActionsContainsFoulAndChainsaw() {
		FoulLogicModule module = new FoulLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.FOUL));
		assertTrue(actions.contains(ClientAction.CHAINSAW));
		assertTrue(actions.contains(ClientAction.INCORPOREAL));
	}
}
