#!/usr/bin/env python
"""Classify every skill fielded by the 111 drafted squads by what the 33,300-game
event stream can actually prove about it."""
import json
import glob
from collections import Counter

gates = [json.load(open(f)) for f in glob.glob('out/*.json')]
T = Counter()
A = Counter()
SU = Counter()
SD = Counter()
for d in gates:
    T.update(d['types'])
    A.update(d['actions'])
    for k, v in d['skill_used'].items():
        SU[int(k)] += v
    for k, v in d['skill_declined'].items():
        SD[int(k)] += v

inv = json.load(open('inventory.json'))
names = inv['skill_enum']
fielded = json.load(open('fielded.json'))['fielded']       # SkillId -> [cells]

SUID = {names[i]: v for i, v in SU.items()}
SDID = {names[i]: v for i, v in SD.items()}

# skill -> (evidence label, count).  event: a GameEvent type; action: a declared PlayerAction;
# skilluse: the SkillUse event. None => the engine emits nothing that names this skill.
EV = {
    'Dodge': ('skillUse', SUID.get('Dodge', 0) + SDID.get('Dodge', 0)),
    'Wrestle': ('skillUse', SUID.get('Wrestle', 0) + SDID.get('Wrestle', 0)),
    'Horns': ('skillUse', SUID.get('Horns', 0)),
    'Tackle': ('skillUse', SUID.get('Tackle', 0)),
    'Juggernaut': ('skillUse', SUID.get('Juggernaut', 0) + SDID.get('Juggernaut', 0)),
    'DumpOff': ('skillUse', SUID.get('DumpOff', 0) + SDID.get('DumpOff', 0)),
    'Loner': ('event lonerRoll', T['lonerRoll']),
    'Pro': ('event proRoll', T['proRoll']),
    'OldPro': ('event proRoll', T['proRoll']),
    'Regeneration': ('event regenerationRoll', T['regenerationRoll']),
    'RightStuff': ('event rightStuffRoll', T['rightStuffRoll']),
    'ThrowTeamMate': ('event throwTeamMateRoll', T['throwTeamMateRoll']),
    'AlwaysHungry': ('event alwaysHungry', T['alwaysHungry']),
    'BoneHead': ('event confusionRoll', T['confusionRoll']),
    'ReallyStupid': ('event confusionRoll', T['confusionRoll']),
    'TakeRoot': ('event confusionRoll', T['confusionRoll']),
    'AnimalSavagery': ('event animalSavagery', T['animalSavagery']),
    'UnchannelledFury': ('event animalSavagery', T['animalSavagery']),
    'WildAnimal': ('event animalSavagery', T['animalSavagery']),
    'BloodLust': ('event bloodLustRoll', T['bloodLustRoll']),
    'FoulAppearance': ('event foulAppearanceRoll', T['foulAppearanceRoll']),
    'Dauntless': ('event dauntlessRoll', T['dauntlessRoll']),
    'Animosity': ('event animosityRoll', T['animosityRoll']),
    'Leap': ('event jumpRoll', T['jumpRoll']),
    'VeryLongLegs': ('event jumpRoll', T['jumpRoll']),
    'Pogo': ('event jumpRoll', T['jumpRoll']),
    'PogoStick': ('event jumpRoll', T['jumpRoll']),
    'JumpUp': ('event jumpUpRoll', T['jumpUpRoll']),
    'HypnoticGaze': ('event hypnoticGazeRoll', T['hypnoticGazeRoll']),
    'BalefulHex': ('event balefulHexRoll', T['balefulHexRoll']),
    'LookIntoMyEyes': ('event lookIntoMyEyesRoll', T['lookIntoMyEyesRoll']),
    'WeepingDagger': ('event weepingDaggerRoll', T['weepingDaggerRoll']),
    'BreatheFire': ('event breatheFireRoll', T['breatheFireRoll']),
    'ProjectileVomit': ('event projectileVomitRoll', T['projectileVomitRoll']),
    'Chainsaw': ('event chainsawRoll', T['chainsawRoll']),
    'BallAndChain': ('event escapeRoll', T['escapeRoll']),
    'SafeThrow': ('event safeThrowRoll', T['safeThrowRoll']),
    'SafePass': ('event safeThrowRoll', T['safeThrowRoll']),
    'Bombardier': ('action ThrowBomb', A['ThrowBomb']),
    'HitAndRun': ('event hitAndRun', T['hitAndRun']),
    'KickTeamMate': ('action KickTeamMate', A['KickTeamMate']),
    'AllYouCanEat': ('event allYouCanEatRoll', T['allYouCanEatRoll']),
    'Swoop': ('event swoopPlayer', T['swoopPlayer']),
    'PickMeUp': ('event pickMeUpRoll', T['pickMeUpRoll']),
    'Swarming': ('event swarmingPlayersRoll', T['swarmingPlayersRoll']),
    'PilingOn': ('event pilingOn', T['pilingOn']),
    'PileDriver': ('event pilingOn', T['pilingOn']),
    'Fumblerooskie': ('event fumblerooskie', T['fumblerooskie']),
    'Leader': ('event leader', T['leader']),
    'TeamCaptain': ('event teamCaptainRoll', T['teamCaptainRoll']),
    'PumpUpTheCrowd': ('event pumpUpTheCrowdReRoll', T['pumpUpTheCrowdReRoll']),
    'PassBlock': ('event passBlock', T['passBlock']),
    'ThenIStartedBlastin': ('event thenIStartedBlastin', T['thenIStartedBlastin']),
    'BlastinSolvesEverything': ('event thenIStartedBlastin', T['thenIStartedBlastin']),
    'BeerBarrelBash': ('event kegThrow', T['kegThrow']),
    'WisdomOfTheWhiteDwarf': ('action WisdomOfTheWhiteDwarf', A['WisdomOfTheWhiteDwarf']),
    'Treacherous': ('action Treacherous', A['Treacherous']),
    'RaidingParty': ('action RaidingParty', A['RaidingParty']),
    'BlackInk': ('action BlackInk', A['BlackInk']),
    'CatchOfTheDay': ('action CatchOfTheDay', A['CatchOfTheDay']),
    'FuriousOutburst': ('action FuriousOutburst', A['FuriousOutburst']),
    'Punt': ('action Punt', A['Punt']),
    'MultipleBlock': ('action MultipleBlock', A['MultipleBlock']),
    'HailMaryPass': ('action HailMaryPass', A['HailMaryPass']),
    'ExcuseMeAreYouAZoat': ('action AutoGazeZoat', A['AutoGazeZoat']),
    'Kick': ('event kickoffScatter', 0),   # no skill-specific evidence
}

rows = []
for sk, cells in fielded.items():
    lab, n = EV.get(sk, (None, 0))
    rows.append((sk, len(cells), lab, n))

silent = [r for r in rows if r[2] is None]
dead = [r for r in rows if r[2] is not None and r[3] == 0]
live = [r for r in rows if r[2] is not None and r[3] > 0]

print('fielded skills: %d   with telemetry that FIRED: %d   with telemetry that NEVER fired: %d   with NO telemetry at all: %d'
      % (len(rows), len(live), len(dead), len(silent)))

print('\n== A. FIELDED, has a dedicated event, NEVER fired in 33,300 games ==')
for sk, c, lab, n in sorted(dead, key=lambda r: -r[1]):
    print('  %-22s fielded in %3d cells   %s = 0' % (sk, c, lab))

print('\n== B. FIELDED, engine emits NOTHING that names it (invisible to the event stream) ==')
for sk, c, lab, n in sorted(silent, key=lambda r: -r[1]):
    print('  %-22s fielded in %3d cells' % (sk, c))

print('\n== C. FIELDED and provably exercised ==')
for sk, c, lab, n in sorted(live, key=lambda r: -r[3]):
    print('  %-22s fielded in %3d cells   %-32s %d' % (sk, c, lab, n))

json.dump({'live': live, 'dead': dead, 'silent': silent}, open('skill_class.json', 'w'), indent=1)
