package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.CloudBurster;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class CloudBursterSkillTest {

    private CloudBurster skill;

    @BeforeEach
    void setUp() {
        skill = new CloudBurster();
        skill.postConstruct();
    }

    @Test
    void name_is_Cloud_Burster() {
        assertEquals("Cloud Burster", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_can_force_interception_reroll_of_long_passes_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canForceInterceptionRerollOfLongPasses));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = CloudBurster.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
