package com.fumbbl.ffb.client;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.SpecialEffect;
import com.fumbbl.ffb.StatusType;
import com.fumbbl.ffb.model.BlockRoll;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith(MockitoExtension.class)
class ClientDataTest {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player selectedPlayer;

	@Test
	void setBlockDiceResultReplacesPrevious() {
		ClientData data = new ClientData();
		data.setBlockDiceResult(java.util.List.of(new BlockRoll()));
		assertEquals(1, data.getBlockRolls().size());
		data.setBlockDiceResult(java.util.List.of(new BlockRoll(), new BlockRoll()));
		assertEquals(2, data.getBlockRolls().size());
	}

	@Test
	void clearBlockDiceResultEmptiesList() {
		ClientData data = new ClientData();
		data.setBlockDiceResult(java.util.List.of(new BlockRoll()));
		data.clearBlockDiceResult();
		assertTrue(data.getBlockRolls().isEmpty());
	}

	@Test
	void setAndClearStatus() {
		ClientData data = new ClientData();
		data.setStatus("t", "m", StatusType.WAITING);
		assertEquals("t", data.getStatusTitle());
		data.clearStatus();
		assertNull(data.getStatusTitle());
		assertNull(data.getStatusMessage());
		assertNull(data.getStatusType());
	}

	@Test
	void clearResetsAllTransientFields() {
		ClientData data = new ClientData();
		data.setSelectedPlayer(selectedPlayer);
		data.setDragStartPosition(new FieldCoordinate(1, 1));
		data.setDragEndPosition(new FieldCoordinate(2, 2));
		data.setBlockDiceResult(java.util.List.of(new BlockRoll()));
		data.setStatus("t", "m", StatusType.WAITING);
		data.setActingPlayerUpdated(true);
		data.setWizardSpell(SpecialEffect.FIREBALL);
		data.setEndTurnButtonHidden(true);

		data.clear();

		assertNull(data.getSelectedPlayer());
		assertNull(data.getDragStartPosition());
		assertNull(data.getDragEndPosition());
		assertTrue(data.getBlockRolls().isEmpty());
		assertNull(data.getStatusTitle());
		assertFalse(data.isActingPlayerUpdated());
		assertNull(data.getWizardSpell());
		assertFalse(data.isEndTurnButtonHidden());
	}

	@Test
	void clearDoesNotResetPersistentFields() {
		// Java's clear() intentionally leaves turnTimerStopped, spectatorCount, spectators,
		// and coachControllingReplay untouched - only re-verify per-selection state resets.
		ClientData data = new ClientData();
		data.setTurnTimerStopped(true);
		data.setSpectatorCount(3);
		data.clear();
		assertTrue(data.isTurnTimerStopped());
		assertEquals(3, data.getSpectatorCount());
	}
}
