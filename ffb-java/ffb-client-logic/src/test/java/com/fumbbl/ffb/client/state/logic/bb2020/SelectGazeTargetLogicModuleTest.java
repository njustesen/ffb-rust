package com.fumbbl.ffb.client.state.logic.bb2020;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
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

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from
 * {@code ffb-rust/crates/ffb-client/src/client/state/logic/bb2020/select_gaze_target_logic_module.rs}
 * against the real bb2020 {@link SelectGazeTargetLogicModule} (extends MoveLogicModule;
 * isValidGazeTarget is private → exercised via playerPeek).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code is_valid_gaze_target_true_for_opponent_with_tacklezones} (redundant with
 * {@code player_peek_performs_for_valid_gaze_target} once isValidGazeTarget is observed through
 * playerPeek — the private method collapses the two into one observable), and
 * {@code player_peek_invalid_without_game} / {@code player_interaction_ignores_without_game}
 * (Rust no-game short-circuits).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SelectGazeTargetLogicModuleTest {

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
	Team actingTeam;

	@Mock
	FieldModel fieldModel;

	@Mock
	PlayerState playerState;

	@Mock
	ClientCommunication communication;

	@SuppressWarnings("rawtypes")
	@Mock
	Player target;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private SelectGazeTargetLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingTeam()).thenReturn(actingTeam);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(client.getCommunication()).thenReturn(communication);
		module = new SelectGazeTargetLogicModule(client);
	}

	// rust: available_actions_matches_java
	@Test
	void availableActionsMatchesJava() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertEquals(9, actions.size());
	}

	// rust: is_valid_gaze_target_false_for_own_team (observed via playerPeek → INVALID)
	@Test
	void isValidGazeTargetFalseForOwnTeam() {
		when(actingTeam.hasPlayer(target)).thenReturn(true);
		InteractionResult result = module.playerPeek(target);
		assertEquals(InteractionResult.Kind.INVALID, result.getKind());
	}

	// rust: player_peek_performs_for_valid_gaze_target
	@Test
	void playerPeekPerformsForValidGazeTarget() {
		when(actingTeam.hasPlayer(target)).thenReturn(false);
		when(fieldModel.getPlayerState(target)).thenReturn(playerState);
		when(playerState.hasTacklezones()).thenReturn(true);
		InteractionResult result = module.playerPeek(target);
		assertEquals(InteractionResult.Kind.PERFORM, result.getKind());
	}

	// rust: perform_available_action_end_move_sends_target_selected
	@Test
	void performAvailableActionEndMoveSendsTargetSelected() {
		when(player.getId()).thenReturn("p1");
		module.performAvailableAction(player, ClientAction.END_MOVE);
		verify(communication).sendTargetSelected("p1");
	}
}
