package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.HypnoticGaze;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class HypnoticGazeSkillTest {

    private HypnoticGaze skill;

    @BeforeEach
    void setUp() {
        skill = new HypnoticGaze();
        skill.postConstruct();
    }

    @Test
    void name_is_Hypnotic_Gaze() {
        assertEquals("Hypnotic Gaze", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_inflicts_confusion_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.inflictsConfusion));
    }

    @Test
    void has_can_gaze_during_move_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canGazeDuringMove));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = HypnoticGaze.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
