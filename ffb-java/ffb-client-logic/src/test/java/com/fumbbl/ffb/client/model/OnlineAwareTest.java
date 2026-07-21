package com.fumbbl.ffb.client.model;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class OnlineAwareTest {

	private static final class Impl implements OnlineAware {
		private boolean online;

		@Override
		public void setOnline(boolean online) {
			this.online = online;
		}
	}

	@Test
	void setOnlineTrue() {
		Impl i = new Impl();
		i.setOnline(true);
		assertTrue(i.online);
	}

	@Test
	void setOnlineFalse() {
		Impl i = new Impl();
		i.online = true;
		i.setOnline(false);
		assertFalse(i.online);
	}
}
