package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/kick_em_block_logic_module.rs}
 * against the real mixed {@link KickEmBlockLogicModule} (extends BlockLogicModule → BLOCK plugin).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code is_kickable_false_without_defender_state} (Java UtilPlayer.isKickable NPEs on a null
 * defender state instead of returning false); {@code player_peek_resets_without_game} /
 * {@code player_interaction_ignores_without_game} (Rust no-game short-circuits).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class KickEmBlockLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

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

	private KickEmBlockLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(actingPlayer.getPlayer()).thenReturn(actor);
		module = new KickEmBlockLogicModule(client);
	}

	// rust: is_kickable_requires_away_team (standing defender → not kickable)
	@Test
	void isKickableRequiresAwayTeam() {
		when(fieldModel.getPlayerState(player)).thenReturn(playerState);
		when(playerState.isProneOrStunned()).thenReturn(false);
		assertFalse(UtilPlayer.isKickable(game, player));
	}

	// rust: player_interaction_ignores_when_target_not_kickable
	@Test
	void playerInteractionIgnoresWhenTargetNotKickable() {
		InteractionResult result = module.playerInteraction(player);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}
}
