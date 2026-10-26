#!/usr/bin/env python3
"""Render E08 source/output rows and compute metrics from explicit Codex review.

No lexical matching is used to claim semantic support or recall. Human acceptance
is always pending; the generated sheet is for the required independent review.
"""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path


def parse_response(raw):
    if not isinstance(raw, str):
        return None
    text = raw.strip()
    if text.startswith('```'):
        text = text.split('\n', 1)[-1].rsplit('```', 1)[0].strip()
    try:
        parsed = json.loads(text)
        return parsed if isinstance(parsed,dict) and isinstance(parsed.get('claims'),list) else None
    except (ValueError, TypeError):
        return None


def claims(raw):
    return (parse_response(raw) or {}).get('claims',[])


def review_claims(raw):
    """Recover readable claims for review only, never for production acceptance."""
    if parse_response(raw) is not None:
        return claims(raw)
    if not isinstance(raw,str):
        return []
    text=raw.strip()
    if text.startswith('```'):
        text=text.split('\n',1)[-1].rsplit('```',1)[0].strip()
    try:
        value,_=json.JSONDecoder().raw_decode(text)
        items=value if isinstance(value,list) else value.get('claims',[])
        return [c for c in items if isinstance(c,dict) and 'text' in c]
    except (ValueError,TypeError,AttributeError):
        return []


def claim_rows(run):
    rows = []
    for a in run['attempts']:
        partition = a.get('partition', '')
        prefix = f"c{a['chunk']}" + (f"-p{partition}" if any(c in partition for c in 'LR') else '')
        for i, c in enumerate(review_claims(a['raw_response'])):
            if 'citations' in c:
                c = dict(c)
                citations = []
                for ref in c['citations']:
                    index = int(ref[1:]) if isinstance(ref, str) and ref.startswith('p') and ref[1:].isdigit() else -1
                    source = a['sources'][index] if 0 <= index < len(a['sources']) else None
                    citations.append({'source_id': source['source_id'] if source else 'unresolved:'+str(ref),
                                      'excerpt': source['text'] if source else ''})
                c['sources'] = citations
            rows.append({'row_id': f"{prefix}-a{a['attempt']}-i{i}", 'claim': c,
                         'stage': 'raw' if a['attempt'] == 0 else 'repair',
                         'structural_error': a['validation_error'], 'chunk_sources': a['sources']})
        if a['raw_response'] and parse_response(a['raw_response']) is None:
            rows.append({'row_id': f"{prefix}-a{a['attempt']}-unparsed", 'claim': {'text': a['raw_response']},
                         'stage': 'unparsed response', 'structural_error': a['validation_error'], 'chunk_sources': a['sources']})
    for i, c in enumerate((run.get('validated_candidate') or {}).get('claims', [])):
        rows.append({'row_id': f'accepted-i{i}', 'claim': c, 'stage': 'validated candidate',
                     'structural_error': None, 'chunk_sources': run['source_snapshot']['sources']})
    return rows


def metrics(run, review):
    rows = claim_rows(run)
    assert set(review['claims']) == {r['row_id'] for r in rows}, 'Every raw/repaired/accepted claim requires review'
    expected = {e['id'] for e in run['fixture']['annotations']['relevant_decisions_actions']}
    priorities = {p['id'] for p in run['fixture']['annotations']['user_priorities']}
    recalled = review['recalled_expectations']; retained = review['retained_priorities']
    accepted = {r['row_id'] for r in rows if r['stage'] == 'validated candidate'}
    assert set(recalled) <= expected and set(retained) <= priorities
    for mapping in (recalled, retained):
        for references in mapping.values():
            assert references and set(references) <= accepted, 'Recall must point to delivered candidate claims'
    violations = defaultdict(int); raw_violations = defaultdict(int)
    for row in rows:
        verdict = review['claims'][row['row_id']]
        assert verdict['support'] in ('supported','qualified','unsupported','unparseable')
        assert verdict.get('notes'), 'Review must give a source-based reason'
        if verdict['support']=='unsupported':
            (violations if row['stage']=='validated candidate' else raw_violations)['unsupported_claim'] += 1
        invalid_count=0
        for citation in row['claim'].get('sources',[]):
            valid=any(s['source_id']==citation.get('source_id') and citation.get('excerpt')
                      and citation['excerpt'] in s['text'] for s in row['chunk_sources'])
            if not valid:
                invalid_count+=1
                (violations if row['stage']=='validated candidate' else raw_violations)['invalid_reference'] += 1
        if verdict.get('invalid_reference') and not invalid_count:
            (violations if row['stage']=='validated candidate' else raw_violations)['invalid_reference'] += 1
        for flag in ('fabricated_confirmed_decision','fabricated_owner_office','fabricated_deadline',
                     'citation_error','lost_uncertainty'):
            if verdict.get(flag, False):
                (violations if row['stage']=='validated candidate' else raw_violations)[flag] += 1
    recalled_count = len(recalled) if accepted else 0
    retained_count = len(retained) if accepted else 0
    raw_fabrication=any(raw_violations.get(flag,0) for flag in
        ('fabricated_confirmed_decision','fabricated_owner_office','fabricated_deadline'))
    run_pass=(recalled_count/len(expected)>=.9 and retained_count/len(priorities)>=.9
              and not any(violations.values()) and not raw_fabrication)
    return {'run_id':run['run_id'],'fixture_id':run['fixture']['id'],'phase':run['phase'],'outcome':run['outcome'],'elapsed_ms':run['elapsed_ms'],
            'expected':len(expected),'recalled':recalled_count,'priorities':len(priorities),'retained':retained_count,
            'recall_pct':100*recalled_count/len(expected),'priority_pct':100*retained_count/len(priorities),
            'accepted_violations':dict(violations),'raw_and_repair_violations':dict(raw_violations),
            'codex_review_outcome':'run checks met; human acceptance pending' if run_pass else 'fails run quality checks',
            'reviewed_rows':len(rows),'error':run['error'],'human_acceptance':'pending'}


def escape(text):
    return str(text or '').replace('|','\\|').replace('\n','<br>')


def render(runs, reviews, output):
    sheet=['# Enhancement source/output review sheet', '',
           'Semantic assessments below are **Codex review**. Human acceptance is pending for every run. '
           'Read complete source labels and raw response in the linked JSON when a row is ambiguous. '
           'Structural acceptance creates an unapproved candidate, never human acceptance.', '']
    results=[]; groups=defaultdict(list)
    for path, run in runs:
        key=f"{run['phase']}/{run['run_id']}"; review=reviews.get(key)
        annotations=run['fixture']['annotations']
        sheet.extend([f"## {key}", '', f"Outcome: {run['outcome']}; error: {run['error'] or 'none'}. "
                      f"[Full evidence]({path.name}).", '',
                      'Expected decisions/actions: '+escape('; '.join(e['id']+': '+e['text']+' ['+e['agreement_status']+']' for e in annotations['relevant_decisions_actions'])), '',
                      'User priorities: '+escape('; '.join(p['id']+': '+p['topic'] for p in annotations['user_priorities'])), '',
                      'Forbidden inferences: '+escape('; '.join(annotations['forbidden_inferences'])), '',
                      '| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |',
                      '| --- | --- | --- | --- | --- |'])
        for row in claim_rows(run):
            # The human packet shows each delivered claim once plus rejected
            # attempts. Complete raw/repair adjudications remain in review JSON.
            if (row['stage']!='validated candidate' and row['structural_error'] is None
                    and run.get('validated_candidate') is not None):
                continue
            c=row['claim']; cited=[]
            for citation in c.get('sources',[]):
                source=next((s for s in row['chunk_sources'] if s['source_id']==citation.get('source_id')),None)
                actual=source['text'] if source else 'NONEXISTENT / OUTSIDE THIS CHUNK'
                if len(actual)>500:
                    start=max(0,actual.find(citation.get('excerpt',''))-80)
                    actual=actual[start:start+500]+' [full passage in JSON]'
                cited.append(f"{citation.get('source_id')}: quoted `{citation.get('excerpt')}`; source: {actual}")
            v=(review or {}).get('claims',{}).get(row['row_id'])
            verdict=f"{v['support']}: {v['notes']}" if v else 'Pending'
            text=f"{c.get('kind')} / {c.get('status')}: {c.get('text')} (owner={c.get('owner')}; deadline={c.get('deadline')})"
            sheet.append('| '+' | '.join(map(escape,[row['row_id']+' / '+row['stage'],text,'; '.join(cited),verdict,'Pending']))+' |')
        if review:
            m=metrics(run,review); results.append(m)
            groups[(run['phase'],run['provider_config']['provider'],run['provider_config']['model'])].append(m)
            sheet.extend(['',f"Recall {m['recalled']}/{m['expected']}; priorities {m['retained']}/{m['priorities']}. {review.get('notes','')}"])
        else: sheet.extend(['','Codex review pending.'])
        sheet.append('')
    aggregate=[]
    for (phase, provider,model), runs in groups.items():
        expected=sum(r['expected'] for r in runs); priorities=sum(r['priorities'] for r in runs)
        recall=sum(r['recalled'] for r in runs); retained=sum(r['retained'] for r in runs)
        flags=defaultdict(int); raw_flags=defaultdict(int)
        for r in runs:
            for flag,count in r['accepted_violations'].items():flags[flag]+=count
            for flag,count in r['raw_and_repair_violations'].items():raw_flags[flag]+=count
        coverage=Counter(r['fixture_id'] for r in runs)
        complete=set(coverage)=={f'fx-{i:02d}' for i in range(1,13)} and all(n>=2 for n in coverage.values())
        raw_fabrication=any(raw_flags.get(flag,0) for flag in
            ('fabricated_confirmed_decision','fabricated_owner_office','fabricated_deadline'))
        passes=recall/expected>=.9 and retained/priorities>=.9 and not any(flags.values()) and not raw_fabrication and complete
        aggregate.append({'phase':phase,'provider':provider,'model':model,'runs':len(runs),
            'structurally_accepted':sum(r['outcome']=='structurally_accepted_candidate' for r in runs),
            'recalled':recall,'expected':expected,'recall_pct':100*recall/expected,
            'retained':retained,'priorities':priorities,'priority_pct':100*retained/priorities,
            'accepted_violations':dict(flags),'raw_and_repair_violations':dict(raw_flags),
            'fixture_run_counts':dict(coverage),'coverage_complete':complete,
            'codex_ac07_gate':passes,'human_acceptance':'pending',
            'suitability':('evaluation incomplete; no suitability conclusion' if not complete else
                'provisional Codex gate pass; human acceptance pending' if passes else 'unsuitable under tested configuration')})
    output.mkdir(parents=True,exist_ok=True)
    (output/'source-output-review.md').write_text('\n'.join(sheet)+'\n')
    (output/'metrics.json').write_text(json.dumps({'runs':results,'models':aggregate},ensure_ascii=False,indent=2)+'\n')
    return aggregate


def main():
    ap=argparse.ArgumentParser(description=__doc__); ap.add_argument('evidence',type=Path)
    ap.add_argument('--reviews',type=Path)
    ap.add_argument('--require-codex-gates',action='store_true',help='Exit 1 unless every complete model matrix passes Codex AC07 checks; human acceptance remains pending')
    args=ap.parse_args()
    runs=[(p,json.loads(p.read_text())) for p in sorted(args.evidence.glob('*-fx-*.json'))]
    reviews=json.loads(args.reviews.read_text()) if args.reviews else {}
    models=render(runs,reviews,args.evidence)
    print(json.dumps(models,indent=2))
    if args.require_codex_gates and (not models or not all(m['codex_ac07_gate'] for m in models)):
        raise SystemExit(1)

if __name__=='__main__':main()
