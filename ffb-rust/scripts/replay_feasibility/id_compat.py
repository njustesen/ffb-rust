import json, re, sys
J = 'C:/Users/Admin/niels/ffb-rust/ffb-java/ffb-common/src/main/java/com/fumbbl/ffb/'
facts = json.load(open(sys.argv[1]))
def ids(path):
    return set(re.findall(r'\b[A-Z0-9_]+\("([A-Za-z0-9]+)"', open(J + path, encoding='utf-8').read()))
rep = ids('report/ReportId.java')
mc = ids('model/change/ModelChangeId.java')
opt = ids('option/GameOptionId.java')
print('report ids known', len(rep), 'model change ids known', len(mc), 'option ids known', len(opt))
print('UNKNOWN report ids:', sorted(set(facts['reportCounts']) - rep))
print('UNKNOWN model change ids:', sorted(set(facts['modelChangeCounts']) - mc))
used_opts = {k.split(' ', 1)[1] for k in facts['gameOptions']}
print('UNKNOWN option ids:', sorted(used_opts - opt))
# field-level: compare report sample fields with the Java report class json keys is done separately
