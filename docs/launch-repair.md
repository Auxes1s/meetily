# Meetily launch repair — 14 October 2026

Historical receipt. On 15 October, the verified UI build was installed as the
single `/Applications/meetily.app` copy. The duplicate and timestamped app
backups described below were moved to Trash at the user's request. See the
[current installation receipt](plans/evidence/ui-install-20261015/README.md).

The repeated 11:11 crash reports identify
`/Applications/meetily.app/Contents/MacOS/meetily`, the September upstream bundle.
It predates the notepad/enhancement database schema. The newer local app opened
successfully against the same library; the database returned `ok` from its
integrity check.

The current source was rebuilt with `./scripts/meetily-notepad.sh build`, including
L05 exports and the local updater setting. Both installed launch copies now use
the verified release bundle:

- `/Applications/meetily.app`
- `~/Applications/Meetily Notepad.app`

Before replacement, the running local app quit normally through its save-flush
handler. Staged bundles passed `codesign --verify --deep --strict` and matched
the build file-for-file. Previous bundles were retained at:

- `/Applications/meetily.previous-20261014-112147.app`
- `~/Applications/Meetily Notepad.previous-20261014-112147.app`

Backup file manifests match the originals. Both new copies pass signature and
file-manifest checks. Their main executable SHA-256 is
`72df7cc0cc99fceda70f98334ce5453fbf0e8715dfdb6b0a4e77d9c6b870d05a`.

The normal `/Applications/meetily.app` path was launched and remained running.
Its native window rendered the workspace and new Export control. At the first
post-replacement verification, the shared library contained **11 meetings and
466 transcript passages**, and every table matched the snapshot taken immediately
before replacement. Integrity and foreign key checks passed. Earlier launch/index
work changed only disposable search projections; original sources remained
unchanged. Subsequent library activity changed records and initiated deletions;
those later changes are separate from the replacement comparison and were not
rolled back.

SQLite online backups and verification receipts remain outside the repository
in the private temporary directory identified by
`/tmp/meetily-launch-repair-current.txt`. These backups include the local library
and must remain private. The prior L05 evidence package describes the earlier
isolated test run; this subsequent repair is the installation and actual-library
launch check.
