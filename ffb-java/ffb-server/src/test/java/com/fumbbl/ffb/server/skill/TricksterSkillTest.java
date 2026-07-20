package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.Trickster;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.*;

class TricksterSkillTest {

    private Trickster skill;

    @BeforeEach
    void setUp() {
        skill = new Trickster();
        skill.postConstruct();
    }

    @Test
    void name_is_Trickster() {
        assertEquals("Trickster", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_canMoveBeforeBeingBlocked_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMoveBeforeBeingBlocked),
            "Trickster must register canMoveBeforeBeingBlocked so the player may move away before being blocked");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Trickster does not force follow-up");
    }

    @Test
    void is_bb2020_and_bb2025_edition() {
        RulesCollection[] annotations = Trickster.class.getDeclaredAnnotationsByType(RulesCollection.class);
        assertTrue(Arrays.stream(annotations).anyMatch(a -> a.value() == RulesCollection.Rules.BB2020),
            "Trickster must be available in BB2020");
        assertTrue(Arrays.stream(annotations).anyMatch(a -> a.value() == RulesCollection.Rules.BB2025),
            "Trickster must be available in BB2025");
    }
}
