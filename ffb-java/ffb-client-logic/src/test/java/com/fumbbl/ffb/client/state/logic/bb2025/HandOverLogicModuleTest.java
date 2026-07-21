package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
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

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/hand_over_logic_module.rs
// (Rust: mod tests). HandOverLogicModule extends MoveLogicModule, whose constructor eagerly
// resolves a MoveLogicPlugin; game/factory are mocked explicitly (not deep-stub cascaded) and
// wired to a real MoveLogicPlugin mock so construction succeeds.
//
// SKIPPED (with reasons):
// - ball_in_hand_false_without_acting_player / can_player_get_hand_over_false_without_catcher /
//   can_player_get_hand_over_false_without_adjacency / can_player_get_hand_over_true_when_adjacent_home_team:
//   Java's `ballInHand()`/`canPlayerGetHandOver(Player)` read `client.getGame()` (live Game with
//   teams/field model/player states) — out of scope per task instructions.
// - action_context_empty_without_any_availability: `actionContext(ActingPlayer)` fans out into
//   many isXAvailable helpers plus `client.getGame()` itself (also called directly inside the
//   method) — needs a live Game/ActingPlayer graph, out of scope.
// - player_interaction_ignores_without_game / player_peek_ignores_without_game: Java calls
//   client.getGame() unconditionally (NPEs rather than short-circuiting), needing a live Game.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class HandOverLogicModuleTest {

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
	void getIdReturnsHandOver() {
		HandOverLogicModule module = new HandOverLogicModule(client);
		assertEquals(ClientStateId.HAND_OVER, module.getId());
	}

	@Test
	void fieldPeekDelegatesToMove() {
		HandOverLogicModule module = new HandOverLogicModule(client);
		InteractionResult result = module.fieldPeek(new FieldCoordinate(1, 1));
		assertEquals(InteractionResult.Kind.DELEGATE, result.getKind());
		assertEquals(ClientStateId.MOVE, result.getDelegate());
	}
}
