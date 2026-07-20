package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.BloodLust;
import com.fumbbl.ffb.skill.mixed.Bloodlust;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.*;

class BloodLustSkillTest {

    private BloodLust skill;

    @BeforeEach
    void setUp() {
        skill = new BloodLust();
        skill.postConstruct();
    }

    @Test
    void name_is_Blood_Lust() {
        assertEquals("Blood Lust", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = BloodLust.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}

/**
 * Test for the mixed-edition {@link Bloodlust} (BB2020/BB2025). Named BloodlustMixedSkillTest because BloodlustSkillTest.class would case-collide with BloodLustSkillTest.class on Windows. Lives in this file because
 * Windows file systems cannot hold both BloodLustSkillTest.java and BloodlustSkillTest.java
 * (case-insensitive collision with the bb2016 test above).
 */
class BloodlustMixedSkillTest {

    private Bloodlust skill;

    @BeforeEach
    void setUp() {
        skill = new Bloodlust();
        skill.postConstruct();
    }

    @Test
    void name_is_Bloodlust() {
        assertEquals("Bloodlust", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_enableStandUpAndEndBlitzAction_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.enableStandUpAndEndBlitzAction),
            "Bloodlust must register enableStandUpAndEndBlitzAction so the vampire can still stand up and end a blitz on a failed roll");
    }

    @Test
    void has_needsToRollForActionBlockingIsEasier_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.needsToRollForActionBlockingIsEasier),
            "Bloodlust must register needsToRollForActionBlockingIsEasier so the vampire rolls when declaring an action");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Bloodlust does not force follow-up");
    }

    @Test
    void is_bb2020_and_bb2025_edition() {
        RulesCollection[] annotations = Bloodlust.class.getDeclaredAnnotationsByType(RulesCollection.class);
        assertTrue(Arrays.stream(annotations).anyMatch(a -> a.value() == RulesCollection.Rules.BB2020),
            "Bloodlust must be available in BB2020");
        assertTrue(Arrays.stream(annotations).anyMatch(a -> a.value() == RulesCollection.Rules.BB2025),
            "Bloodlust must be available in BB2025");
    }
}
