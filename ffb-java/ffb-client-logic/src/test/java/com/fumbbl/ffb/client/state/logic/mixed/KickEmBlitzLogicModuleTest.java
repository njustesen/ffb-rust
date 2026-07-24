package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.util.UtilPlayer;
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
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/kick_em_blitz_logic_module.rs}
 * against the real mixed {@link KickEmBlitzLogicModule} (extends BlitzLogicModule → MOVE + BLOCK
 * plugins). The Rust {@code is_kickable} free fn mirrors {@code UtilPlayer.isKickable}.
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code is_kickable_false_without_defender_state} (Java isKickable calls
 * {@code defenderState.isProneOrStunned()} first — a null state NPEs rather than returning false,
 * a Rust-vs-Java representation difference); and {@code player_peek_resets_without_game} /
 * {@code player_interaction_ignores_without_game} / {@code perform_available_action_no_op_without_game}
 * (Rust no-game short-circuits).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class KickEmBlitzLogicModuleTest {

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

	@Mock
	ActingPlayer actingPlayer;

	@Mock
	FieldModel fieldModel;

	@Mock
	PlayerState playerState;

	@SuppressWarnings("rawtypes")
	@Mock
	Player actor;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private KickEmBlitzLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(actingPlayer.getPlayer()).thenReturn(actor);
		module = new KickEmBlitzLogicModule(client);
	}

	// rust: move_action_is_kick_em_blitz
	@Test
	void moveActionIsKickEmBlitz() {
		assertEquals(PlayerAction.KICK_EM_BLITZ, module.moveAction());
	}

	// rust: is_kickable_requires_away_team_and_adjacency (standing defender → not kickable)
	@Test
	void isKickableRequiresAwayTeamAndAdjacency() {
		when(fieldModel.getPlayerState(player)).thenReturn(playerState);
		when(playerState.isProneOrStunned()).thenReturn(false);
		assertFalse(UtilPlayer.isKickable(game, player));
	}

	// rust: available_actions_delegates_to_blitz
	@Test
	void availableActionsDelegatesToBlitz() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
	}
}
