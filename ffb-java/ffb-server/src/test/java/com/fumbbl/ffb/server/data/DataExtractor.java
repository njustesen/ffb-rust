package com.fumbbl.ffb.server.data;

import org.junit.jupiter.api.Test;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.NodeList;

import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.stream.Collectors;

/**
 * Extracts all game data from Java source and roster XMLs to JSON files in ffb-rust/data/.
 *
 * Run with: mvn test -pl ffb-server -Dtest=DataExtractor -DfailIfNoTests=false
 */
public class DataExtractor {

    private static final String ROSTER_DIR = "../ffb-server/rosters";
    private static final String OUTPUT_DIR = "../ffb-rust/data";

    @Test
    public void extractAll() throws Exception {
        extractRosters();
        writeInducementsBb2020();
        writeInducementsBb2016();
        writeInducementsBb2025();
        writePrayersBb2020();
        writePrayersBb2025();
        writeSeriousInjuries();
        System.out.println("Data extraction complete. Output: " + OUTPUT_DIR);
    }

    // -------------------------------------------------------------------------
    // Roster Extraction
    // -------------------------------------------------------------------------

    private void extractRosters() throws Exception {
        File rosterDir = new File(ROSTER_DIR);
        if (!rosterDir.exists()) {
            System.out.println("Roster dir not found, trying absolute path...");
            rosterDir = new File("C:/Users/Admin/niels/ffb/ffb-server/rosters");
        }
        File outDir = resolveOutputDir();
        mkdirs(new File(outDir, "rosters/bb2020"));
        mkdirs(new File(outDir, "rosters/bb2016"));
        mkdirs(new File(outDir, "rosters/bb2025"));
        mkdirs(new File(outDir, "star_players"));

        DocumentBuilderFactory dbf = DocumentBuilderFactory.newInstance();
        DocumentBuilder db = dbf.newDocumentBuilder();

        List<Map<String, Object>> allStarPlayers = new ArrayList<>();
        Map<String, String> starPlayerSeen = new HashMap<>();

        for (File xmlFile : Objects.requireNonNull(rosterDir.listFiles(f -> f.getName().endsWith(".xml")))) {
            Document doc = db.parse(xmlFile);
            doc.getDocumentElement().normalize();

            String rosterName = getText(doc, "name");
            String rosterId = doc.getDocumentElement().getAttribute("id");
            if (rosterId == null || rosterId.isEmpty()) {
                rosterId = doc.getDocumentElement().getAttribute("team");
            }
            int rerollCost = getIntText(doc, "reRollCost", 50000);
            int maxRerolls = getIntText(doc, "maxReRolls", 8);
            boolean hasApothecary = !"false".equals(getText(doc, "apothecary"));
            boolean isUndead = "true".equals(getText(doc, "undead"));
            boolean hasNecromancer = "true".equals(getText(doc, "necromancer"));
            String nameGenerator = getText(doc, "nameGenerator");

            // Parse keywords
            List<String> rosterKeywords = new ArrayList<>();
            NodeList kwNodes = doc.getElementsByTagName("keyword");
            for (int k = 0; k < kwNodes.getLength(); k++) {
                Element kwEl = (Element) kwNodes.item(k);
                if (kwEl.getParentNode().getNodeName().equals("keywords")) {
                    rosterKeywords.add(kwEl.getTextContent().trim());
                }
            }

            List<Map<String, Object>> positions = new ArrayList<>();
            List<Map<String, Object>> starPlayers = new ArrayList<>();
            List<Map<String, Object>> infamousStaff = new ArrayList<>();

            NodeList positionNodes = doc.getElementsByTagName("position");
            for (int i = 0; i < positionNodes.getLength(); i++) {
                Element pos = (Element) positionNodes.item(i);
                String posId = pos.getAttribute("id");
                String posName = getChildText(pos, "name");
                String displayName = getChildText(pos, "displayName");
                if (displayName == null || displayName.isEmpty()) displayName = posName;
                String type = getChildText(pos, "type");
                int quantity = getChildInt(pos, "quantity", 16);
                int cost = getChildInt(pos, "cost", 0);
                int ma = getChildInt(pos, "movement", 6);
                int st = getChildInt(pos, "strength", 3);
                int ag = getChildInt(pos, "agility", 3);
                int pa = getChildInt(pos, "passing", -1);  // -1 = no PA (-)
                String passingStr = getChildText(pos, "passing");
                if (passingStr == null || passingStr.isEmpty() || passingStr.equals("-")) pa = -1;
                int av = getChildInt(pos, "armour", 8);
                String shorthand = getChildText(pos, "shorthand");
                boolean undead = "true".equals(getChildText(pos, "undead"));
                boolean thrall = "true".equals(getChildText(pos, "thrall"));

                // Skills
                List<String> skills = new ArrayList<>();
                Map<String, Object> skillsWithValues = new LinkedHashMap<>();
                NodeList skillNodes = pos.getElementsByTagName("skill");
                for (int j = 0; j < skillNodes.getLength(); j++) {
                    Element sk = (Element) skillNodes.item(j);
                    if (!sk.getParentNode().getNodeName().equals("skillList")) continue;
                    String skillName = sk.getTextContent().trim();
                    String value = sk.getAttribute("value");
                    if (value != null && !value.isEmpty()) {
                        skillsWithValues.put(skillName, Integer.parseInt(value));
                    } else {
                        skills.add(skillName);
                    }
                }
                // merge skills with values into skills list for simplicity
                List<Object> allSkills = new ArrayList<>(skills);
                for (Map.Entry<String, Object> e : skillsWithValues.entrySet()) {
                    Map<String, Object> sv = new LinkedHashMap<>();
                    sv.put("name", e.getKey());
                    sv.put("value", e.getValue());
                    allSkills.add(sv);
                }

                // Skill categories
                Map<String, List<String>> skillCategories = new LinkedHashMap<>();
                NodeList catList = pos.getElementsByTagName("skillCategoryList");
                if (catList.getLength() > 0) {
                    Element catEl = (Element) catList.item(0);
                    for (String catType : Arrays.asList("normal", "double")) {
                        NodeList cats = catEl.getElementsByTagName(catType);
                        List<String> catNames = new ArrayList<>();
                        for (int c = 0; c < cats.getLength(); c++) {
                            catNames.add(cats.item(c).getTextContent().trim());
                        }
                        if (!catNames.isEmpty()) skillCategories.put(catType, catNames);
                    }
                }

                Map<String, Object> entry = new LinkedHashMap<>();
                entry.put("id", posId);
                entry.put("name", posName);
                entry.put("display_name", displayName);
                entry.put("type", type != null ? type : "Regular");
                entry.put("quantity", quantity);
                entry.put("cost", cost);
                entry.put("ma", ma);
                entry.put("st", st);
                entry.put("ag", ag);
                entry.put("pa", pa);
                entry.put("av", av);
                if (shorthand != null && !shorthand.isEmpty()) entry.put("shorthand", shorthand);
                if (undead) entry.put("undead", true);
                if (thrall) entry.put("thrall", true);
                if (!allSkills.isEmpty()) entry.put("skills", allSkills);
                if (!skillCategories.isEmpty()) entry.put("skill_categories", skillCategories);

                if ("Star".equals(type) || "HiredStar".equals(type)) {
                    // star players available for this race
                    if (!starPlayerSeen.containsKey(posId)) {
                        starPlayerSeen.put(posId, posId);
                        Map<String, Object> sp = new LinkedHashMap<>(entry);
                        sp.put("available_for", new ArrayList<>(Collections.singletonList(rosterName)));
                        allStarPlayers.add(sp);
                    } else {
                        // add this race to existing star player
                        allStarPlayers.stream()
                            .filter(s -> posId.equals(s.get("id")))
                            .findFirst()
                            .ifPresent(s -> ((List<String>) s.get("available_for")).add(rosterName));
                    }
                    starPlayers.add(entry);
                } else if ("Infamous Staff".equals(type)) {
                    infamousStaff.add(entry);
                } else {
                    positions.add(entry);
                }
            }

            Map<String, Object> roster = new LinkedHashMap<>();
            roster.put("id", rosterId);
            roster.put("name", rosterName);
            roster.put("reroll_cost", rerollCost);
            roster.put("max_rerolls", maxRerolls);
            roster.put("apothecary", hasApothecary);
            roster.put("undead", isUndead);
            roster.put("necromancer", hasNecromancer);
            if (nameGenerator != null && !nameGenerator.isEmpty()) roster.put("name_generator", nameGenerator);
            if (!rosterKeywords.isEmpty()) roster.put("keywords", rosterKeywords);
            roster.put("positions", positions);
            if (!infamousStaff.isEmpty()) roster.put("infamous_staff", infamousStaff);

            String baseName = xmlFile.getName().replace(".xml", "");
            String raceName = rosterName.toLowerCase().replace(" ", "_").replace("'", "");
            // Write to bb2020 (primary), bb2016, bb2025 (same source for now - editions differ by rules, not roster data)
            for (String edition : Arrays.asList("bb2020", "bb2016", "bb2025")) {
                writeJson(new File(outDir, "rosters/" + edition + "/" + baseName + ".json"), roster);
            }
            System.out.println("Extracted roster: " + rosterName + " (" + baseName + ")");
        }

        // Write all star players
        Map<String, Object> spWrapper = new LinkedHashMap<>();
        spWrapper.put("star_players", allStarPlayers);
        writeJson(new File(outDir, "star_players/all_editions.json"), spWrapper);
        System.out.println("Extracted " + allStarPlayers.size() + " star players");
    }

    // -------------------------------------------------------------------------
    // Inducements - BB2020
    // -------------------------------------------------------------------------

    private void writeInducementsBb2020() throws Exception {
        List<Map<String, Object>> inducements = new ArrayList<>();

        inducements.add(inducement("bloodweiserKegs", "Bloodweiser Kegs", "Bloodweiser Keg", 50000, 3, "KNOCKOUT_RECOVERY", null));
        inducements.add(inducement("bribes", "Bribes", "Bribe", 100000, 3, "AVOID_BAN", null));
        inducements.add(inducement("briberyAndCorruption", "Bribery and Corruption ReRoll", "Bribery and Corruption ReRoll", 0, 0, "REROLL_ARGUE", "special_rule:BRIBERY_AND_CORRUPTION"));
        inducements.add(inducement("halflingMasterChef", "Halfling Master Chef", "Halfling Master Chef", 300000, 1, "STEAL_REROLL", null));
        inducements.add(inducement("mortuaryAssistant", "Mortuary Assistant", "Mortuary Assistant", 100000, 1, "REGENERATION", "special_rule:SYLVANIAN_SPOTLIGHT"));
        inducements.add(inducement("plagueDoctor", "Plague Doctor", "Plague Doctor", 200000, 2, "REGENERATION,APOTHECARY_JOURNEYMEN", "special_rule:FAVOURED_OF_NURGLE"));
        inducements.add(inducement("riotousRookies", "Riotous Rookies", "Riotous Rookies", 100000, 3, "ADD_LINEMEN", "special_rule:LOW_COST_LINEMEN"));
        inducements.add(inducement("tempCheerleader", "Temp Agency Cheerleaders", "Temp Agency Cheerleader", 20000, 5, "ADD_CHEERLEADER", null));
        inducements.add(inducement("partTimeCoach", "Part-time Assistant Coaches", "Part-time Assistant Coach", 10000, 6, "ADD_COACH", null));
        inducements.add(inducement("weatherMage", "Weather Mage", "Weather Mage", 30000, 1, "GAME_MODIFICATION", null));
        inducements.add(inducement("biasedReferee", "Biased Referee", "Biased Referee", 130000, 1, "GAME_MODIFICATION", null));
        inducements.add(inducement("bugmansXXXXXX", "Bugman's XXXXXX", "Bugman's XXXXXX", 100000, 1, "ADD_COACH", null));
        inducements.add(inducement("throwARock", "Throw a Rock", "Throw a Rock", 0, 0, "GAME_MODIFICATION", "special_rule:SPIRITED_CROWD"));
        inducements.add(inducement("prayers", "Prayers of Nuffle", "Prayer of Nuffle", 100000, 3, "GAME_MODIFICATION", null));
        inducements.add(inducement("infamousStaff", "Infamous Coaching Staff", "Infamous Coaching Staff", 0, 0, "ADD_INFAMOUS_STAFF", "roster_staff"));
        inducements.add(inducement("starPlayer", "Star Player", "Star Player", 0, 2, "ADD_STAR_PLAYER", "roster_star"));

        writeEditionInducements("bb2020", inducements);
    }

    private void writeInducementsBb2016() throws Exception {
        List<Map<String, Object>> inducements = new ArrayList<>();

        inducements.add(inducement("bloodweiserBabes", "Bloodweiser Babes", "Bloodweiser Babe", 50000, 2, "KNOCKOUT_RECOVERY", null));
        inducements.add(inducement("assistantCoaches", "Assistant Coaches", "Assistant Coach", 10000, 6, "ADD_COACH", null));
        inducements.add(inducement("cheerleaders", "Cheerleaders", "Cheerleader", 10000, 12, "ADD_CHEERLEADER", null));
        inducements.add(inducement("bribes", "Bribes", "Bribe", 100000, 3, "AVOID_BAN", null));
        inducements.add(inducement("halflingMasterChef", "Halfling Master Chef", "Halfling Master Chef", 300000, 1, "STEAL_REROLL", null));
        inducements.add(inducement("wizard", "Wizard", "Wizard", 150000, 1, "SPELL", null));
        inducements.add(inducement("igor", "Igor", "Igor", 100000, 1, "REGENERATION", "special_rule:SYLVANIAN_SPOTLIGHT"));
        inducements.add(inducement("cards", "Cards", "Card", 15000, 3, "CARD", null));
        inducements.add(inducement("starPlayer", "Star Player", "Star Player", 0, 2, "ADD_STAR_PLAYER", "roster_star"));

        writeEditionInducements("bb2016", inducements);
    }

    private void writeInducementsBb2025() throws Exception {
        List<Map<String, Object>> inducements = new ArrayList<>();

        inducements.add(inducement("teamMascot", "Team Mascot", "Team Mascot", 100000, 1, "REROLL", null));
        inducements.add(inducement("bloodweiserKegs", "Blitzer's Best Kegs", "Blitzer's Best Keg", 50000, 3, "KNOCKOUT_RECOVERY", null));
        inducements.add(inducement("bribes", "Bribes", "Bribe", 100000, 3, "AVOID_BAN", null));
        inducements.add(inducement("briberyAndCorruption", "Bribery and Corruption ReRoll", "Bribery and Corruption ReRoll", 0, 0, "REROLL_ARGUE", "special_rule:BRIBERY_AND_CORRUPTION"));
        inducements.add(inducement("halflingMasterChef", "Halfling Master Chef", "Halfling Master Chef", 300000, 1, "STEAL_REROLL", null));
        inducements.add(inducement("mortuaryAssistant", "Mortuary Assistant", "Mortuary Assistant", 100000, 1, "REGENERATION", "special_rule:SYLVANIAN_SPOTLIGHT"));
        inducements.add(inducement("plagueDoctor", "Plague Doctor", "Plague Doctor", 200000, 2, "REGENERATION,APOTHECARY_JOURNEYMEN", "special_rule:FAVOURED_OF_NURGLE"));
        inducements.add(inducement("riotousRookies", "Riotous Rookies", "Riotous Rookies", 100000, 3, "ADD_LINEMEN", "special_rule:LOW_COST_LINEMEN"));
        inducements.add(inducement("tempCheerleader", "Temp Agency Cheerleaders", "Temp Agency Cheerleader", 20000, 5, "ADD_CHEERLEADER", null));
        inducements.add(inducement("partTimeCoach", "Part-time Assistant Coaches", "Part-time Assistant Coach", 10000, 6, "ADD_COACH", null));
        inducements.add(inducement("weatherMage", "Weather Mage", "Weather Mage", 30000, 1, "GAME_MODIFICATION", null));
        inducements.add(inducement("biasedReferee", "Biased Referee", "Biased Referee", 130000, 1, "GAME_MODIFICATION", null));
        inducements.add(inducement("bugmansXXXXXX", "Bugman's XXXXXX", "Bugman's XXXXXX", 100000, 1, "ADD_COACH", null));
        inducements.add(inducement("throwARock", "Throw a Rock", "Throw a Rock", 0, 0, "GAME_MODIFICATION", "special_rule:SPIRITED_CROWD"));
        inducements.add(inducement("prayers", "Prayers of Nuffle", "Prayer of Nuffle", 100000, 3, "GAME_MODIFICATION", null));
        inducements.add(inducement("infamousStaff", "Infamous Coaching Staff", "Infamous Coaching Staff", 0, 0, "ADD_INFAMOUS_STAFF", "roster_staff"));
        inducements.add(inducement("starPlayer", "Star Player", "Star Player", 0, 2, "ADD_STAR_PLAYER", "roster_star"));

        writeEditionInducements("bb2025", inducements);
    }

    // -------------------------------------------------------------------------
    // Prayers - BB2020
    // -------------------------------------------------------------------------

    private void writePrayersBb2020() throws Exception {
        List<Map<String, Object>> prayers = new ArrayList<>();
        prayers.add(prayer(1, "TREACHEROUS_TRAPDOOR", "Treacherous Trapdoor",
            "Trapdoors appear. On a roll of 1 a player stepping on them falls through them",
            "UNTIL_END_OF_HALF", true, false, null, null));
        prayers.add(prayer(2, "FRIENDS_WITH_THE_REF", "Friends with the Ref",
            "Argue the call succeeds on 5+",
            "UNTIL_END_OF_GAME", true, false, null, null));
        prayers.add(prayer(3, "STILETTO", "Stiletto",
            "One random player available to play during this drive without Loner gains Stab",
            "UNTIL_END_OF_GAME", false, true, "Stab", null));
        prayers.add(prayer(4, "IRON_MAN", "Iron Man",
            "One chosen player without Loner improves AV by 1 (Max 11+)",
            "UNTIL_END_OF_GAME", false, true, null, map("stat", "AV", "delta", 1)));
        prayers.add(prayer(5, "KNUCKLE_DUSTERS", "Knuckle Dusters",
            "One chosen player without Loner gains Mighty Blow (+1)",
            "UNTIL_END_OF_GAME", false, true, "Mighty Blow", null));
        prayers.add(prayer(6, "BAD_HABITS", "Bad Habits",
            "D3 random opponent players without Loner gain Loner (2+)",
            "UNTIL_END_OF_GAME", false, true, "Loner", null));
        prayers.add(prayer(7, "GREASY_CLEATS", "Greasy Cleats",
            "One random opponent player without Loner has MA reduced by 1",
            "UNTIL_END_OF_GAME", false, true, null, map("stat", "MA", "delta", -1)));
        prayers.add(prayer(8, "BLESSED_STATUE_OF_NUFFLE", "Blessed Statue of Nuffle",
            "One random player without Loner gains Pro",
            "UNTIL_END_OF_GAME", true, false, "Pro", null));
        prayers.add(prayer(9, "MOLES_UNDER_THE_PITCH", "Moles under the Pitch",
            "GFI and rush rolls suffer a -1 modifier",
            "UNTIL_END_OF_GAME", false, true, null, map("effect", "GFI_MODIFIER", "value", -1)));
        prayers.add(prayer(10, "PERFECT_PASSING", "Perfect Passing",
            "Successful completions gain 2 SPP instead of 1",
            "UNTIL_END_OF_GAME", false, true, null, map("effect", "COMPLETION_SPP_BONUS", "value", 1)));
        prayers.add(prayer(11, "FAN_INTERACTION", "Fan Interaction",
            "Crowd push casualties gain 2 SPP instead of 1",
            "UNTIL_END_OF_GAME", false, true, null, map("effect", "CROWD_PUSH_CAS_SPP_BONUS", "value", 1)));
        prayers.add(prayer(12, "NECESSARY_VIOLENCE", "Necessary Violence",
            "Casualties caused by opponents against your players give 2 SPP",
            "UNTIL_END_OF_GAME", false, true, null, map("effect", "OPP_CAS_SPP_BONUS", "value", 1)));
        prayers.add(prayer(13, "FOULING_FRENZY", "Fouling Frenzy",
            "Foul casualties gain 2 SPP instead of 1",
            "UNTIL_END_OF_GAME", false, true, null, map("effect", "FOUL_CAS_SPP_BONUS", "value", 1)));
        prayers.add(prayer(14, "THROW_A_ROCK", "Throw a Rock",
            "Once per game, knock down an opponent on a 4+",
            "UNTIL_END_OF_GAME", false, true, null, map("effect", "THROW_A_ROCK")));
        prayers.add(prayer(15, "UNDER_SCRUTINY", "Under Scrutiny",
            "Fouls always spotted if armor broken",
            "UNTIL_END_OF_GAME", false, true, null, map("effect", "FOULS_ALWAYS_SPOTTED")));
        prayers.add(prayer(16, "INTENSIVE_TRAINING", "Intensive Training",
            "A random player gains a primary skill",
            "UNTIL_END_OF_GAME", false, true, null, map("effect", "GAIN_PRIMARY_SKILL")));

        Map<String, Object> wrapper = new LinkedHashMap<>();
        wrapper.put("edition", "bb2020");
        wrapper.put("prayers", prayers);
        File outDir = resolveOutputDir();
        mkdirs(new File(outDir, "prayers"));
        writeJson(new File(outDir, "prayers/bb2020_prayers.json"), wrapper);
        System.out.println("Wrote bb2020 prayers: " + prayers.size());
    }

    private void writePrayersBb2025() throws Exception {
        // BB2025 uses same prayers as BB2020 with same structure
        List<Map<String, Object>> prayers = new ArrayList<>();
        prayers.add(prayer(1, "TREACHEROUS_TRAPDOOR", "Treacherous Trapdoor",
            "Trapdoors appear. On a roll of 1 a player stepping on them falls through them",
            "UNTIL_END_OF_HALF", true, false, null, null));
        prayers.add(prayer(2, "FRIENDS_WITH_THE_REF", "Friends with the Ref",
            "Argue the call succeeds on 5+", "UNTIL_END_OF_GAME", true, false, null, null));
        prayers.add(prayer(3, "STILETTO", "Stiletto",
            "One random player gains Stab", "UNTIL_END_OF_GAME", false, true, "Stab", null));
        prayers.add(prayer(4, "IRON_MAN", "Iron Man",
            "One chosen player improves AV by 1 (Max 11+)", "UNTIL_END_OF_GAME", false, true, null, map("stat", "AV", "delta", 1)));
        prayers.add(prayer(5, "KNUCKLE_DUSTERS", "Knuckle Dusters",
            "One chosen player gains Mighty Blow (+1)", "UNTIL_END_OF_GAME", false, true, "Mighty Blow", null));
        prayers.add(prayer(6, "BAD_HABITS", "Bad Habits",
            "D3 random opponent players gain Loner (2+)", "UNTIL_END_OF_GAME", false, true, "Loner", null));
        prayers.add(prayer(7, "GREASY_CLEATS", "Greasy Cleats",
            "One random opponent has MA reduced by 1", "UNTIL_END_OF_GAME", false, true, null, map("stat", "MA", "delta", -1)));
        prayers.add(prayer(8, "BLESSING_OF_NUFFLE", "Blessing of Nuffle",
            "One random player gains Pro", "UNTIL_END_OF_GAME", true, false, "Pro", null));
        prayers.add(prayer(9, "MOLES_UNDER_THE_PITCH", "Moles under the Pitch",
            "GFI rolls suffer a -1 modifier", "UNTIL_END_OF_GAME", false, true, null, map("effect", "GFI_MODIFIER", "value", -1)));
        prayers.add(prayer(10, "PERFECT_PASSING", "Perfect Passing",
            "Successful completions gain 2 SPP", "UNTIL_END_OF_GAME", false, true, null, map("effect", "COMPLETION_SPP_BONUS", "value", 1)));
        prayers.add(prayer(11, "DAZZLING_CATCHING", "Dazzling Catching",
            "Catches from passes gain 1 SPP", "UNTIL_END_OF_GAME", false, true, null, map("effect", "CATCH_SPP_BONUS", "value", 1)));
        prayers.add(prayer(12, "FAN_INTERACTION", "Fan Interaction",
            "Crowd push casualties gain 2 SPP", "UNTIL_END_OF_GAME", false, true, null, map("effect", "CROWD_PUSH_CAS_SPP_BONUS", "value", 1)));
        prayers.add(prayer(13, "FOULING_FRENZY", "Fouling Frenzy",
            "Foul casualties gain 2 SPP", "UNTIL_END_OF_GAME", false, true, null, map("effect", "FOUL_CAS_SPP_BONUS", "value", 1)));
        prayers.add(prayer(14, "THROW_A_ROCK", "Throw a Rock",
            "Once per game, knock down an opponent on a 4+", "UNTIL_END_OF_GAME", false, true, null, map("effect", "THROW_A_ROCK")));
        prayers.add(prayer(15, "UNDER_SCRUTINY", "Under Scrutiny",
            "Fouls always spotted if armor broken", "UNTIL_END_OF_GAME", false, true, null, map("effect", "FOULS_ALWAYS_SPOTTED")));
        prayers.add(prayer(16, "INTENSIVE_TRAINING", "Intensive Training",
            "A random player gains a primary skill", "UNTIL_END_OF_GAME", false, true, null, map("effect", "GAIN_PRIMARY_SKILL")));

        Map<String, Object> wrapper = new LinkedHashMap<>();
        wrapper.put("edition", "bb2025");
        wrapper.put("prayers", prayers);
        File outDir = resolveOutputDir();
        writeJson(new File(outDir, "prayers/bb2025_prayers.json"), wrapper);
        System.out.println("Wrote bb2025 prayers: " + prayers.size());
    }

    // -------------------------------------------------------------------------
    // Serious Injuries
    // -------------------------------------------------------------------------

    private void writeSeriousInjuries() throws Exception {
        // Serious injury table (2D6 roll: 2-7 = Badly Hurt, etc.)
        // BB2020/BB2025 injury table
        List<Map<String, Object>> injuries = new ArrayList<>();
        injuries.add(injuryEntry(2, 7, "BADLY_HURT", "Badly Hurt", false));
        injuries.add(injuryEntry(8, 9, "BADLY_HURT", "Badly Hurt", false));
        injuries.add(injuryEntry(10, 10, "NIGGLE", "Niggling Injury", true));
        injuries.add(injuryEntry(11, 11, "STAT_DECREASE", "Serious Injury (stat decrease)", true));
        injuries.add(injuryEntry(12, 12, "DEAD", "DEAD!", false));

        // Stat decrease sub-table (D6)
        List<Map<String, Object>> statDecrease = new ArrayList<>();
        statDecrease.add(statDecreaseEntry(1, 2, "MOVEMENT_DECREASE", "MA", -1));
        statDecrease.add(statDecreaseEntry(3, 4, "AGILITY_DECREASE", "AG", -1));
        statDecrease.add(statDecreaseEntry(5, 5, "PASSING_DECREASE", "PA", -1));
        statDecrease.add(statDecreaseEntry(6, 6, "STRENGTH_DECREASE", "ST", -1));

        Map<String, Object> wrapper = new LinkedHashMap<>();
        wrapper.put("bb2020", injuries);
        wrapper.put("bb2025", injuries); // same table
        wrapper.put("stat_decrease_table", statDecrease);

        // BB2016 also uses the same injury table structure
        wrapper.put("bb2016", injuries);

        File outDir = resolveOutputDir();
        mkdirs(new File(outDir, "injuries"));
        writeJson(new File(outDir, "injuries/serious_injuries.json"), wrapper);
        System.out.println("Wrote serious injury tables");
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    private Map<String, Object> inducement(String id, String namePlural, String nameSingular,
            int cost, int maxCount, String usage, String availability) {
        Map<String, Object> m = new LinkedHashMap<>();
        m.put("id", id);
        m.put("name", namePlural);
        m.put("name_singular", nameSingular);
        m.put("cost", cost);
        m.put("max_count", maxCount);
        m.put("usage", usage);
        if (availability != null) m.put("availability", availability);
        return m;
    }

    private Map<String, Object> prayer(int roll, String id, String name, String description,
            String duration, boolean exhibitionLegal, boolean leagueOnly,
            String grantsSkill, Map<String, Object> effect) {
        Map<String, Object> m = new LinkedHashMap<>();
        m.put("roll", roll);
        m.put("id", id);
        m.put("name", name);
        m.put("description", description);
        m.put("duration", duration);
        m.put("exhibition_legal", exhibitionLegal);
        m.put("league_only", leagueOnly);
        if (grantsSkill != null) m.put("grants_skill", grantsSkill);
        if (effect != null) m.put("effect", effect);
        return m;
    }

    private Map<String, Object> injuryEntry(int rollMin, int rollMax, String type, String name, boolean serious) {
        Map<String, Object> m = new LinkedHashMap<>();
        m.put("roll_min", rollMin);
        m.put("roll_max", rollMax);
        m.put("type", type);
        m.put("name", name);
        m.put("is_serious", serious);
        return m;
    }

    private Map<String, Object> statDecreaseEntry(int rollMin, int rollMax, String type, String stat, int delta) {
        Map<String, Object> m = new LinkedHashMap<>();
        m.put("roll_min", rollMin);
        m.put("roll_max", rollMax);
        m.put("type", type);
        m.put("stat", stat);
        m.put("delta", delta);
        return m;
    }

    private void writeEditionInducements(String edition, List<Map<String, Object>> inducements) throws Exception {
        Map<String, Object> wrapper = new LinkedHashMap<>();
        wrapper.put("edition", edition);
        wrapper.put("inducements", inducements);
        File outDir = resolveOutputDir();
        mkdirs(new File(outDir, "inducements"));
        writeJson(new File(outDir, "inducements/" + edition + "_inducements.json"), wrapper);
        System.out.println("Wrote " + edition + " inducements: " + inducements.size());
    }

    private Map<String, Object> map(Object... keyValues) {
        Map<String, Object> m = new LinkedHashMap<>();
        for (int i = 0; i + 1 < keyValues.length; i += 2) {
            m.put(keyValues[i].toString(), keyValues[i + 1]);
        }
        return m;
    }

    private File resolveOutputDir() {
        // Try relative path first, then absolute
        File f = new File(OUTPUT_DIR);
        if (!f.exists()) {
            f = new File("C:/Users/Admin/niels/ffb-rust/data");
        }
        return f;
    }

    private void mkdirs(File dir) {
        if (!dir.exists()) dir.mkdirs();
    }

    private void writeJson(File file, Object obj) throws Exception {
        String json = toJson(obj, 0);
        try (PrintWriter w = new PrintWriter(new FileWriter(file))) {
            w.print(json);
        }
    }

    private String toJson(Object obj, int indent) {
        if (obj == null) return "null";
        String pad = "  ".repeat(indent);
        String padInner = "  ".repeat(indent + 1);
        if (obj instanceof String) {
            return "\"" + ((String) obj).replace("\\", "\\\\").replace("\"", "\\\"")
                .replace("\n", "\\n").replace("\r", "\\r") + "\"";
        }
        if (obj instanceof Integer || obj instanceof Long || obj instanceof Boolean) {
            return obj.toString();
        }
        if (obj instanceof List) {
            List<?> list = (List<?>) obj;
            if (list.isEmpty()) return "[]";
            StringBuilder sb = new StringBuilder("[\n");
            for (int i = 0; i < list.size(); i++) {
                sb.append(padInner).append(toJson(list.get(i), indent + 1));
                if (i < list.size() - 1) sb.append(",");
                sb.append("\n");
            }
            return sb.append(pad).append("]").toString();
        }
        if (obj instanceof Map) {
            Map<?, ?> map = (Map<?, ?>) obj;
            if (map.isEmpty()) return "{}";
            StringBuilder sb = new StringBuilder("{\n");
            List<? extends Map.Entry<?, ?>> entries = new ArrayList<>(map.entrySet());
            for (int i = 0; i < entries.size(); i++) {
                Map.Entry<?, ?> e = entries.get(i);
                sb.append(padInner).append("\"").append(e.getKey()).append("\": ")
                  .append(toJson(e.getValue(), indent + 1));
                if (i < entries.size() - 1) sb.append(",");
                sb.append("\n");
            }
            return sb.append(pad).append("}").toString();
        }
        return "\"" + obj.toString() + "\"";
    }

    // XML helpers
    private String getText(Document doc, String tag) {
        NodeList nodes = doc.getElementsByTagName(tag);
        if (nodes.getLength() == 0) return null;
        return nodes.item(0).getTextContent().trim();
    }

    private int getIntText(Document doc, String tag, int defaultVal) {
        String s = getText(doc, tag);
        if (s == null || s.isEmpty()) return defaultVal;
        try { return Integer.parseInt(s); } catch (NumberFormatException e) { return defaultVal; }
    }

    private String getChildText(Element el, String tag) {
        NodeList nodes = el.getElementsByTagName(tag);
        if (nodes.getLength() == 0) return null;
        // Only direct children
        for (int i = 0; i < nodes.getLength(); i++) {
            if (nodes.item(i).getParentNode() == el) {
                return nodes.item(i).getTextContent().trim();
            }
        }
        return nodes.item(0).getTextContent().trim();
    }

    private int getChildInt(Element el, String tag, int defaultVal) {
        String s = getChildText(el, tag);
        if (s == null || s.isEmpty() || s.equals("-")) return defaultVal;
        try { return Integer.parseInt(s); } catch (NumberFormatException e) { return defaultVal; }
    }
}
