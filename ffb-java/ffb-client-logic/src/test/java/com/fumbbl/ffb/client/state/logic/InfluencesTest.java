package com.fumbbl.ffb.client.state.logic;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

class InfluencesTest {

	@Test
	public void testHasActedInfluencesEndMove() {
		assertEquals(Arrays.asList(ClientAction.END_MOVE), Influences.HAS_ACTED.getInfluencedActions());
	}

	@Test
	public void testBallActionsDueToTreacherousHasThreeActions() {
		assertEquals(
			Arrays.asList(ClientAction.PASS, ClientAction.HAND_OVER, ClientAction.SHOT_TO_NOTHING),
			Influences.BALL_ACTIONS_DUE_TO_TREACHEROUS.getInfluencedActions());
	}

	@Test
	public void testIncorporealActiveInfluencesTwoActions() {
		assertEquals(
			Arrays.asList(ClientAction.INCORPOREAL, ClientAction.END_MOVE),
			Influences.INCORPOREAL_ACTIVE.getInfluencedActions());
	}

	@Test
	public void testIsThrowingHailMaryInfluencesBothBombAndPass() {
		assertEquals(
			Arrays.asList(ClientAction.HAIL_MARY_BOMB, ClientAction.HAIL_MARY_PASS),
			Influences.IS_THROWING_HAIL_MARY.getInfluencedActions());
	}
}
