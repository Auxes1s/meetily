from pathlib import Path
import subprocess,time,json,re,sqlite3,hashlib
base=Path('/tmp/meetily-l05-checks')
profile=Path.home()/'Library/Application Support/com.meetily.export.test'
report=profile/'meeting-export-smoke.json'
started=time.time();handled=set();deadline=started+240

def script(body):
    pid=int(subprocess.check_output(['pgrep','-f','^./target/debug/meetily$'],text=True).strip())
    prefix=f'tell application "System Events" to tell (first process whose unix id is {pid})\n set frontmost to true\n'
    return subprocess.check_output(['osascript'],input=prefix+body+'\nend tell\n',text=True).strip()

def screenshot(name):
    bounds=script('get {position of window 1, size of window 1}')
    numbers=re.findall(r'-?\d+',bounds)
    if len(numbers)==4:
        subprocess.run(['screencapture','-x','-R'+','.join(numbers),str(base/name)],check=True)

while time.time()<deadline:
    if report.exists() and report.stat().st_mtime>=started:
        d=json.loads(report.read_text());phase=d.get('phase')
        if phase=='failed': raise RuntimeError(d)
        if phase not in handled:
            print(phase,flush=True);handled.add(phase)
            if phase=='awaiting-cancel':
                time.sleep(0.65);screenshot('native-save-dialog.png');script('key code 53')
            elif phase in ('awaiting-json-save','awaiting-markdown-save'):
                if phase=='awaiting-json-save':
                    assert not (base/'Native export smoke — 日本語.json').exists(), 'Cancellation created an output file'
                time.sleep(0.65)
                script('keystroke "g" using {command down, shift down}\n delay 0.35\n keystroke "/tmp/meetily-l05-checks"\n delay 0.15\n key code 36\n delay 0.5\n key code 36')
            elif phase=='finished':
                screenshot('native-library-export.png')
                record_path=base/'Native export smoke — 日本語.json';md_path=base/'Native export smoke — 日本語.md'
                record=json.loads(record_path.read_text());markdown=md_path.read_text()
                assert record['format']=='meetily.meeting-record' and record['schema_version']==1
                assert len(record['transcripts'])==1207
                assert record['transcripts'][-1]['text']=='Offline source 1206 — Taglish 日本語'
                assert [p['audio_source'] for p in record['transcripts'][:4]]==['microphone','system','mixed','unknown']
                assert record['meeting']['participants']==['José'] and record['meeting']['tags']==['予算']
                assert 'Latest native export notes — café 日本語' in record['original_notes']['notes_markdown']
                assert '## Full transcript' in markdown and 'Offline source 1206 — Taglish 日本語' in markdown
                assert all(label in markdown for label in ['Microphone','System audio','Mixed audio','Unknown'])
                db=sqlite3.connect(f'file:{profile / "meeting_minutes.sqlite"}?mode=ro',uri=True)
                stored=db.execute('SELECT notes_json,revision FROM meeting_notes WHERE meeting_id=?',(d['meeting_id'],)).fetchone()
                assert record['original_notes']['notes_json']==stored[0] and record['original_notes']['revision']==stored[1]
                assert db.execute('SELECT COUNT(*) FROM transcripts WHERE meeting_id=?',(d['meeting_id'],)).fetchone()[0]==1207
                db.close()
                d.update({'parsed_exports_verified':True,'original_json_matches_sqlite':True,'hashes':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [record_path,md_path]},'sizes':{p.name:p.stat().st_size for p in [record_path,md_path]}})
                (base/'native-webview-result.json').write_text(json.dumps(d,indent=2,ensure_ascii=False))
                print(json.dumps(d,ensure_ascii=False),flush=True);break
    time.sleep(0.1)
else: raise RuntimeError('Native export smoke timed out')
