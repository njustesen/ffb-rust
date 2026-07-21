package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.PlayerChoiceModeFactory;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/player_choice_mode.rs for {@link PlayerChoiceMode}.
 */
public class PlayerChoiceModeTest {

	private final PlayerChoiceModeFactory factory = new PlayerChoiceModeFactory();

	@Test
	public void forNameRoundTrips() {
		assertEquals(PlayerChoiceMode.TENTACLES, factory.forName("tentacles"));
		assertNull(factory.forName("invalid"));
	}

	@Test
	public void getDialogHeaderMvpInterpolatesPlayerCount() {
		assertEquals("Nominate 3 for the MVP", PlayerChoiceMode.MVP.getDialogHeader(3));
		assertEquals("Select a player to use Tentacles", PlayerChoiceMode.TENTACLES.getDialogHeader(0));
	}

}
