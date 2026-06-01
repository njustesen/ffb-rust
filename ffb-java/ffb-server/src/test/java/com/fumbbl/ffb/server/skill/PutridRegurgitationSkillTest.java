package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.PutridRegurgitation;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PutridRegurgitationSkillTest {

    private PutridRegurgitation skill;

    @BeforeEach
    void setUp() {
        skill = new PutridRegurgitation();
        skill.postConstruct();
    }

    @Test
    void name_is_Putrid_Regurgitation() {
        assertEquals("Putrid Regurgitation", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_use_vomit_after_block_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canUseVomitAfterBlock));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = PutridRegurgitation.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
