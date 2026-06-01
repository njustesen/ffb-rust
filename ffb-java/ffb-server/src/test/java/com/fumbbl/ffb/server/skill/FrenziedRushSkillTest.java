package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.FrenziedRush;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class FrenziedRushSkillTest {

    private FrenziedRush skill;

    @BeforeEach
    void setUp() {
        skill = new FrenziedRush();
        skill.postConstruct();
    }

    @Test
    void name_is_Frenzied_Rush() {
        assertEquals("Frenzied Rush", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_gain_frenzy_for_blitz_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canGainFrenzyForBlitz));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = FrenziedRush.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
