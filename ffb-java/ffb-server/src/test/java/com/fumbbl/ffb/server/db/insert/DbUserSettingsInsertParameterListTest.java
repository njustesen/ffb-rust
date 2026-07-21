package com.fumbbl.ffb.server.db.insert;

import com.fumbbl.ffb.CommonProperty;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust {@code db_user_settings_insert_parameter_list} test.
 * Rust's {@code add_parameter_values} maps to the overloaded
 * {@link DbUserSettingsInsertParameterList#addParameter(String, CommonProperty, String)}.
 */
class DbUserSettingsInsertParameterListTest {

	@Test
	void addParameterValues() {
		DbUserSettingsInsertParameterList list = new DbUserSettingsInsertParameterList();
		list.addParameter("c", CommonProperty.SETTING_SOUND_VOLUME, "v");
		assertEquals(1, list.getParameters().length);
	}
}
