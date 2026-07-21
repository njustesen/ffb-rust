package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.BlockResult;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBlockChoiceTest {

	private ReportBlockChoice make() {
		return new ReportBlockChoice(2, new int[]{3, 5}, 1, BlockResult.PUSHBACK, "def1", false, true, 42);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBlockChoice original = make();
		JsonObject json = original.toJsonValue();
		ReportBlockChoice restored = new ReportBlockChoice().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getNrOfDice(), restored.getNrOfDice());
		assertArrayEquals(original.getBlockRoll(), restored.getBlockRoll());
		assertEquals(original.getDiceIndex(), restored.getDiceIndex());
		assertEquals(original.getBlockRollId(), restored.getBlockRollId());
		assertEquals(original.getBlockResult(), restored.getBlockResult());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
		assertEquals(original.isSuppressExtraEffectHandling(), restored.isSuppressExtraEffectHandling());
		assertEquals(original.isShowNameInReport(), restored.isShowNameInReport());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("blockChoice", json.get("reportId").asString());
	}
}
