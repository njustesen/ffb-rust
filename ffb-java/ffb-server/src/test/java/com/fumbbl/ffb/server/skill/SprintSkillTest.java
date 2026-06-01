package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.Sprint;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SprintSkillTest {

    private Sprint skill;

    @BeforeEach
    void setUp() {
        skill = new Sprint();
        skill.postConstruct();
    }

    @Test
    void name_is_Sprint() {
        assertEquals("Sprint", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_make_extra_gfi_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMakeAnExtraGfi));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Sprint.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
