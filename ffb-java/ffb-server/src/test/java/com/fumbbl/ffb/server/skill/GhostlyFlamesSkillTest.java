package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2020.special.GhostlyFlames;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class GhostlyFlamesSkillTest {

    private GhostlyFlames skill;

    @BeforeEach
    void setUp() {
        skill = new GhostlyFlames();
        skill.postConstruct();
    }

    @Test
    void name_is_Ghostly_Flames() {
        assertEquals("Ghostly Flames", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = GhostlyFlames.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
