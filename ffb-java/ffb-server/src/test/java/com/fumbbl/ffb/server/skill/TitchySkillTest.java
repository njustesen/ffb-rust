package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Titchy;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TitchySkillTest {

    private Titchy skill;

    @BeforeEach
    void setUp() {
        skill = new Titchy();
        skill.postConstruct();
    }

    @Test
    void name_is_Titchy() {
        assertEquals("Titchy", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_no_tacklezone_for_dodging_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.hasNoTacklezoneForDodging));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Titchy.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
