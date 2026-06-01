package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.BlastIt;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BlastItSkillTest {

    private BlastIt skill;

    @BeforeEach
    void setUp() {
        skill = new BlastIt();
        skill.postConstruct();
    }

    @Test
    void name_is_Blast_It() {
        assertEquals("Blast It!", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_re_roll_hmp_scatter_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canReRollHmpScatter));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = BlastIt.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
