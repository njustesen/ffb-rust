package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.Horns;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class HornsSkillTest {

    private Horns skill;

    @BeforeEach
    void setUp() {
        skill = new Horns();
        skill.postConstruct();
    }

    @Test
    void name_is_Horns() {
        assertEquals("Horns", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_add_strength_on_blitz_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.addStrengthOnBlitz));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Horns.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
