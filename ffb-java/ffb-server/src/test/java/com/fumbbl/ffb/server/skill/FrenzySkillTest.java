package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Frenzy;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class FrenzySkillTest {

    private Frenzy skill;

    @BeforeEach
    void setUp() {
        skill = new Frenzy();
        skill.postConstruct();
    }

    @Test
    void name_is_Frenzy() {
        assertEquals("Frenzy", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_forceFollowup_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Frenzy must always follow up when the opponent is pushed back");
    }

    @Test
    void has_forceSecondBlock_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.forceSecondBlock),
            "Frenzy must throw a second block after a Pushed or Defender Stumbles result");
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Frenzy.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation, "Frenzy must have a RulesCollection annotation");
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
