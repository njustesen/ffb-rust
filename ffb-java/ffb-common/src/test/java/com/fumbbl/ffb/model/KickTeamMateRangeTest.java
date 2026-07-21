package com.fumbbl.ffb.model;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/kick_team_mate_range.rs for {@link KickTeamMateRange}.
 */
public class KickTeamMateRangeTest {

	@Test
	public void serdeRoundTrip() {
		for (KickTeamMateRange v : new KickTeamMateRange[]{KickTeamMateRange.LONG, KickTeamMateRange.MEDIUM, KickTeamMateRange.SHORT}) {
			assertEquals(v, KickTeamMateRange.valueOf(v.name()));
		}
	}

}
