package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.ReRollSourceFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/re_roll_sources.rs for {@link ReRollSources}.
 */
public class ReRollSourcesTest {

	@Test
	public void forNameFindsKnownSource() {
		ReRollSourceFactory r = new ReRollSourceFactory();
		assertNotNull(r.forName("Pro"));
		assertEquals("Pro", r.forName("pro").getName());
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(new ReRollSourceFactory().forName("NOT_VALID"));
	}

	@Test
	public void theBallistaIsRegisteredWithPriorityTwo() {
		ReRollSourceFactory r = new ReRollSourceFactory();
		ReRollSource ballista = r.forName("The Ballista");
		assertNotNull(ballista, "The Ballista must be in the registry");
		assertEquals("The Ballista", ballista.getName());
		assertEquals(2, ballista.getPriority());
	}
}
