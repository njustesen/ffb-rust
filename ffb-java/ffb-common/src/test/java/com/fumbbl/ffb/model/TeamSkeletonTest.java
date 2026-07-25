package com.fumbbl.ffb.model;

import com.fumbbl.ffb.factory.IFactorySource;
import org.junit.jupiter.api.Test;
import org.xml.sax.helpers.AttributesImpl;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-model/src/model/team_skeleton.rs tests.
 *
 * <p>Rust drives XmlHandler::parse over a string; here the SAX IXmlReadable callbacks
 * (startXmlElement/endXmlElement) are invoked directly with the same event sequence, which is the
 * behavior the Rust test exercises. Rust asserts on the private `parsing_player` flag; Java's is
 * private with no getter, so the nested-name test asserts the observable effect (team name not
 * overwritten) instead.
 */
public class TeamSkeletonTest {

	private AttributesImpl idAttr(String id) {
		AttributesImpl a = new AttributesImpl();
		a.addAttribute("", "id", "id", "CDATA", id);
		return a;
	}

	// rust: parses_id_name_coach_from_xml
	@Test
	public void parsesIdNameCoachFromXml() {
		TeamSkeleton t = new TeamSkeleton((IFactorySource) null);
		t.startXmlElement(null, "team", idAttr("42"));
		t.endXmlElement(null, "coach", "Kalimar");
		t.endXmlElement(null, "name", "Chaos");
		assertEquals("42", t.getId());
		assertEquals("Kalimar", t.getCoach());
		assertEquals("Chaos", t.getName());
	}

	// rust: parses_team_value_tag
	@Test
	public void parsesTeamValueTag() {
		TeamSkeleton t = new TeamSkeleton((IFactorySource) null);
		t.startXmlElement(null, "team", idAttr("1"));
		t.endXmlElement(null, "teamValue", "1100000");
		assertEquals(1_100_000, t.getTeamValue());
	}

	// rust: nested_player_name_does_not_overwrite_team_name
	@Test
	public void nestedPlayerNameDoesNotOverwriteTeamName() {
		TeamSkeleton t = new TeamSkeleton((IFactorySource) null);
		t.startXmlElement(null, "team", idAttr("1"));
		t.endXmlElement(null, "name", "Chaos");
		t.startXmlElement(null, "player", new AttributesImpl());
		t.endXmlElement(null, "name", "Bob");
		t.endXmlElement(null, "player", null);
		assertEquals("Chaos", t.getName());
	}
}
