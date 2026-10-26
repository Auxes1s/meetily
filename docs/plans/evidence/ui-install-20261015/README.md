# UI installation and stale-app cleanup — 15 October 2026

The user requested removal of stale Meetily apps and installation of the new UI
build as their application. The canonical installed app is now:

`/Applications/meetily.app`

Its bundle identifier remains `com.meetily.ai`, retaining the existing library
location. The source was the verified
`target/release/bundle/macos/Meetily Notepad.app` from the
[interaction verification](../interaction-stress/README.md). The staged and
installed bundles matched the release file-for-file and passed strict deep
signature verification. The main executable SHA-256 is
`53408f31f259865b26794cbaf34ad7a550f51a5fc908ceedcb10f4f4e113a60a`.

Fourteen timestamped backup applications, one duplicate user-local app and the
superseded canonical app were moved to Trash through Finder: **16 old copies**.
The Trash was not emptied. Cleanup operated only on verified application bundles;
no meeting-data or recording folder was moved or deleted. Eight generated
test/build bundles, including the release source used for installation, were
preserved in the project archive excluded from Spotlight:

`target/archived-app-bundles.noindex/20261015T103927/`

The release source is in its `release/Meetily Notepad.app` subdirectory. Future
build commands recreate the normal output paths.

The canonical app registration was refreshed without resetting the global Launch
Services database. A final Spotlight application query returned only
`/Applications/meetily.app`. Resolving
`meetily` by application name returned `/Applications/meetily.app/`. Launching
with `open -a meetily` produced a running process from that path with one native
window. The application was left open for the user.

The [installation record](installation.json) retains the exact removed paths,
copy/signature checks, registration results and launch verification. Some old
registrations returned an already-unregistered error; these did not prevent
Finder removal or successful registration of the new canonical app. The earlier
build/interaction evidence describes its own test-time installation boundary;
this subsequent installation was separately authorized by the user.
