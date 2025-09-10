"""Minimal CSV artifact example using flashvm.

1. Runs code in a microVM that writes /work/out/data.csv and /work/out/stats.json.
2. Uses expect patterns to fetch those files back as artifacts.
3. Saves inline contents locally (artifacts/ directory).
"""

import flashvm as vm
from pathlib import Path

code = """
import csv, json, pathlib, io
from statistics import mean
outdir = pathlib.Path('/work/out')
outdir.mkdir(exist_ok=True)

values = [10, 15, 20, 25, 30]
with open(outdir/'data.csv','w',newline='') as f:
    w = csv.writer(f)
    w.writerow(['id','value'])
    for i,v in enumerate(values, start=1):
        w.writerow([i,v])
stats = {
  'count': len(values),
  'avg': mean(values),
  'max': max(values),
  'min': min(values)
}
with open(outdir/'stats.json','w') as f:
    json.dump(stats, f)
print('wrote artifacts: data.csv, stats.json')
"""

res = vm.run(code, expect=["out/*.csv", "out/*.json"])
print(f"Exit code: {res['exit_code']}")
print(res['stdout'].rstrip())

arts = res['artifacts']
if not arts:
    print("No artifacts returned (check patterns or file sizes).")
else:
    out_dir = Path('artifacts')
    out_dir.mkdir(exist_ok=True)
    for a in arts:
        name = Path(a['guest_path']).name
        content = a.get('content')
        if content is None:
            print(f"{name}: not inlined (increase max_bytes_inline).")
            continue
        (out_dir / name).write_bytes(content)
        print(f"Saved {name} ({a['size_bytes']} bytes)")
    print(f"Local copies in: {out_dir.resolve()}")
