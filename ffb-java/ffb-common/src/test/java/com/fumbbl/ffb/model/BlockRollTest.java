package com.fumbbl.ffb.model;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.ReRollSources;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/types/block_types.rs for {@link BlockRoll}.
 */
public class BlockRollTest {

	@Test
	public void blockRollNeedsSelectionWhenIndexNegative() {
		BlockRoll br = new BlockRoll("p1", new PlayerState(PlayerState.STANDING), 1);
		assertTrue(br.needsSelection());
	}

	@Test
	public void blockRollDoesNotNeedSelectionWhenIndexZero() {
		BlockRoll br = new BlockRoll("p1", new PlayerState(PlayerState.STANDING), 1);
		br.setSelectedIndex(0);
		assertFalse(br.needsSelection());
	}

	@Test
	public void blockRollRerollTracking() {
		BlockRoll br = new BlockRoll("p1", new PlayerState(1), 1);
		br.setReRollDiceIndexes(new int[] { 0, 2 });
		assertTrue(br.indexWasReRolled(0));
		assertFalse(br.indexWasReRolled(1));
		assertTrue(br.indexWasReRolled(2));
	}

	@Test
	public void blockRollHasRerollsLeftWhenSourceAdded() {
		BlockRoll br = new BlockRoll("p1", new PlayerState(1), 1);
		assertFalse(br.hasReRollsLeft());
		br.add(ReRollSources.TEAM_RE_ROLL);
		assertTrue(br.hasReRollsLeft());
	}

	@Test
	public void blockRollSerdeRoundTrip() {
		BlockRoll br = new BlockRoll("p1", new PlayerState(1), 42);
		JsonValue json = br.toJsonValue();
		BlockRoll back = new BlockRoll().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(br, back);
	}
}
