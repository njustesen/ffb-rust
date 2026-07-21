package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.FieldCoordinate;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_kick_skill_parameter.rs for
 * {@link DialogKickSkillParameter}.
 */
public class DialogKickSkillParameterTest {

	@Test
	public void transformMirrorsBallCoordinatesNotNaiveClone() {
		FieldCoordinate original = new FieldCoordinate(5, 5);
		FieldCoordinate withKick = new FieldCoordinate(10, 8);
		DialogKickSkillParameter p = new DialogKickSkillParameter("kicker", original, withKick);
		DialogKickSkillParameter transformed = (DialogKickSkillParameter) p.transform();

		assertEquals("kicker", transformed.getPlayerId());

		assertEquals(original.transform(), transformed.getBallCoordinate());
		assertNotEquals(original, transformed.getBallCoordinate());
		assertEquals(withKick.transform(), transformed.getBallCoordinateWithKick());
		assertNotEquals(withKick, transformed.getBallCoordinateWithKick());
	}

	@Test
	public void transformNoneCoordinatesStayNone() {
		DialogKickSkillParameter p = new DialogKickSkillParameter("kicker", null, null);
		DialogKickSkillParameter transformed = (DialogKickSkillParameter) p.transform();
		assertNull(transformed.getBallCoordinate());
		assertNull(transformed.getBallCoordinateWithKick());
	}

}
