package com.fumbbl.ffb;

import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/track_number.rs for {@link TrackNumber}.
 */
public class TrackNumberTest {

	@Test
	public void serdeRoundTrip() {
		TrackNumber t = new TrackNumber(new FieldCoordinate(3, 4), 9);
		JsonValue json = t.toJsonValue();
		TrackNumber back = new TrackNumber().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(9, back.getNumber());
		assertEquals(new FieldCoordinate(3, 4), back.getCoordinate());
	}

}
