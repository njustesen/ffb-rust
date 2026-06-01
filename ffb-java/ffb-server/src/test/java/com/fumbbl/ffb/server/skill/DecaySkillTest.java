package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Decay;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DecaySkillTest {

    private Decay skill;

    @BeforeEach
    void setUp() {
        skill = new Decay();
        skill.postConstruct();
    }

    @Test
    void name_is_Decay() {
        assertEquals("Decay", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_requires_second_casualty_roll_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.requiresSecondCasualtyRoll));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Decay.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
