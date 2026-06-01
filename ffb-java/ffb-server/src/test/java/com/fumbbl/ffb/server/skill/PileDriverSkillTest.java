package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.PileDriver;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PileDriverSkillTest {

    private PileDriver skill;

    @BeforeEach
    void setUp() {
        skill = new PileDriver();
        skill.postConstruct();
    }

    @Test
    void name_is_Pile_Driver() {
        assertEquals("Pile Driver", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_can_foul_after_block_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canFoulAfterBlock));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = PileDriver.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
