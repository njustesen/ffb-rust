package com.fumbbl.ffb;

import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/pushback.rs for {@link Pushback}.
 */
public class PushbackTest {

	@Test
	public void transformPreservesPlayerId() {
		Pushback p = new Pushback("p2", new FieldCoordinate(2, 4));
		Pushback t = p.transform();
		assertEquals("p2", t.getPlayerId());
	}

	@Test
	public void serdeRoundTrip() {
		Pushback p = new Pushback("p3", new FieldCoordinate(7, 3));
		JsonValue json = p.toJsonValue();
		Pushback back = new Pushback().initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals("p3", back.getPlayerId());
		assertEquals(new FieldCoordinate(7, 3), back.getCoordinate());
	}

	@Test
	public void transformChangesCoordinate() {
		Pushback p = new Pushback("p1", new FieldCoordinate(5, 5));
		Pushback t = p.transform();
		assertNotNull(t.getCoordinate());
	}

}
