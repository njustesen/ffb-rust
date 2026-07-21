package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.SendToBoxReasonFactory;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/send_to_box_reason.rs for {@link SendToBoxReason}.
 */
public class SendToBoxReasonTest {

	private final SendToBoxReasonFactory factory = new SendToBoxReasonFactory();

	@Test
	public void forNameCaseInsensitive() {
		assertEquals(SendToBoxReason.MNG, factory.forName("mng"));
		assertEquals(SendToBoxReason.MNG, factory.forName("MNG"));
		assertEquals(SendToBoxReason.FOUL_BAN, factory.forName("foulBan"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(factory.forName("NOT_VALID"));
	}

}
