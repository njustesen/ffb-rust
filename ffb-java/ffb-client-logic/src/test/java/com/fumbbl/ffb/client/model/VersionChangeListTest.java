package com.fumbbl.ffb.client.model;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class VersionChangeListTest {

	@Test
	void newHasNoEntries() {
		VersionChangeList v = new VersionChangeList("3.2.3");
		assertEquals("3.2.3", v.getVersion());
		assertFalse(v.hasEntries());
	}

	@Test
	void addBugfixTracksEntry() {
		VersionChangeList v = new VersionChangeList("3.2.3").addBugfix("fixed something");
		assertTrue(v.hasBugfixes());
		assertEquals(List.of("fixed something"), v.getBugfixes());
		assertTrue(v.hasEntries());
	}

	@Test
	void addFeatureAndImprovement() {
		VersionChangeList v = new VersionChangeList("3.2.0")
			.addFeature("new feature")
			.addImprovement("improved thing");
		assertTrue(v.hasFeatures());
		assertTrue(v.hasImprovements());
		assertEquals(List.of("new feature"), v.getFeatures());
		assertEquals(List.of("improved thing"), v.getImprovements());
	}

	@Test
	void addBehaviorChangeRemovalRuleChange() {
		VersionChangeList v = new VersionChangeList("3.2.1")
			.addBehaviorChange("behavior")
			.addRemoval("removal")
			.addRuleChange("rule");
		assertTrue(v.hasBehaviorChanges());
		assertTrue(v.hasRemovals());
		assertTrue(v.hasRuleChanges());
	}

	@Test
	void setDescriptionSetsHasDescription() {
		VersionChangeList v = new VersionChangeList("3.0.0").setDescription("First version");
		assertTrue(v.hasDescription());
		assertEquals("First version", v.getDescription());
		assertTrue(v.hasEntries());
	}

	@Test
	void emptyDescriptionIsNotADescription() {
		VersionChangeList v = new VersionChangeList("3.0.0").setDescription("");
		assertFalse(v.hasDescription());
	}

	@Test
	void equalityComparesAllFields() {
		VersionChangeList a = new VersionChangeList("1.0").addBugfix("x");
		VersionChangeList b = new VersionChangeList("1.0").addBugfix("x");
		VersionChangeList c = new VersionChangeList("1.0").addBugfix("y");
		assertEquals(a, b);
		assertNotEquals(a, c);
	}
}
