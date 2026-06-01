package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.PogoStick;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PogoStickSkillTest {

    private PogoStick skill;

    @BeforeEach
    void setUp() {
        skill = new PogoStick();
        skill.postConstruct();
    }

    @Test
    void name_is_Pogo_Stick() {
        assertEquals("Pogo Stick", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_leap_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canLeap));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = PogoStick.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
