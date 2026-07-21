package com.fumbbl.ffb.model;

import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/types/block_types.rs for {@link BlockTarget}.
 */
public class BlockTargetTest {

	@Test
	public void blockTargetSerde() {
		BlockTarget bt = new BlockTarget("p2", BlockKind.BLOCK, new PlayerState(0x00001));
		JsonValue json = bt.toJsonValue();
		BlockTarget back = new BlockTarget().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(bt.getPlayerId(), back.getPlayerId());
		assertEquals(bt.getKind(), back.getKind());
		assertEquals(bt.getOriginalPlayerState(), back.getOriginalPlayerState());
	}

	@Test
	public void toJsonValueHasBlockKindAndPlayerId() {
		BlockTarget t = new BlockTarget("p1", BlockKind.STAB, null);
		JsonObject json = t.toJsonValue();
		assertEquals("STAB", json.get("blockKind").asString());
		assertEquals("p1", json.get("playerId").asString());
	}

	@Test
	public void jsonRoundTripWithAllFields() {
		BlockTarget t = new BlockTarget("p2", BlockKind.CHAINSAW, new PlayerState(4));
		JsonValue json = t.toJsonValue();
		BlockTarget restored = new BlockTarget().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals("p2", restored.getPlayerId());
		assertEquals(BlockKind.CHAINSAW, restored.getKind());
		assertEquals(new PlayerState(4), restored.getOriginalPlayerState());
	}
}
