package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Team;
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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/pass_logic_module.rs
// (Rust: mod tests). PassLogicModule extends MoveLogicModule, whose constructor eagerly resolves a
// MoveLogicPlugin; game/factory are mocked explicitly (not deep-stub cascaded) and wired to a real
// MoveLogicPlugin mock so construction succeeds.
//
// Pruned from Rust rather than ported (kept the suites 1:1):
// - action_context_empty_without_any_special_availability / action_context_always_adds_end_move:
//   `actionContext(ActingPlayer)` fans out into ~12 isXAvailable helpers + jump mechanic + ball
//   -in-hand — fixture-inexpressible with targeted mocks.
// - player_interaction_ignores_without_game / perform_available_action_no_op_without_game:
//   Rust-only `client.game()?` no-game short-circuits with no Java counterpart.
// The previously-Java-only trivial getIdReturnsPass getter test was also removed (no Rust twin).
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class PassLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	MoveLogicPlugin moveLogicPlugin;

	@Mock
	ActingPlayer actingPlayer;

	@Mock
	FieldModel fieldModel;

	@Mock
	Team teamHome;

	@Mock
	PlayerState catcherState;

	@SuppressWarnings("rawtypes")
	@Mock
	Player actor;

	@SuppressWarnings("rawtypes")
	@Mock
	Player catcher;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
	}

	@Test
	void availableActionsIncludesHailMaryPassAndMove() {
		when(moveLogicPlugin.availableActions()).thenReturn(Collections.emptySet());
		PassLogicModule module = new PassLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.HAIL_MARY_PASS));
		assertTrue(actions.contains(ClientAction.MOVE));
	}

	@Test
	void performsRangeGridActionTrueWhenNotPassed() {
		PassLogicModule module = new PassLogicModule(client);
		ActingPlayer ap = new ActingPlayer(null);
		assertTrue(module.performsRangeGridAction(ap, game));
	}

	@Test
	void performsRangeGridActionFalseWhenPassed() {
		PassLogicModule module = new PassLogicModule(client);
		ActingPlayer ap = new ActingPlayer(null);
		ap.setHasPassed(true);
		assertFalse(module.performsRangeGridAction(ap, game));
	}

	@Test
	void actionIsHmpFalseByDefault() {
		PassLogicModule module = new PassLogicModule(client);
		assertFalse(module.actionIsHmp());
	}

	@Test
	void actionIsHmpTrueWhenActionMatches() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.HAIL_MARY_PASS);
		PassLogicModule module = new PassLogicModule(client);
		assertTrue(module.actionIsHmp());
	}

	@Test
	void fieldInteractionDelegatesOnPassMove() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.PASS_MOVE);
		PassLogicModule module = new PassLogicModule(client);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.DELEGATE, result.getKind());
		assertEquals(ClientStateId.MOVE, result.getDelegate());
	}

	@Test
	void fieldPeekPreviewsThrowByDefault() {
		PassLogicModule module = new PassLogicModule(client);
		InteractionResult result = module.fieldPeek(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.PREVIEW_THROW, result.getKind());
	}

	// rust: can_player_get_pass_false_without_acting_player
	@Test
	void canPlayerGetPassFalseWithoutActingPlayer() {
		when(actingPlayer.getPlayer()).thenReturn(null);
		PassLogicModule module = new PassLogicModule(client);
		assertFalse(module.canPlayerGetPass(catcher));
	}

	// rust: can_player_get_pass_requires_home_team_and_tacklezones
	@Test
	void canPlayerGetPassRequiresHomeTeamAndTacklezones() {
		when(actingPlayer.getPlayer()).thenReturn(actor);
		when(actingPlayer.isSufferingAnimosity()).thenReturn(false);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(fieldModel.getPlayerState(catcher)).thenReturn(catcherState);
		when(catcherState.hasTacklezones()).thenReturn(true);
		when(game.getTeamHome()).thenReturn(teamHome);
		when(catcher.getTeam()).thenReturn(teamHome);
		PassLogicModule module = new PassLogicModule(client);
		assertTrue(module.canPlayerGetPass(catcher));
	}
}
