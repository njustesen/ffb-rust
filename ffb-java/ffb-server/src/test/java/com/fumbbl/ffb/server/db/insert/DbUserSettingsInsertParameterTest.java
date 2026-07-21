package com.fumbbl.ffb.server.db.insert;

import com.fumbbl.ffb.CommonProperty;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust {@code db_user_settings_insert_parameter} test. The Java
 * constructor takes a {@link CommonProperty} for the setting name.
 */
class DbUserSettingsInsertParameterTest {

	@Test
	void getUpdatedRowsInitial() {
		DbUserSettingsInsertParameter p = new DbUserSettingsInsertParameter("coach1",
			CommonProperty.SETTING_SOUND_VOLUME, "value_b");
		assertEquals(0, p.getUpdatedRows());
	}
}
