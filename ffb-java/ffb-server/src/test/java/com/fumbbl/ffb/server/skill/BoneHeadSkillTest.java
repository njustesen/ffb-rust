package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.BoneHead;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BoneHeadSkillTest {

    private BoneHead skill;

    @BeforeEach
    void setUp() {
        skill = new BoneHead();
        skill.postConstruct();
    }

    @Test
    void name_is_Bone_Head() {
        assertEquals("Bone Head", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_applies_confusion_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.appliesConfusion));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = BoneHead.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
