package com.fumbbl.ffb.report.bb2020;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_skill_use_other_player.rs tests.
 * Skill is factory-backed (left null, option null-safe). Rust stores the skill-use as a free string
 * ("USE"); Java uses the SkillUse enum, so a real enum value is used for the round-trip (documented
 * Rust free-string vs Java-enum divergence).
 */
public class ReportSkillUseOtherPlayerTest {

	private ReportSkillUseOtherPlayer make() {
		return new ReportSkillUseOtherPlayer("p1", null, SkillUse.WOULD_NOT_HELP, "p2");
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportSkillUseOtherPlayer original = make();
		JsonObject json = original.toJsonValue();
		ReportSkillUseOtherPlayer restored = new ReportSkillUseOtherPlayer().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getOtherPlayerId(), restored.getOtherPlayerId());
		assertEquals(original.getSkillUse(), restored.getSkillUse());
		assertNull(restored.getSkill());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("skillUseOtherPlayer", json.get("reportId").asString());
	}
}
