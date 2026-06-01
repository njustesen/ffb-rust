package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2020.special.DwarfenScourge;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DwarfenScourgeSkillTest {

    private DwarfenScourge skill;

    @BeforeEach
    void setUp() {
        skill = new DwarfenScourge();
        skill.postConstruct();
    }

    @Test
    void name_is_Dwarfen_Scourge() {
        assertEquals("Dwarfen Scourge", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = DwarfenScourge.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}
