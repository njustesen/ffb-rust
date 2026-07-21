package com.fumbbl.ffb.client.model;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ControlAwareTest {

	private static final class Impl implements ControlAware {
		private boolean control;

		@Override
		public void setControl(boolean control) {
			this.control = control;
		}
	}

	@Test
	void setControlTrue() {
		Impl i = new Impl();
		i.setControl(true);
		assertTrue(i.control);
	}

	@Test
	void setControlFalse() {
		Impl i = new Impl();
		i.control = true;
		i.setControl(false);
		assertFalse(i.control);
	}
}
