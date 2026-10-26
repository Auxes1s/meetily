import subprocess, pathlib, os, json, time, sys, shutil
run=pathlib.Path(__file__).parent; source=run/'source'
env=os.environ.copy(); env.update({'CI':'true','NEXT_TELEMETRY_DISABLED':'1','CARGO_NET_OFFLINE':'true','CMAKE_DISABLE_FIND_PACKAGE_OpenSSL':'TRUE','CARGO_TARGET_DIR':str(run/'native-target'),'PYTHONDONTWRITEBYTECODE':'1'})
env['PATH']=str(source/'scripts/macos-clt')+os.pathsep+env['PATH']
group=sys.argv[1]
groups={
'frontend': [('frontend-tests',['bun','test','frontend/tests'],source,180),('frontend-build',['pnpm','--dir','frontend','build'],source,300),('typecheck',['pnpm','--dir','frontend','exec','tsc','--noEmit','--incremental','false'],source,180),('lint',['pnpm','--dir','frontend','lint'],source,60)],
'auxiliary': [('python-tests',['python3','-m','unittest','discover','-s','scripts/tests'],source,180),('portable-ggml-tests',['node','.github/verify-portable-ggml.test.cjs'],source,60),('onboarding-node-tests',['node','frontend/tests/lib/onboarding-summary-model.test.mjs'],source,60),('rustfmt',['cargo','fmt','--all','--','--check'],source,120)],
'native': [('rust-tests',['cargo','test','--locked','--offline','-p','meetily','--lib'],source,1200),('clippy',['cargo','clippy','--locked','--offline','-p','meetily','--lib','--tests','--','-D','warnings'],source,1200)],
'research': [('local-research',['repo-research','--path',str(source/'frontend/src-tauri/src'),'--mode','evidence','--depth','quick','--output-budget','compact','--question','Locate native stop/finalization and recovery paths that ignore persistence errors, and deletion writer lease lifetimes. Return exact source spans and distinguish implementation from tests.','--fresh','--json'],source,300)]}
results=[]
for name,command,cwd,timeout in groups[group]:
 started=time.monotonic(); log=run/(name+'.log')
 with log.open('w') as stream:
  try:
   p=subprocess.run(command,cwd=cwd,env=env,stdin=subprocess.DEVNULL,stdout=stream,stderr=subprocess.STDOUT,timeout=timeout)
   result={'name':name,'command':command,'cwd':str(cwd),'exit_code':p.returncode,'seconds':round(time.monotonic()-started,2),'log':log.name}
  except (subprocess.TimeoutExpired,OSError) as exc:
   result={'name':name,'command':command,'cwd':str(cwd),'exit_code':None,'seconds':round(time.monotonic()-started,2),'error':str(exc),'log':log.name}
 results.append(result); (run/(group+'-results.json')).write_text(json.dumps(results,indent=2)+'\n')
 print(json.dumps(result),flush=True)
