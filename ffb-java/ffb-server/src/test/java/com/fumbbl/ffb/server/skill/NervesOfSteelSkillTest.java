package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.NervesOfSteel;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class NervesOfSteelSkillTest {

    private NervesOfSteel skill;

    @BeforeEach
    void setUp() {
        skill = new NervesOfSteel();
        skill.postConstruct();
    }

    @Test
    void name_is_Nerves_of_Steel() {
        assertEquals("Nerves of Steel", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_ignore_tacklezones_when_passing_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.ignoreTacklezonesWhenPassing));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = NervesOfSteel.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
