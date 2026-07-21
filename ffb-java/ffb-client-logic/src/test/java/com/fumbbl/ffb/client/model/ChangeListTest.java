package com.fumbbl.ffb.client.model;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ChangeListTest {

	@Test
	void newHasEightVersions() {
		ChangeList list = new ChangeList();
		assertEquals(8, list.getVersions().size());
	}

	@Test
	void firstVersionIs323() {
		ChangeList list = new ChangeList();
		assertEquals("3.2.3", list.getVersions().get(0).getVersion());
		assertTrue(list.getVersions().get(0).hasBugfixes());
	}

	@Test
	void lastVersionIs300WithDescription() {
		ChangeList list = new ChangeList();
		List<VersionChangeList> versions = list.getVersions();
		VersionChangeList last = versions.get(versions.size() - 1);
		assertEquals("3.0.0", last.getVersion());
		assertTrue(last.hasDescription());
	}

	@Test
	void fingerPrintIsStable() {
		ChangeList a = new ChangeList();
		ChangeList b = new ChangeList();
		assertEquals(a.fingerPrint(), b.fingerPrint());
	}

	@Test
	void instanceIsSingleton() {
		assertEquals(ChangeList.INSTANCE.fingerPrint(), new ChangeList().fingerPrint());
	}
}
