package com.fumbbl.ffb.model;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/player_status.rs for {@link PlayerStatus}.
 */
public class PlayerStatusTest {

	@Test
	public void forNameActive() {
		assertEquals(PlayerStatus.ACTIVE, PlayerStatus.forName("ACTIVE"));
		assertEquals(PlayerStatus.ACTIVE, PlayerStatus.forName("active"));
	}

	@Test
	public void forNameUnknown() {
		assertNull(PlayerStatus.forName("invalid"));
	}

	@Test
	public void serdeRoundTrip() {
		for (PlayerStatus v : new PlayerStatus[]{PlayerStatus.ACTIVE, PlayerStatus.JOURNEYMAN}) {
			assertEquals(v, PlayerStatus.valueOf(v.name()));
		}
	}

}
