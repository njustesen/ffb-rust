package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.model.AnimationType;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/throw_a_rock_handler.rs
 * tests (portable subset). The Java mixed class only defines animationType(); the Rust mixed
 * module's no-op init/remove helper tests were pruned as Rust-structural plumbing.
 */
public class ThrowARockHandlerTest {

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_THROW_A_ROCK,
			new com.fumbbl.ffb.server.inducements.bb2020.prayers.ThrowARockHandler().animationType());
	}
}
