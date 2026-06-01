package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.mixed.special.WatchOut;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class WatchOutSkillTest {

    private WatchOut skill;

    @BeforeEach
    void setUp() {
        skill = new WatchOut();
        skill.postConstruct();
    }

    @Test
    void name_is_WatchOut() {
        assertEquals("Watch Out!", skill.getName());
    }

    @Test
    void category_is_TRAIT() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_two_edition_annotations() {
        RulesCollection[] annotations = WatchOut.class.getDeclaredAnnotationsByType(RulesCollection.class);
        assertEquals(2, annotations.length);
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}
