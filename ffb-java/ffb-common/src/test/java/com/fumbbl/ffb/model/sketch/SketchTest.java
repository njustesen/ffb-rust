package com.fumbbl.ffb.model.sketch;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/sketch/sketch.rs for {@link Sketch}.
 */
public class SketchTest {

	@Test
	public void addCoordinateDedupsConsecutiveDuplicates() {
		Sketch s = new Sketch(0);
		s.addCoordinate(new FieldCoordinate(1, 1));
		s.addCoordinate(new FieldCoordinate(1, 1));
		assertEquals(1, s.getPath().size());
		s.addCoordinate(new FieldCoordinate(2, 2));
		assertEquals(2, s.getPath().size());
	}

	@Test
	public void roundTripWithIdRgbLabelAndPath() {
		Sketch s = new Sketch(255);
		String originalId = s.getId();
		s.setLabel("hello");
		s.addCoordinate(new FieldCoordinate(3, 4));
		JsonValue json = s.toJsonValue();
		Sketch restored = new Sketch(0).initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(originalId, restored.getId());
		assertEquals(255, restored.getRgb());
		assertEquals("hello", restored.getLabel());
		assertEquals(1, restored.getPath().size());
		assertEquals(new FieldCoordinate(3, 4), restored.getPath().get(0));
	}
}
