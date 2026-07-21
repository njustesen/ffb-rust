package com.fumbbl.ffb;

import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/knockout_recovery.rs for {@link KnockoutRecovery}.
 */
public class KnockoutRecoveryTest {

	@Test
	public void serdeRoundTrip() {
		KnockoutRecovery k = new KnockoutRecovery("p1", true, 4, 2, "loner");
		JsonValue json = k.toJsonValue();
		KnockoutRecovery back = new KnockoutRecovery().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals("p1", back.getPlayerId());
		assertEquals("loner", back.getReRollReason());
	}

}
