# GitHub replacement and Wiki workflow

Repository: https://github.com/TrissTheBoss/Sunset

This delivery lives on a dedicated handbook branch. Its tree contains the new planning kit, not the old engine. Main and the published Wiki were not replaced by this delivery. This gives a concrete reviewable starting point while retaining the old implementation's ancestry.

## Adopt the new project

Inspect default branch, current HEAD, local changes, open PRs, releases, CI and Wiki state. Preserve unrelated work. Record the old HEAD and create an archival tag/backup as part of the explicitly authorized replacement operation. Verify recovery before switching the active tree.

Review the handbook branch diff: old production/configuration files removed, new rules/docs present, no accidental secret/license loss. Retain required historical license/attribution records if any old content is carried forward. Old files are not reused by default. Do not delete .git or force-push rewritten history.

Merge a replacement PR or fast-forward the approved main branch as appropriate. If main changed, resolve against current state and rerun checks; do not overwrite unseen work. Close/supersede obsolete work explicitly with links, not silent deletion. The requested complete overwrite means replacing the active file tree, not destroying recoverable history.

Keep small purpose-specific branches/PRs, evidence in descriptions, and annotated version tags for releases. GitHub issue templates should reference requirement/parity/test IDs. Protect main with relevant checks. No token in repository files or shell logs.

## Wiki publication

GitHub Wiki has its own Git repository: https://github.com/TrissTheBoss/Sunset.wiki.git . Its initial page may need creation through GitHub before cloning. Verify availability/permissions.

Treat main-repository wiki/ Markdown as canonical authored content. At adoption, back up the current Wiki, review deletions of obsolete pages, publish Home and sidebar plus the included starter pages. Extend the Wiki for architecture, content definitions, modules, hooks, renderer packs, data/resource compatibility, automation, troubleshooting and contribution.

Implement a publication task that checks Markdown links, generates API reference from contract metadata (not stale prose), records source commit/target/API versions, transforms repository-relative links for Wiki navigation, and updates only managed pages. Stage and inspect the Wiki diff. Push through a narrowly scoped credential tested for Wiki access; do not assume the default workflow token can write a separate wiki repository. Do not embed credentials in clone URLs.

Publish on tested release tags or explicitly authorized documentation updates. Never describe future APIs as shipped. A failed Wiki publish fails documentation delivery and remains visible; main success alone does not prove Wiki success. Source docs and API examples change in the same PR as contracts. Keep old API versions discoverable with deprecation/migration notes.
