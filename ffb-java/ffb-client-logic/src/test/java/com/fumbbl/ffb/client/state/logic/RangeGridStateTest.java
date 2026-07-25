package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.CommonProperty;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.IClientPropertyValue;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class RangeGridStateTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private Game game;

	@Mock
	private ActingPlayer actingPlayer;

	@Mock
	private FieldModel fieldModel;

	@Mock
	private Player<?> player;

	@BeforeEach
	public void setUp() {
		given(client.getGame()).willReturn(game);
		given(game.getActingPlayer()).willReturn(actingPlayer);
		given(game.getFieldModel()).willReturn(fieldModel);
		given(actingPlayer.getPlayer()).willReturn((com.fumbbl.ffb.model.Player) player);
	}

	@Test
	public void testNewDefaultsToHidden() {
		RangeGridState state = new RangeGridState(client, false);
		assertFalse(state.isShowRangeGrid());
	}

	@Test
	public void testSetShowRangeGridTogglesFlag() {
		RangeGridState state = new RangeGridState(client, false);
		state.setShowRangeGrid(true);
		assertTrue(state.isShowRangeGrid());
	}

	@Test
	public void testRefreshRangeGridResetsWhenHidden() {
		RangeGridState state = new RangeGridState(client, false);
		assertEquals(InteractionResult.Kind.RESET, state.refreshRangeGrid().getKind());
	}

	@Test
	public void testRefreshRangeGridPerformsWithoutPlayerCoordinate() {
		RangeGridState state = new RangeGridState(client, false);
		state.setShowRangeGrid(true);
		// getPlayerCoordinate(player) is left unstubbed -> null. Java still returns PERFORM here,
		// attaching the null coordinate; the Kind is not gated on coordinate presence.

		InteractionResult result = state.refreshRangeGrid();
		assertEquals(InteractionResult.Kind.PERFORM, result.getKind());
		assertEquals(null, result.getCoordinate());
	}

	@Test
	public void testRefreshRangeGridPerformsWhenShownAndNotGatedByTtm() {
		RangeGridState state = new RangeGridState(client, false);
		state.setShowRangeGrid(true);
		FieldCoordinate coordinate = new FieldCoordinate(2, 3);
		given(fieldModel.getPlayerCoordinate(player)).willReturn(coordinate);

		InteractionResult result = state.refreshRangeGrid();
		assertEquals(InteractionResult.Kind.PERFORM, result.getKind());
		assertEquals(coordinate, result.getCoordinate());
	}

	@Test
	public void testRefreshRangeGridGatedByTtmRequiresMatchingAction() {
		RangeGridState state = new RangeGridState(client, true);
		state.setShowRangeGrid(true);
		FieldCoordinate coordinate = new FieldCoordinate(2, 3);
		given(fieldModel.getPlayerCoordinate(player)).willReturn(coordinate);

		given(actingPlayer.getPlayerAction()).willReturn(PlayerAction.MOVE);
		assertEquals(InteractionResult.Kind.RESET, state.refreshRangeGrid().getKind());

		given(actingPlayer.getPlayerAction()).willReturn(PlayerAction.THROW_TEAM_MATE);
		InteractionResult result = state.refreshRangeGrid();
		assertEquals(InteractionResult.Kind.PERFORM, result.getKind());
		assertEquals(coordinate, result.getCoordinate());
	}

	@Test
	public void testRefreshSettingsIgnoresWhenPropertyNotAlwaysOn() {
		RangeGridState state = new RangeGridState(client, false);
		given(client.getProperty(CommonProperty.SETTING_RANGEGRID)).willReturn(IClientPropertyValue.SETTING_RANGEGRID_TOGGLE);

		InteractionResult result = state.refreshSettings();
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
		assertFalse(state.isShowRangeGrid());
	}

	@Test
	public void testRefreshSettingsTurnsOnAndRefreshesWhenAlwaysOnAndHidden() {
		RangeGridState state = new RangeGridState(client, false);
		given(client.getProperty(CommonProperty.SETTING_RANGEGRID)).willReturn(IClientPropertyValue.SETTING_RANGEGRID_ALWAYS_ON);
		FieldCoordinate coordinate = new FieldCoordinate(4, 4);
		given(fieldModel.getPlayerCoordinate(player)).willReturn(coordinate);

		InteractionResult result = state.refreshSettings();
		assertTrue(state.isShowRangeGrid());
		assertEquals(InteractionResult.Kind.PERFORM, result.getKind());
		assertEquals(coordinate, result.getCoordinate());
	}

	@Test
	public void testRefreshSettingsIgnoresWhenAlreadyShown() {
		RangeGridState state = new RangeGridState(client, false);
		state.setShowRangeGrid(true);
		given(client.getProperty(CommonProperty.SETTING_RANGEGRID)).willReturn(IClientPropertyValue.SETTING_RANGEGRID_ALWAYS_ON);

		InteractionResult result = state.refreshSettings();
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}
}
