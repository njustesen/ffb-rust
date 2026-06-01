package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2016.Accurate;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class AccurateSkillTest {

    private Accurate skill;

    @BeforeEach
    void setUp() {
        skill = new Accurate();
        skill.postConstruct();
    }

    @Test
    void name_is_Accurate() {
        assertEquals("Accurate", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Accurate.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}
