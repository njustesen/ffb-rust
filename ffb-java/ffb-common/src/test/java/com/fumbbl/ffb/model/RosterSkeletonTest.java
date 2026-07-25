package com.fumbbl.ffb.model;

import org.junit.jupiter.api.Test;
import org.xml.sax.helpers.AttributesImpl;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-model/src/model/roster_skeleton.rs tests.
 * The SAX startXmlElement callback is invoked directly with the root-tag attributes.
 */
public class RosterSkeletonTest {

	// rust: parses_id_and_team_attributes_from_root_tag
	@Test
	public void parsesIdAndTeamAttributesFromRootTag() {
		RosterSkeleton r = new RosterSkeleton();
		AttributesImpl a = new AttributesImpl();
		a.addAttribute("", "id", "id", "CDATA", "undead");
		a.addAttribute("", "team", "team", "CDATA", "42");
		r.startXmlElement(null, "roster", a);
		assertEquals("undead", r.getId());
		assertEquals("42", r.getTeamId());
	}
}
