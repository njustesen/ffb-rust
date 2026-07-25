package com.fumbbl.ffb.xml;

import org.junit.jupiter.api.Test;
import org.xml.sax.helpers.AttributesImpl;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-model/src/xml/util_xml.rs tests.
 */
public class UtilXmlTest {

	private AttributesImpl atts(String name, String value) {
		AttributesImpl a = new AttributesImpl();
		a.addAttribute("", name, name, "CDATA", value);
		return a;
	}

	private AttributesImpl empty() {
		return new AttributesImpl();
	}

	// rust: get_string_attribute_trims_and_returns
	@Test
	public void getStringAttributeTrimsAndReturns() {
		assertEquals("42", UtilXml.getStringAttribute(atts("id", " 42 "), "id"));
	}

	// rust: get_string_attribute_missing_returns_none
	@Test
	public void getStringAttributeMissingReturnsNone() {
		assertNull(UtilXml.getStringAttribute(empty(), "id"));
	}

	// rust: get_int_attribute_or_parses_value
	@Test
	public void getIntAttributeOrParsesValue() {
		assertEquals(3, UtilXml.getIntAttribute(atts("size", "3"), "size", -1));
	}

	// rust: get_int_attribute_or_missing_returns_default
	@Test
	public void getIntAttributeOrMissingReturnsDefault() {
		assertEquals(-1, UtilXml.getIntAttribute(empty(), "size", -1));
	}

	// rust: get_int_attribute_missing_returns_zero
	@Test
	public void getIntAttributeMissingReturnsZero() {
		assertEquals(0, UtilXml.getIntAttribute(empty(), "size"));
	}

	// rust: get_boolean_attribute_true
	@Test
	public void getBooleanAttributeTrue() {
		assertTrue(UtilXml.getBooleanAttribute(atts("recovering", "true"), "recovering"));
	}

	// rust: get_boolean_attribute_missing_is_false
	@Test
	public void getBooleanAttributeMissingIsFalse() {
		assertFalse(UtilXml.getBooleanAttribute(empty(), "recovering"));
	}

	// rust: get_boolean_attribute_case_insensitive
	@Test
	public void getBooleanAttributeCaseInsensitive() {
		assertTrue(UtilXml.getBooleanAttribute(atts("recovering", "TRUE"), "recovering"));
	}
}
