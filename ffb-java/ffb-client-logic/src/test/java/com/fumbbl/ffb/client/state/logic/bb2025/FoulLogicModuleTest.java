package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
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
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.model.property.NamedProperties;
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
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/bb2025/foul_logic_module.rs}
 * against the real bb2025 {@link FoulLogicModule} (extends MoveLogicModule, whose constructor
 * eagerly resolves a MoveLogicPlugin, so game/factory are mocked explicitly and wired to a real
 * MoveLogicPlugin mock).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code bloodlust_action_context_empty_when_not_suffering} (private {@code bloodlustActionContext}
 * is only reached when suffering blood lust — the empty branch is unreachable/unobservable), and
 * {@code player_peek_ignores_without_game} / {@code player_interaction_ignores_without_game} /
 * {@code perform_available_action_no_op_without_game} (Rust {@code client.game()?} no-game
 * short-circuits with no Java counterpart). The previously-Java-only trivial
 * {@code getIdReturnsFoul} getter test was also removed (getter tautology, no Rust twin).
 */
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

	@Mock
	ActingPlayer actingPlayer;

	@Mock
	FieldModel fieldModel;

	@Mock
	Team teamAway;

	@SuppressWarnings("rawtypes")
	@Mock
	Player actor;

	@SuppressWarnings("rawtypes")
	@Mock
	Player defender;

	private FoulLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(game.getTeamAway()).thenReturn(teamAway);
		when(actingPlayer.getPlayer()).thenReturn(actor);
		module = new FoulLogicModule(client);
	}

	/** Makes {@link #defender} foulable: prone, on the away team, adjacent to the actor, unprotected. */
	private void makeFoulable() {
		when(fieldModel.getPlayerState(defender)).thenReturn(new PlayerState(PlayerState.PRONE));
		when(fieldModel.getPlayerCoordinate(defender)).thenReturn(new FieldCoordinate(1, 1));
		when(fieldModel.getPlayerCoordinate(actor)).thenReturn(new FieldCoordinate(1, 2));
		when(teamAway.hasPlayer(defender)).thenReturn(true);
		when(defender.hasSkillProperty(NamedProperties.preventBeingFouled)).thenReturn(false);
	}

	// rust: available_actions_contains_foul_and_chainsaw
	@Test
	void availableActionsContainsFoulAndChainsaw() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.FOUL));
		assertTrue(actions.contains(ClientAction.CHAINSAW));
		assertTrue(actions.contains(ClientAction.INCORPOREAL));
	}

	// rust: is_foulable_false_without_player
	@Test
	void isFoulableFalseWithoutPlayer() {
		assertFalse(UtilPlayer.isFoulable(game, null));
	}

	// rust: is_foulable_true_for_prone_adjacent_away_player
	@Test
	void isFoulableTrueForProneAdjacentAwayPlayer() {
		makeFoulable();
		assertTrue(UtilPlayer.isFoulable(game, defender));
	}

	// rust: is_foulable_false_for_standing_player
	@Test
	void isFoulableFalseForStandingPlayer() {
		when(fieldModel.getPlayerState(defender)).thenReturn(new PlayerState(PlayerState.STANDING));
		when(fieldModel.getPlayerCoordinate(defender)).thenReturn(new FieldCoordinate(1, 1));
		when(fieldModel.getPlayerCoordinate(actor)).thenReturn(new FieldCoordinate(1, 2));
		when(teamAway.hasPlayer(defender)).thenReturn(true);
		assertFalse(UtilPlayer.isFoulable(game, defender));
	}

	// rust: bloodlust_action_context_has_move_and_end_move (via playerInteraction on the acting player)
	@Test
	void bloodlustActionContextHasMoveAndEndMove() {
		when(actingPlayer.isSufferingBloodLust()).thenReturn(true);
		InteractionResult result = module.playerInteraction(actor);
		assertEquals(InteractionResult.Kind.SELECT_ACTION, result.getKind());
		assertTrue(result.getActionContext().getActions().contains(ClientAction.MOVE));
		assertTrue(result.getActionContext().getActions().contains(ClientAction.END_MOVE));
	}
}
