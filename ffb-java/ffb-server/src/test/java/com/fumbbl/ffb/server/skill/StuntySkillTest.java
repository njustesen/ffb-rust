package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Stunty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class StuntySkillTest {

    private Stunty skill;

    @BeforeEach
    void setUp() {
        skill = new Stunty();
        skill.postConstruct();
    }

    @Test
    void name_is_Stunty() {
        assertEquals("Stunty", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_ignore_tacklezones_when_dodging_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.ignoreTacklezonesWhenDodging));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Stunty.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
