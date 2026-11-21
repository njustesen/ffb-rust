"""For a few engine files, find which upstream (christerk/ffb) revision our copy matches."""
import base64, json, subprocess, sys
ROOTS = {'harness-tree': 'C:/Users/Admin/niels/ffb-rust/ffb-java/ffb/', 'root-tree': 'C:/Users/Admin/niels/ffb-rust/ffb-java/'}
FILES = [
    'ffb-common/src/main/java/com/fumbbl/ffb/model/change/ModelChangeId.java',
    'ffb-common/src/main/java/com/fumbbl/ffb/option/GameOptionId.java',
    'ffb-common/src/main/java/com/fumbbl/ffb/report/ReportId.java',
    'ffb-server/src/main/java/com/fumbbl/ffb/server/step/bb2025/shared/StepInitSelecting.java',
    'ffb-server/src/main/java/com/fumbbl/ffb/server/step/bb2025/move/StepMoveDodge.java',
    'ffb-common/src/main/java/com/fumbbl/ffb/model/ActingPlayer.java',
]
def gh(path):
    return json.loads(subprocess.run(['gh', 'api', path], capture_output=True, text=True, check=True).stdout)
def norm(s):
    return s.replace('\r\n', '\n').strip()
for f in FILES:
    commits = gh('repos/christerk/ffb/commits?path=%s&per_page=60' % f)
    for name, root in ROOTS.items():
        try:
            ours = norm(open(root + f, encoding='utf-8').read())
        except OSError:
            print(f.split('/')[-1], name, 'MISSING locally'); continue
        hit = None
        for c in commits:
            sha = c['sha']
            try:
                blob = gh('repos/christerk/ffb/contents/%s?ref=%s' % (f, sha))
            except subprocess.CalledProcessError:
                continue
            theirs = norm(base64.b64decode(blob['content']).decode('utf-8'))
            if theirs == ours:
                hit = (c['commit']['committer']['date'], c['commit']['message'].split('\n')[0][:60]); break
        newest = commits[0]['commit']['committer']['date'] if commits else '?'
        print('%-28s %-12s matches=%s  upstream-newest=%s  upstream-revs=%d' % (f.split('/')[-1], name, hit, newest, len(commits)))
