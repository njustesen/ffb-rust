package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
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

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/pass_logic_module.rs}
 * against the real mixed {@link PassLogicModule}.
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code action_context_adds_end_move_by_default} (Java {@code actionContext} fans out into ~12
 * availability helpers — fixture-inexpressible), and {@code player_interaction_ignores_without_game}
 * / {@code field_interaction_ignores_without_game} / {@code perform_available_action_no_op_without_game}
 * (Rust {@code client.game()?} no-game short-circuits with no Java counterpart).
 */
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

	private PassLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		module = new PassLogicModule(client);
	}

	// rust: available_actions_contains_hail_mary_pass
	@Test
	void availableActionsContainsHailMaryPass() {
		when(moveLogicPlugin.availableActions()).thenReturn(Collections.emptySet());
		assertTrue(module.availableActions().contains(ClientAction.HAIL_MARY_PASS));
		assertTrue(module.availableActions().contains(ClientAction.MOVE));
	}

	// rust: performs_range_grid_action_true_without_pass
	@Test
	void performsRangeGridActionTrueWithoutPass() {
		ActingPlayer ap = new ActingPlayer(null);
		assertTrue(module.performsRangeGridAction(ap, game));
	}

	// rust: performs_range_grid_action_false_after_pass
	@Test
	void performsRangeGridActionFalseAfterPass() {
		ActingPlayer ap = new ActingPlayer(null);
		ap.setHasPassed(true);
		assertFalse(module.performsRangeGridAction(ap, game));
	}

	// rust: action_is_hmp_false_without_game (adapted: no HAIL_MARY_PASS action set → false)
	@Test
	void actionIsHmpFalseByDefault() {
		assertFalse(module.actionIsHmp());
	}

	// rust: can_player_get_pass_false_without_catcher_state
	@Test
	void canPlayerGetPassFalseWithoutCatcherState() {
		when(actingPlayer.getPlayer()).thenReturn(actor);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(fieldModel.getPlayerState(catcher)).thenReturn(null);
		assertFalse(module.canPlayerGetPass(catcher));
	}

	// rust: can_player_get_pass_true_for_home_team_with_tacklezones
	@Test
	void canPlayerGetPassTrueForHomeTeamWithTacklezones() {
		when(actingPlayer.getPlayer()).thenReturn(actor);
		when(actingPlayer.isSufferingAnimosity()).thenReturn(false);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(fieldModel.getPlayerState(catcher)).thenReturn(catcherState);
		when(catcherState.hasTacklezones()).thenReturn(true);
		when(game.getTeamHome()).thenReturn(teamHome);
		when(catcher.getTeam()).thenReturn(teamHome);
		assertTrue(module.canPlayerGetPass(catcher));
	}
}
