package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2025.special.KrumpAndSmash;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class KrumpAndSmashSkillTest {

    private KrumpAndSmash skill;

    @BeforeEach
    void setUp() {
        skill = new KrumpAndSmash();
        skill.postConstruct();
    }

    @Test
    void name_is_Krump_and_Smash() {
        assertEquals("Krump and Smash", skill.getName());
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
        RulesCollection annotation = KrumpAndSmash.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
