package com.fumbbl.ffb.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class IProgressListenerTest {

	private static final class Recorder implements IProgressListener {
		private int minimum;
		private int maximum;
		private int progress;

		@Override
		public void initProgress(int pMinimum, int pMaximum) {
			minimum = pMinimum;
			maximum = pMaximum;
		}

		@Override
		public void updateProgress(int pProgress) {
			progress = pProgress;
		}
	}

	@Test
	void initProgressSetsBounds() {
		Recorder r = new Recorder();
		r.initProgress(0, 100);
		assertEquals(0, r.minimum);
		assertEquals(100, r.maximum);
	}

	@Test
	void updateProgressSetsProgress() {
		Recorder r = new Recorder();
		r.updateProgress(42);
		assertEquals(42, r.progress);
	}
}
