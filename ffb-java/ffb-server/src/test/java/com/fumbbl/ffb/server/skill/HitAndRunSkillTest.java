package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.HitAndRun;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class HitAndRunSkillTest {

    private HitAndRun skill;

    @BeforeEach
    void setUp() {
        skill = new HitAndRun();
        skill.postConstruct();
    }

    @Test
    void name_is_Hit_And_Run() {
        assertEquals("Hit And Run", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_move_after_block_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMoveAfterBlock));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = HitAndRun.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
