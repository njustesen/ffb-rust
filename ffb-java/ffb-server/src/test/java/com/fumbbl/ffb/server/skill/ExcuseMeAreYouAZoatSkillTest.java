package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.ExcuseMeAreYouAZoat;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ExcuseMeAreYouAZoatSkillTest {

    private ExcuseMeAreYouAZoat skill;

    @BeforeEach
    void setUp() {
        skill = new ExcuseMeAreYouAZoat();
        skill.postConstruct();
    }

    @Test
    void name_contains_Zoat() {
        assertTrue(skill.getName().contains("Zoat"));
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_gain_gaze_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canGainGaze));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = ExcuseMeAreYouAZoat.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
