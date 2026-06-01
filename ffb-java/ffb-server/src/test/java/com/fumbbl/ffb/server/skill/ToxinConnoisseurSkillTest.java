package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.mixed.special.ToxinConnoisseur;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ToxinConnoisseurSkillTest {

    private ToxinConnoisseur skill;

    @BeforeEach
    void setUp() {
        skill = new ToxinConnoisseur();
        skill.postConstruct();
    }

    @Test
    void name_is_Toxin_Connoisseur() {
        assertEquals("Toxin Connoisseur", skill.getName());
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
    void class_name_is_ToxinConnoisseur() {
        assertEquals("ToxinConnoisseur", skill.getClass().getSimpleName());
    }
}
