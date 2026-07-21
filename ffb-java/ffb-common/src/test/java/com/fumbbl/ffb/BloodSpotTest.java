package com.fumbbl.ffb;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/model/blood_spot.rs for {@link BloodSpot}.
 */
public class BloodSpotTest {

	@Test
	public void serdeRoundTripDefault() {
		BloodSpot b = new BloodSpot();
		JsonValue json = b.toJsonValue();
		BloodSpot back = new BloodSpot().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertNull(back.getInjury());
		assertNull(back.getCoordinate());
	}
}
