package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2025.special.DwarvenScourge;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DwarvenScourgeSkillTest {

    private DwarvenScourge skill;

    @BeforeEach
    void setUp() {
        skill = new DwarvenScourge();
        skill.postConstruct();
    }

    @Test
    void name_is_Dwarven_Scourge() {
        assertEquals("Dwarven Scourge", skill.getName());
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
    void is_bb2025_edition() {
        RulesCollection annotation = DwarvenScourge.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
