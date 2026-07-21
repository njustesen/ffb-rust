package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_sketch_command.rs tests.
 * ClientSketchCommand is abstract (extends ClientCommand); a minimal concrete
 * subclass exercises requiresControl()==false, default entropy, and the
 * entropy round-trip (inherited ClientCommand toJsonValue/initFrom).
 */
public class ClientSketchCommandTest {

	private static class TestSketchCommand extends ClientSketchCommand {
		public NetCommandId getId() {
			return NetCommandId.CLIENT_CLEAR_SKETCHES;
		}
	}

	@Test
	public void requiresControlIsFalse() {
		assertFalse(new TestSketchCommand().requiresControl());
	}

	@Test
	public void entropyDefaultsToNone() {
		assertFalse(new TestSketchCommand().hasEntropy());
	}

	@Test
	public void serdeRoundTripWithEntropy() {
		TestSketchCommand cmd = new TestSketchCommand();
		cmd.setEntropy((byte) 99);
		JsonObject json = cmd.toJsonValue();
		TestSketchCommand back = new TestSketchCommand();
		back.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(back.hasEntropy());
		assertEquals((byte) 99, back.getEntropy());
	}
}
