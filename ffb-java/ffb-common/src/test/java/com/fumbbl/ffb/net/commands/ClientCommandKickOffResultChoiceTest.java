package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.factory.KickoffResultFactory;
import com.fumbbl.ffb.kickoff.KickoffResult;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_kick_off_result_choice.rs tests.
 *
 * The default game rules are BB2016; its KickoffResult values include "Blitz" and "Quick Snap"
 * (the Rust enum uses spaceless names "Blitz"/"QuickSnap"). KickoffResult instances are obtained
 * from the game-scoped factory, matching how Java (de)serializes them.
 */
public class ClientCommandKickOffResultChoiceTest {

	private KickoffResult kickoffResult(String name) {
		KickoffResultFactory factory = NetCommandTestUtil.gameSource().getFactory(Factory.KICKOFF_RESULT);
		return factory.forName(name);
	}

	@Test
	public void defaultHasNoKickoffResult() {
		ClientCommandKickOffResultChoice cmd = new ClientCommandKickOffResultChoice();
		assertNull(cmd.getKickoffResult());
	}

	@Test
	public void storesKickoffResult() {
		KickoffResult quickSnap = kickoffResult("Quick Snap");
		ClientCommandKickOffResultChoice cmd = new ClientCommandKickOffResultChoice(quickSnap);
		assertEquals(quickSnap, cmd.getKickoffResult());
	}

	@Test
	public void blitzVariantStored() {
		KickoffResult blitz = kickoffResult("Blitz");
		ClientCommandKickOffResultChoice cmd = new ClientCommandKickOffResultChoice(blitz);
		assertEquals(blitz, cmd.getKickoffResult());
	}

	@Test
	public void getIdIsClientKickOffResultChoice() {
		assertEquals(NetCommandId.CLIENT_KICK_OFF_RESULT_CHOICE, new ClientCommandKickOffResultChoice().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndKickoffResult() {
		ClientCommandKickOffResultChoice cmd = new ClientCommandKickOffResultChoice(kickoffResult("Blitz"));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientKickOffResultChoice", json.get("netCommandId").asString());
		assertEquals("Blitz", json.get("kickoffResult").asString());
	}

	@Test
	public void roundTripWithResultAndEntropy() {
		KickoffResult quickSnap = kickoffResult("Quick Snap");
		ClientCommandKickOffResultChoice cmd = new ClientCommandKickOffResultChoice(quickSnap);
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandKickOffResultChoice restored = (ClientCommandKickOffResultChoice) new ClientCommandKickOffResultChoice()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 3, restored.getEntropy());
		assertEquals(quickSnap, restored.getKickoffResult());
	}

	@Test
	public void roundTripWithNoResult() {
		ClientCommandKickOffResultChoice cmd = new ClientCommandKickOffResultChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandKickOffResultChoice restored = (ClientCommandKickOffResultChoice) new ClientCommandKickOffResultChoice()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getKickoffResult());
	}
}
