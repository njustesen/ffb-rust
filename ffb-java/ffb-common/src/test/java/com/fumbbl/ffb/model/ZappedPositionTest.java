package com.fumbbl.ffb.model;

import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/zapped_position.rs for {@link ZappedPosition}.
 */
public class ZappedPositionTest {

	private ZappedPosition zapped(String id, String name) {
		RosterPosition position = new RosterPosition(id);
		position.setName(name);
		return new ZappedPosition(position, NetCommandTestUtil.gameSource());
	}

	@Test
	public void delegatesIdToOriginal() {
		ZappedPosition pos = zapped("lineman-id", "Lineman");
		assertEquals("lineman-id", pos.getId());
	}

	@Test
	public void delegatesNameToOriginal() {
		ZappedPosition pos = zapped("p1", "Snotling");
		assertEquals("Snotling", pos.getName());
	}

}
