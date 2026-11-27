# ZzClawTerm fork notes

- Fork: <https://github.com/jackfahdin/suppaftp>
- Upstream: <https://github.com/nyakang/suppaftp>
- Branch: `nyaterm`
- Source revision: `8a9882d1f591711cd83fad3ca370b14a44623025`

The source tree is identical to the upstream `nyaterm` branch. ZzClawTerm uses
this revision for FTP/FTPS support without local implementation patches.

## Validation on Windows

`cargo +1.99.1 check -p suppaftp` passed, and all 13 `types::` unit tests
passed. The complete library run compiled successfully and produced 134 passing
tests, 43 failures and 2 ignored tests. Every failure came from the test
container fixture: Docker is unavailable on this host, so the
`delfer/alpine-ftp-server` container could not be created. No source or test was
changed to bypass that environment requirement.
