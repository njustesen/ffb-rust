package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.ReRolledActionFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/re_rolled_actions.rs for {@link ReRolledActions}.
 */
public class ReRolledActionsTest {

	@Test
	public void forNameCaseInsensitive() {
		ReRolledActionFactory r = new ReRolledActionFactory();
		assertNotNull(r.forName("Go For It"));
		assertNotNull(r.forName("go for it"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(new ReRolledActionFactory().forName("NOT_AN_ACTION"));
	}

	@Test
	public void forNameReturnsActionWithOriginalCasing() {
		ReRolledActionFactory r = new ReRolledActionFactory();
		ReRolledAction action = r.forName("DODGE");
		assertNotNull(action, "should find Dodge case-insensitively");
		assertEquals("Dodge", action.getName());
	}
}
