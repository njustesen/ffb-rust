use super::version_change_list::VersionChangeList;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

/// 1:1 translation of com.fumbbl.ffb.client.model.ChangeList (Java class).
pub struct ChangeList {
    versions: Vec<VersionChangeList>,
}

impl ChangeList {
    pub fn instance() -> &'static ChangeList {
        static INSTANCE: OnceLock<ChangeList> = OnceLock::new();
        INSTANCE.get_or_init(ChangeList::new)
    }

    pub fn new() -> Self {
        let mut versions = Vec::new();

        versions.push(VersionChangeList::new("3.2.3")
            .add_bugfix("Weather Mage effect only lasted until end of drive/opponents next turn")
            .add_bugfix("On Linux JVMs past 1.8 it was not possible to close the actions menu by clicking the active player again")
            .add_bugfix("Apply confusion flag if player is prone and fails the respective check (Bone Head, Really Stupid, Animal Savagery)")
            .add_bugfix("Ball & Chain hit by bomb did roll for armour")
            .add_bugfix("Dodgy Snack did not trigger auto marking update")
            .add_bugfix("Multiblock did not generate spp")
            .add_bugfix("Punt: If direction or distance put the ball out of bounds re-rolling the result did not reset the ball being in bounds")
            .add_bugfix("Blessing of Nuffle: Description text was incorrect")
            .add_bugfix("With JVMs newer than 8, range rulers did not show the required roll anymore")
            .add_bugfix("Gaining additional Hatred results in duplication of existing Hatred skill listings")
            .add_bugfix("Bloodlust: When opting to move instead of fouling directly due to failed Bloodlust the game crashed")
            .add_bugfix("Missing Zoat and Spite keywords caused Hatred/Getting Even to show Unknown"));

        versions.push(VersionChangeList::new("3.2.2")
            .add_bugfix("All prayer rolls resulted in Blessing of Nuffle"));

        versions.push(VersionChangeList::new("3.2.1")
            .add_bugfix("Disabling timeout button also disabled the turn timer")
            .add_bugfix("All ruleset: Touchback with only no ball players could result in the ball not being available for the drive")
            .add_behavior_change("All ruleset: In case of a touchback with no players or only no ball players placing the ball in a field does not bounce it anymore")
            .add_bugfix("\"Did not stall\" message was displayed even if there was no potential stalling"));

        versions.push(VersionChangeList::new("3.2.0")
            .add_bugfix("Banned coach does not affect Brilliant Coaching roll")
            .add_bugfix("Prevent staff and technical player types to be eligible to be raised")
            .add_bugfix("Safe Pair of Hands did prevent turnovers")
            .add_bugfix("Leap was not applied when combined with other positive modifiers like Very Long Legs and the resulting modifier was lower than 2")
            .add_bugfix("Player with Fend and Taunt was not able to use Taunt")
            .add_bugfix("Fumbled KTM did not apply stunty to injury roll")
            .add_bugfix("Give and Go did not trigger when a bomb was intercepted/caught")
            .add_feature("Wisdom of the White Dwarf (Star Grombrindal)")
            .add_improvement("Set antialiasing for non-menu text components (mainly affecting Linux environments")
            .add_bugfix("Player Markings for 2020 skills caused false positives in 2025 games")
            .add_bugfix("Blessing of Nuffle was not applied randomly (and still used the old name)")
            .add_bugfix("Thinking Man's Troll could not be used on regeneration re-rolls")
            .add_bugfix("Kaboom! did not work on bouncing bomb")
            .add_bugfix("Fumblerooski was not reverted when player was held in place by tentacles")
            .add_bugfix("Arm Bar against non-dodge players caused a second re-roll option in case of a failed dodge")
            .add_feature("Added game option to turn off timeouts")
            .add_bugfix("Timeout did not work for first turn of a drive"));

        versions.push(VersionChangeList::new("3.1.2")
            .add_bugfix("B&C could perform Multi Block if skill was present")
            .add_improvement("Reword B&C knock out message")
            .add_bugfix("B&C self cas did not generate spp")
            .add_bugfix("For underdog teams with less than 50k treasury the report used treasury was reported incorrectly")
            .add_bugfix("Foul Appearance triggered for the first move after blitzing a player with that skill")
            .add_bugfix("Monstrous Mouth: On both downs chomp states were not always removed properly")
            .add_bugfix("Leader re-roll was not restored if player returned to pitch after KO or surf")
            .add_bugfix("Using Safe Pair Of Hands with Wrestle on ball carrier did not prevent turnover")
            .add_behavior_change("Fallback checkbox for team re-roll on mascot use is now pre-selected")
            .add_bugfix("Diving Catch did not trigger for kick-offs")
            .add_improvement("Message about preventing Strip Ball (Stand Firm, Rooted, Chomped) is only shown if player is actually carrying the ball")
            .add_bugfix("Lone Fouler did not work for Chainsaw fouls")
            .add_bugfix("TTM landing on the ball did allow a pick up")
            .add_bugfix("It was possible to move players on the pitch during mvp selection")
            .add_bugfix("Brilliant coaching message reported a tie when rolls were equal but ignored modifiers")
            .add_improvement("Technical: Game results are now also loaded from backups if game is not in cache anymore")
            .add_bugfix("Eye Gouge: In addition to not assisting, gouged players did also not cancel opposing assist")
            .add_bugfix("Steady Footing was triggered for prone/stunned players being hit by Ball&Chain")
            .add_bugfix("Knocking down team-mates on TTM/KTM did not cause turnovers")
            .add_bugfix("Chomp was not available on blitz during Charge!")
            .add_bugfix("Target selection was not always removed after a blitz")
            .add_bugfix("When using swoop it is now possible to re-roll direction and distance")
            .add_bugfix("Bomb knock down team-mates did not cause a turnover")
            .add_bugfix("Interception rolls where not modified per tacklezone but only by one for being marked")
            .add_bugfix("Prayers were not added to inducement count")
            .add_bugfix("When a Steady Footing player blitzed the ball carrier with a both down (both no block) and got saved by Steady Footing the ball did not bounce")
            .add_bugfix("Interception SPP were not awarded"));

        versions.push(VersionChangeList::new("3.1.1")
            .add_bugfix("Reloading during kick off sequence was broken")
            .add_bugfix("Steady Footing after being pushed on ball did not bounce the ball")
            .add_bugfix("Hypnotic Gaze triggers Foul Appearance")
            .add_bugfix("Black Ink and Zoat Gaze are only available if non-distracted players are in range")
            .add_bugfix("Punt was not available when rushes were exhausted")
            .add_bugfix("High Kick with no open players did not skip sequence")
            .add_improvement("Skip Pick Me Up in last turn of half")
            .add_bugfix("When Mascot re-roll was available without a team re-roll regular re-roll dialog did not react to mascot button and block dialog did not offer mascot")
            .add_bugfix("Sprint was not considered when calculating blitz range"));

        versions.push(VersionChangeList::new("3.1.0")
            .add_bugfix("Stalling: No stalling did not grant cash bonus")
            .add_improvement("Stalling: On turn 7+ do not roll for stalling")
            .add_bugfix("Do not offer Forgo for prone players")
            .add_bugfix("Jump: Declining re-roll granted a free re-roll")
            .add_bugfix("Hypnotic Gaze: rushing twice would end activation before selecting the target")
            .add_bugfix("Brilliant Coaching: Tied result did give no re-roll to either team")
            .add_improvement("iron Man: Only players with AV 10+ or less are eligible")
            .add_bugfix("Under Scrutiny: Only triggers for av breaks")
            .add_bugfix("Strip Ball: no longer works against Stand Firm/Rooted players")
            .add_bugfix("Master Chef was rolled twice also stealing Leader re-rolls")
            .add_bugfix("Selected kick-off results for overtime did not work")
            .add_bugfix("Steady Footing: Attacker blocks defender with both down, defender uses Steady Footing Successfully while attacker fell down, did not cause a turnover")
            .add_improvement("TTM and KTM: reroll choice for Subpar results")
            .add_bugfix("Solid Defence: During player selection it was able to move players around")
            .add_bugfix("Permanent injuries were not removed by regeneration")
            .add_bugfix("Charge: During kickoff blitz, Dodge and Rush skill re-rolls were not available")
            .add_bugfix("Hypnotic Gaze + Bloodlust: prone players now can move/feed after failed Bloodlust instead of auto-gazing and ending activation")
            .add_bugfix("Fixed wording for \"Under Scrutiny\"")
            .add_improvement("Add strip ball cancel message")
            .add_bugfix("Chomped state was not removed if chomper blocked chompee and rolled a skull")
            .add_feature("Implement concession rules")
            .add_bugfix("Dauntless has to be handled before horns")
            .add_bugfix("Support multiple cheering fans assist per team")
            .add_bugfix("Ensure only players in reserves can be selected for prayers")
            .add_feature("Support icon set index for player icons"));

        versions.push(VersionChangeList::new("3.0.0")
            .set_description("First version of 2025 rules, a.k.a. 3rd Season - beware of bugs"));

        Self { versions }
    }

    pub fn get_versions(&self) -> &[VersionChangeList] {
        &self.versions
    }

    pub fn finger_print(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.versions[0].hash(&mut hasher);
        hasher.finish().to_string()
    }
}

impl Default for ChangeList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_eight_versions() {
        let list = ChangeList::new();
        assert_eq!(list.get_versions().len(), 8);
    }

    #[test]
    fn first_version_is_3_2_3() {
        let list = ChangeList::new();
        assert_eq!(list.get_versions()[0].get_version(), "3.2.3");
        assert!(list.get_versions()[0].has_bugfixes());
    }

    #[test]
    fn last_version_is_3_0_0_with_description() {
        let list = ChangeList::new();
        let last = list.get_versions().last().unwrap();
        assert_eq!(last.get_version(), "3.0.0");
        assert!(last.has_description());
    }

    #[test]
    fn finger_print_is_stable() {
        let a = ChangeList::new();
        let b = ChangeList::new();
        assert_eq!(a.finger_print(), b.finger_print());
    }

    #[test]
    fn instance_is_singleton() {
        let a = ChangeList::instance();
        let b = ChangeList::instance();
        assert_eq!(a.finger_print(), b.finger_print());
    }
}
