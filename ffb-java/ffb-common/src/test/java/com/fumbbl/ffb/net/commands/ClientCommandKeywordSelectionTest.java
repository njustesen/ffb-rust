package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.model.Keyword;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_keyword_selection.rs tests.
 */
public class ClientCommandKeywordSelectionTest {

	@Test
	public void defaultHasNoPlayerId() {
		ClientCommandKeywordSelection cmd = new ClientCommandKeywordSelection();
		assertNull(cmd.getPlayerId());
		assertTrue(cmd.getKeywords().isEmpty());
	}

	@Test
	public void addKeywords() {
		ClientCommandKeywordSelection cmd = new ClientCommandKeywordSelection("player_1", Collections.singletonList(Keyword.GOBLIN));
		assertEquals("player_1", cmd.getPlayerId());
		assertEquals(1, cmd.getKeywords().size());
	}

	@Test
	public void keywordsEmptyByDefault() {
		ClientCommandKeywordSelection cmd = new ClientCommandKeywordSelection();
		assertTrue(cmd.getKeywords().isEmpty());
	}

	@Test
	public void getIdIsClientKeywordSelection() {
		assertEquals(NetCommandId.CLIENT_KEYWORD_SELECTION, new ClientCommandKeywordSelection().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndKeywords() {
		ClientCommandKeywordSelection cmd = new ClientCommandKeywordSelection("player_1", Collections.singletonList(Keyword.GOBLIN));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientKeywordSelection", json.get("netCommandId").asString());
		assertEquals(1, json.get("keywords").asArray().size());
		assertEquals("Goblin", json.get("keywords").asArray().get(0).asString());
	}

	@Test
	public void roundTripWithKeywordsAndEntropy() {
		ClientCommandKeywordSelection cmd = new ClientCommandKeywordSelection("player_1", Arrays.asList(Keyword.GOBLIN, Keyword.BIG_GUY));
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandKeywordSelection restored = new ClientCommandKeywordSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals("player_1", restored.getPlayerId());
		assertEquals(Arrays.asList(Keyword.GOBLIN, Keyword.BIG_GUY), restored.getKeywords());
	}

	@Test
	public void roundTripWithEmptyKeywords() {
		ClientCommandKeywordSelection cmd = new ClientCommandKeywordSelection();
		JsonObject json = cmd.toJsonValue();
		ClientCommandKeywordSelection restored = new ClientCommandKeywordSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.getKeywords().isEmpty());
		assertNull(restored.getPlayerId());
	}
}
