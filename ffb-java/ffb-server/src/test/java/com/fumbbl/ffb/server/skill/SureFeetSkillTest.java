package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.ReRolledActions;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2016.SureFeet;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SureFeetSkillTest {

    private SureFeet skill;

    @BeforeEach
    void setUp() {
        skill = new SureFeet();
        skill.postConstruct();
    }

    @Test
    void name_is_Sure_Feet() {
        assertEquals("Sure Feet", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_sure_feet_reroll_source() {
        assertNotNull(skill.getRerollSource(ReRolledActions.GO_FOR_IT),
            "Sure Feet registers a reroll source for GFI rolls");
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = SureFeet.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
