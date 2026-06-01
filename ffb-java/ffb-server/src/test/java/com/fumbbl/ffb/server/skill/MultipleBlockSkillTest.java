package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.MultipleBlock;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class MultipleBlockSkillTest {

    private MultipleBlock skill;

    @BeforeEach
    void setUp() {
        skill = new MultipleBlock();
        skill.postConstruct();
    }

    @Test
    void name_is_Multiple_Block() {
        assertEquals("Multiple Block", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_can_block_more_than_once_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canBlockMoreThanOnce));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = MultipleBlock.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
