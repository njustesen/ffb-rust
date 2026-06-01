package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Leap;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class LeapSkillTest {

    private Leap skill;

    @BeforeEach
    void setUp() {
        skill = new Leap();
        skill.postConstruct();
    }

    @Test
    void name_is_Leap() {
        assertEquals("Leap", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_leap_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canLeap));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Leap.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
