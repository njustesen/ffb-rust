package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.JumpUp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class JumpUpSkillTest {

    private JumpUp skill;

    @BeforeEach
    void setUp() {
        skill = new JumpUp();
        skill.postConstruct();
    }

    @Test
    void name_is_Jump_Up() {
        assertEquals("Jump Up", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_stand_up_for_free_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canStandUpForFree));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = JumpUp.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
