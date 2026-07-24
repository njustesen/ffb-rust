package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.client.ClientData;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.factory.INamedObjectFactory;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.PassMechanic;
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

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyBoolean;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/dump_off_logic_module.rs}
 * against the real {@link DumpOffLogicModule} (extends MoveLogicModule).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code field_interaction_ignores_without_thrower} / {@code field_peek_resets_without_thrower}
 * (both call {@code testCoordinateInRange}, which reads {@code game.getThrower()}'s coordinate and
 * the PASS mechanic's {@code findPassingDistance} over a live Game — the no-thrower path needs that
 * whole graph). The in-range gate is instead exercised via the out-of-range field-interaction test
 * with a stubbed PASS mechanic.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class DumpOffLogicModuleTest {

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

	@Mock
	ClientCommunication communication;

	@Mock
	PassMechanic passMechanic;

	@SuppressWarnings("rawtypes")
	@Mock
	INamedObjectFactory mechanicFactory;

	@SuppressWarnings("rawtypes")
	@Mock
	Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private DumpOffLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(client.getClientData()).thenReturn(clientData);
		when(client.getCommunication()).thenReturn(communication);
		module = new DumpOffLogicModule(client);
	}

	// rust: set_up_clears_pass_coordinate
	@Test
	void setUpClearsPassCoordinate() {
		module.setUp();
		verify(game).setPassCoordinate(null);
	}

	// rust: field_interaction_ignores_out_of_range_coordinate
	@SuppressWarnings({"rawtypes", "unchecked"})
	@Test
	void fieldInteractionIgnoresOutOfRangeCoordinate() {
		when(game.getThrower()).thenReturn(thrower);
		when(game.<INamedObjectFactory>getFactory(FactoryType.Factory.MECHANIC)).thenReturn(mechanicFactory);
		when(mechanicFactory.forName(Mechanic.Type.PASS.name())).thenReturn(passMechanic);
		when(passMechanic.findPassingDistance(any(), any(), any(), anyBoolean())).thenReturn(PassingDistance.LONG_PASS);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(20, 20));
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: player_peek_sets_selected_player_and_ignores
	@Test
	void playerPeekSetsSelectedPlayerAndIgnores() {
		InteractionResult result = module.playerPeek(player);
		verify(clientData).setSelectedPlayer(player);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: perform_available_action_is_no_op
	@Test
	void performAvailableActionIsNoOp() {
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.MOVE));
	}
}
