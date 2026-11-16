# changelog-build-manual

How versions are cut and changelogs are written on this line of work. Succinct, by the book, no exceptions.

## Versioning

Semver, three numbers, final product starts at 1.0.0:

- MINOR bump = a feature ships (installers, the musl bundle, the srt stripper).
- PATCH bump = fixes or polish only.
- The version lives in exactly ONE home (the workspace manifest's package version). Every binary, installer, tag and document derives from it; the lockfile entries bump in the same commit. Never hardcode a version string anywhere else.
- The tag is lightweight: `vX.Y.Z` on the release commit. No annotated tag, no dates, no per-platform tags.
- One tag ships every platform artifact. No per-platform releases, no bare binaries.

## Release path

1. Bump the version in its one home (+ lockfile entries) in the prep commit.
2. Local gate before anything moves: format check, lint with warnings denied, full test suite, packaged selftest green.
3. First CI build run = the sizes run. Download its artifacts, verify every byte on a real machine (digests, selftests, installers, payload pins).
4. Commit B: README asset table + checksum block with the sizes-run digests, staged release notes.
5. Final CI run on commit B; full re-verify including document byte-compares.
6. Commit C: checksum block swapped to the final run's digests. Tag, push branch then tag, create the release with the notes file and every artifact.
7. Verify at the DESTINATION: re-download from the published page, assert the api digest and size of every asset, the notes checksum block against the bytes, and README/notes block equality. Then the platform proofs again from those bytes.
8. Record gate evidence in the spec sheet (run ids, digests, sizes - facts only). Delete scratch.

Digest rule: digests change between runs (archive timestamps) - the published block always carries the digests of the bytes actually attached, and a re-verification reads the release page, not CI.

## Changelog structure

ONLYOFFICE shape: per-version chapters, newest first, no dates, no names, no process narrative.

```
# Changelog

## 2.4.0

### New features
#### All platforms
* Added the srt timecode stripper ...

#### Windows
* ...

### Fixes
* ...
```

Rules:

- `# Changelog` once at the top; one bare `## X.Y.Z` chapter per version, newest at the top.
- No dates under the headings. The git history carries when; the changelog carries what.
- No owner names, no decision attributions, no run ids, no process narrative, no "this commit" markers. Those live in git history and the spec sheet.
- Chapters carry what the person using the product gets: `### New features`, `### Changes` and `### Fixes`. Platform differences get `####` subheads: All platforms, Windows, Linux, macOS, Linux (musl).
- Terse bullets, one line each. Facts, not salesmanship.
- The changelog ships inside the bundles: it must stay byte-equal to the repo tree, and it never references the release's own run ids or sizes.
