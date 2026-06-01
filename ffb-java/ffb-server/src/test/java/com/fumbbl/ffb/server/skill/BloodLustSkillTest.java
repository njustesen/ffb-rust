package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2016.BloodLust;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BloodLustSkillTest {

    private BloodLust skill;

    @BeforeEach
    void setUp() {
        skill = new BloodLust();
        skill.postConstruct();
    }

    @Test
    void name_is_Blood_Lust() {
        assertEquals("Blood Lust", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = BloodLust.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}
