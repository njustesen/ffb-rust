package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.ConcedeGameStatusFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/concede_game_status.rs for {@link ConcedeGameStatus}.
 */
public class ConcedeGameStatusTest {

	private final ConcedeGameStatusFactory factory = new ConcedeGameStatusFactory();

	@Test
	public void fromNameRoundTrips() {
		assertEquals(ConcedeGameStatus.REQUESTED, factory.forName("requested"));
		assertNull(factory.forName("invalid"));
	}

	@Test
	public void allNamesRoundTrip() {
		for (ConcedeGameStatus v : new ConcedeGameStatus[]{ConcedeGameStatus.REQUESTED, ConcedeGameStatus.CONFIRMED, ConcedeGameStatus.DENIED}) {
			assertEquals(v, factory.forName(v.getName()));
		}
	}

	@Test
	public void serdeRoundTrip() {
		assertEquals(ConcedeGameStatus.CONFIRMED, ConcedeGameStatus.valueOf(ConcedeGameStatus.CONFIRMED.name()));
	}

}
