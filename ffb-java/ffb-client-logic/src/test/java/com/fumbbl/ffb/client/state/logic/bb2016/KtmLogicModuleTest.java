package com.fumbbl.ffb.client.state.logic.bb2016;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.ClientData;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/bb2016/ktm_logic_module.rs}
 * against the real bb2016 {@link KtmLogicModule} (extends MoveLogicModule).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code can_be_kicked_false_without_game} / {@code player_interaction_ignores_without_game} /
 * {@code perform_available_action_no_op_without_game} (Rust no-game short-circuits).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class KtmLogicModuleTest {

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
	ClientData clientData;

	@SuppressWarnings("rawtypes")
	@Mock
	Player actor;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private KtmLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(actingPlayer.getPlayer()).thenReturn(actor);
		when(client.getClientData()).thenReturn(clientData);
		module = new KtmLogicModule(client);
	}

	// rust: available_actions_matches_java
	@Test
	void availableActionsMatchesJava() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.PASS_SHORT));
		assertTrue(actions.contains(ClientAction.PASS_LONG));
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertEquals(3, actions.size());
	}

	// rust: can_be_kicked_false_without_required_skills
	@Test
	void canBeKickedFalseWithoutRequiredSkills() {
		assertFalse(module.canBeKicked(player));
	}

	// rust: field_interaction_performs_when_ktm_move
	@Test
	void fieldInteractionPerformsWhenKtmMove() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.KICK_TEAM_MATE_MOVE);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.PERFORM, result.getKind());
	}

	// rust: field_interaction_ignores_without_ktm_move
	@Test
	void fieldInteractionIgnoresWithoutKtmMove() {
		when(actingPlayer.getPlayerAction()).thenReturn(null);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: player_peek_sets_selected_player_and_resets_when_not_kickable
	@Test
	void playerPeekSetsSelectedPlayerAndResetsWhenNotKickable() {
		when(game.getDefender()).thenReturn(null);
		when(game.getPassCoordinate()).thenReturn(null);
		InteractionResult result = module.playerPeek(player);
		verify(clientData).setSelectedPlayer(player);
		assertEquals(InteractionResult.Kind.RESET, result.getKind());
	}
}
