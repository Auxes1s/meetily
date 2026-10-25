#!/usr/bin/env python3
"""Opt-in E08 harness. Standard library only; synthetic fixtures and loopback only."""
import argparse
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import platform
import sqlite3
import subprocess
import tempfile
import urllib.request
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]

def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(8 * 1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()

def command(*args):
    return subprocess.check_output(args, text=True).strip()

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        raise ValueError('Local evaluation refuses HTTP redirects')

HTTP = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())

def local_endpoint(value):
    u = urlsplit(value)
    if u.scheme not in ('http', 'https') or u.username or u.password or u.query or u.fragment:
        raise ValueError('Evaluation requires a plain HTTP(S) loopback endpoint')
    if u.hostname != 'localhost' and not ipaddress.ip_address(u.hostname).is_loopback:
        raise ValueError('Evaluation refuses non-loopback endpoints')
    # Pin localhost to IPv4 instead of relying on mutable DNS resolution.
    if u.hostname == 'localhost':
        return f'{u.scheme}://127.0.0.1:{u.port or (443 if u.scheme == "https" else 80)}{u.path}'.rstrip('/')
    return value.rstrip('/')

def fetch(endpoint, route, payload=None):
    body = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(endpoint + route, data=body, headers={'Content-Type': 'application/json'})
    with HTTP.open(req, timeout=10) as response:
        return json.load(response)

def protected_inventory(data, helper):
    paths = [data/'meeting_minutes.sqlite', data/'meeting_minutes.sqlite-wal', helper]
    paths += list((data/'models/summary').glob('*.gguf'))
    return {str(p): {'sha256': digest(p), 'bytes': p.stat().st_size} for p in paths if p.is_file()}

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--output', type=Path, required=True, help='New evidence directory; never overwritten')
    ap.add_argument('--phase', default='evaluation')
    ap.add_argument('--provider', choices=['all', 'builtin-ai', 'ollama'], default='all')
    ap.add_argument('--fixture', help='Optional affected-fixture rerun, e.g. fx-11')
    ap.add_argument('--scenario-matrix', choices=['standard', 'scale'], default='standard')
    ap.add_argument('--repeats', type=int, default=2)
    ap.add_argument('--app-data', type=Path, default=Path.home()/'Library/Application Support/com.meetily.ai')
    ap.add_argument('--helper', type=Path, default=Path.home()/'Applications/Meetily Notepad.app/Contents/MacOS/llama-helper')
    ap.add_argument('--ollama-endpoint', help='Defaults to read-only configured endpoint, then localhost:11434')
    ap.add_argument('--inventory-only', action='store_true')
    args = ap.parse_args()
    if args.repeats < 2:
        ap.error('E08 requires at least two repetitions')
    output = args.output.resolve(); output.mkdir(parents=True, exist_ok=False)
    data = args.app_data.resolve(); helper = args.helper.resolve()
    configured = None
    db = data/'meeting_minutes.sqlite'
    if db.exists():
        with sqlite3.connect(db.as_uri()+'?mode=ro', uri=True) as con:
            row = con.execute('SELECT provider,model,ollamaEndpoint FROM settings LIMIT 1').fetchone()
            if row: configured = dict(zip(['provider','model','ollama_endpoint'], row))
    endpoint_error = None
    try:
        endpoint = local_endpoint(args.ollama_endpoint or (configured or {}).get('ollama_endpoint') or 'http://localhost:11434')
    except ValueError as error:
        endpoint = None
        endpoint_error = str(error)
    before = protected_inventory(data, helper)
    models = []; unavailable = []
    if endpoint_error and args.provider in ('all','ollama'):
        unavailable.append({'provider':'ollama','outcome':'not_run','reason':endpoint_error})
    catalog = [('qwen3.5:2b','Qwen3.5-2B-Q4_K_M.gguf'),('qwen3.5:4b','Qwen3.5-4B-Q4_K_M.gguf'),
               ('gemma3:4b','gemma-3-4b-it-Q4_K_M.gguf'),('gemma3:1b','gemma-3-1b-it-Q8_0.gguf')]
    if args.provider in ('all','builtin-ai'):
        for name, filename in catalog:
            path=data/'models/summary'/filename
            if path.is_file() and helper.is_file():
                models.append({'provider':'builtin-ai','name':name,'file':str(path),'file_sha256':before[str(path)]['sha256'],
                               'bytes':path.stat().st_size,'helper':str(helper),'helper_sha256':before[str(helper)]['sha256']})
            else: unavailable.append({'provider':'builtin-ai','name':name,'outcome':'not_run','reason':'model file or helper unavailable'})
    ollama = {}
    if args.provider in ('all','ollama') and endpoint:
        try:
            ollama['version']=fetch(endpoint,'/api/version'); ollama['tags']=fetch(endpoint,'/api/tags')
            for tag in ollama['tags'].get('models',[]):
                name=tag['name']; show=fetch(endpoint,'/api/show',{'name':name,'verbose':True})
                show.pop('tensors',None)  # Model metadata only; tensor names do not establish quality.
                show.pop('license',None)
                models.append({'provider':'ollama','name':name,'identity':tag,'show':show,'server_version':ollama['version']})
            if not ollama['tags'].get('models'): unavailable.append({'provider':'ollama','outcome':'not_run','reason':'no installed models'})
        except Exception as error:
            unavailable.append({'provider':'ollama','outcome':'not_run','reason':str(error)})
    sources=['frontend/src-tauri/src/summary/enhancement.rs','frontend/src-tauri/src/summary/grounding.rs',
             'frontend/src-tauri/src/summary/llm_client.rs','frontend/src-tauri/src/summary/summary_engine/models.rs',
             'frontend/src-tauri/src/summary/enhancement/evaluation.rs',
             'frontend/src-tauri/src/summary/enhancement/generation.rs',
             'frontend/src-tauri/src/summary/enhancement/wire.rs',
             'frontend/src-tauri/src/summary/enhancement/work.rs',
             'frontend/src-tauri/src/summary/summary_engine/client.rs',
             'frontend/src-tauri/src/summary/summary_engine/sidecar.rs',
             'llama-helper/src/main.rs']
    environment={'os':platform.platform(),'architecture':platform.machine(),'python':platform.python_version(),
                 'git_head':command('git','-C',str(ROOT),'rev-parse','HEAD'),
                 'code_sha256':{p:digest(ROOT/p) for p in sources},
                 'cargo':command('cargo','--version'),'rustc':command('rustc','--version'),
                 'network':'macOS sandbox denies non-loopback outbound for evaluator process and children'}
    if platform.system()=='Darwin':
        environment.update({'os_detail':command('sw_vers'),'cpu':command('sysctl','-n','machdep.cpu.brand_string'),
                            'ram_bytes':int(command('sysctl','-n','hw.memsize')),
                            'gpu':json.loads(command('system_profiler','SPDisplaysDataType','-json'))})
    else: environment['network']='Loopback-only adapters; OS network-blocked evidence not available on this host'
    output.joinpath('git-status-before.txt').write_text(command('git','-C',str(ROOT),'status','--short')+'\n')
    output.joinpath('inventory.json').write_text(json.dumps({'configured':configured,'models':models,'not_run':unavailable,
        'ollama_endpoint':endpoint,'environment':environment,'protected_before':before},ensure_ascii=False,indent=2)+'\n')
    code=output/'production-code'; code.mkdir()
    for name in sources:
        code.joinpath(Path(name).name).write_bytes((ROOT/name).read_bytes())
    with tempfile.TemporaryDirectory(prefix='meetily-e08-models-') as isolated:
        model_root=Path(isolated); (model_root/'models/summary').mkdir(parents=True)
        for model in models:
            if model['provider']=='builtin-ai':
                source=Path(model['file']); (model_root/'models/summary'/source.name).symlink_to(source)
        manifest={'output':str(output),'phase':args.phase,'isolated_model_root':isolated,'models':models,
                  'ollama_endpoint':endpoint,'repeats':args.repeats,'fixture':args.fixture,'environment':environment,
                  'scenario_matrix':args.scenario_matrix}
        output.joinpath('manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
        if models and not args.inventory_only:
            env=os.environ.copy(); env.update({'MEETILY_E08_MANIFEST':str(output/'manifest.json'),
                'MEETILY_LLAMA_HELPER':str(helper),'CMAKE_DISABLE_FIND_PACKAGE_OpenSSL':'TRUE',
                'PATH':str(ROOT/'scripts/macos-clt')+os.pathsep+env['PATH']})
            call=['cargo','test','--locked','--offline','-p','meetily','--lib',
                  'summary::enhancement::evaluation::real_local_models','--','--ignored','--exact','--nocapture','--test-threads=1']
            if platform.system()=='Darwin':
                profile='(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip "localhost:*"))'
                call=['/usr/bin/sandbox-exec','-p',profile]+call
            with output.joinpath('execution.log').open('w') as log:
                result=subprocess.run(call,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
            exit_code=result.returncode
        else: exit_code=0
    after=protected_inventory(data,helper)
    output.joinpath('protection.json').write_text(json.dumps({'before':before,'after':after,'unchanged':before==after},indent=2)+'\n')
    print(f'E08 evidence: {output}; {len(models)} available models; process exit {exit_code}; protected files unchanged: {before==after}')
    if before!=after: raise SystemExit('Protected installed artifacts changed during evaluation; inspect protection.json')
    raise SystemExit(exit_code)

if __name__=='__main__':
    main()
