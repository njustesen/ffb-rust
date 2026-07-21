package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSecretWeaponBanTest {

	private ReportSecretWeaponBan make() {
		ReportSecretWeaponBan report = new ReportSecretWeaponBan();
		report.add("p1", 3, true);
		report.add("p2", 5, false);
		return report;
	}

	@Test
	public void serializationRoundTrip() {
		ReportSecretWeaponBan original = make();
		JsonObject json = original.toJsonValue();
		ReportSecretWeaponBan restored = new ReportSecretWeaponBan().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getPlayerIds(), restored.getPlayerIds());
		assertArrayEquals(original.getRolls(), restored.getRolls());
		assertArrayEquals(original.getBans(), restored.getBans());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("secretWeaponBan", json.get("reportId").asString());
	}
}
