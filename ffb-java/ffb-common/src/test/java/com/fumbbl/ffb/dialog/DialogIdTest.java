package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.factory.DialogIdFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_id.rs for {@link DialogId}.
 */
public class DialogIdTest {

	private final DialogIdFactory factory = new DialogIdFactory();

	@Test
	public void getNameRoundTrips() {
		DialogId[] ids = {
			DialogId.RE_ROLL,
			DialogId.BLOCK_ROLL,
			DialogId.APOTHECARY_CHOICE,
			DialogId.FOLLOWUP_CHOICE,
			DialogId.TEAM_SETUP,
			DialogId.PUNT_TO_CROWD,
		};
		for (DialogId id : ids) {
			assertEquals(id, factory.forName(id.getName()));
		}
	}

	@Test
	public void unknownNameReturnsNone() {
		assertNull(factory.forName("nonExistent"));
	}

	@Test
	public void serdeRoundTrip() {
		DialogId id = DialogId.RE_ROLL_BLOCK_FOR_TARGETS;
		assertEquals(id, factory.forName(id.getName()));
	}

	@Test
	public void forNameBlockRollRoundTrips() {
		assertEquals(DialogId.BLOCK_ROLL, factory.forName("blockRoll"));
	}

}
