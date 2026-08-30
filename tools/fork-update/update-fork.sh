#!/usr/bin/env bash
# Rebase our custom features onto the latest stable upstream release.
#
# What this does, and does NOT do:
#   - Fetches upstream + origin, finds the latest stable (non-beta) upstream
#     tag, and compares it to the tag our current `integration<N>` branch is
#     based on. If there's nothing new, it exits quietly.
#   - If there IS a new tag: for each feature branch in FEATURE_BRANCHES,
#     creates `<branch>-<tag>` and rebases it onto the new tag.
#   - Merges every branch that rebased CLEANLY into a new `integration<N+1>`
#     branch, in FEATURE_BRANCHES order.
#   - Runs a fast stub-catalog cross build (aarch64) to smoke-test the
#     result. Past experience: upstream has silently removed/renamed things
#     our code depends on (a function, a struct field) WITHOUT causing a git
#     conflict — only `cargo build` catches those. This build step is not
#     optional.
#   - Only pushes to origin if the build succeeds.
#
#   It does NOT attempt to resolve rebase conflicts, and it does NOT try to
#   fix a failed build. Both require actually reading the code and deciding
#   what changed and why — auto-resolving blind has already produced broken
#   code once (a "clean" auto-merge that scrambled a SQL block). When either
#   happens, the script stops, leaves the repo in the conflicted/broken
#   state, and prints exactly what to look at. A human (or Claude, briefed
#   with this script's output) picks it up from there — same manual
#   technique documented in the project's CHANGELOG / commit messages:
#     - i18n / main.rs / server_fns mod.rs conflicts: almost always
#       additive-only, union the entries, drop only the conflict markers.
#     - replay-control-core/src/library/db.rs conflicts: git often aligns
#       unrelated shared lines (`let now`, `conn.execute(`) as if they were
#       the same edit ("tangled") — resolve by hand, don't trust automerge
#       here even when it reports zero conflict markers.
#     - Never copy a whole i18n file from one branch to another — it drags
#       in keys that belong to a different feature.
#
# Usage:
#   tools/fork-update/update-fork.sh            # check + attempt update
#   tools/fork-update/update-fork.sh --check     # only report, don't rebase
#   tools/fork-update/update-fork.sh --no-push   # do everything but the push
#
# Run from anywhere inside the repo clone that has both `origin` (our fork,
# push access via the SSH deploy key) and `upstream` (lapastillaroja/replay-control)
# remotes configured.

set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────

# Order matters: this is the same merge order used every time so far.
# achievements-gallery is stacked on top of game-status (written against
# its allowlist/state shape), so it must merge after it.
FEATURE_BRANCHES=(
    feat/game-notes
    feat/hltb
    feat/stats
    feat/game-status
    feat/achievements-gallery
    feat/favorites-collections
)

BUILD_TARGET="aarch64"
REPO_ROOT="$(git rev-parse --show-toplevel)"

# ── Args ──────────────────────────────────────────────────────────────────

MODE="update"
for arg in "$@"; do
    case "$arg" in
        --check) MODE="check" ;;
        --no-push) MODE="no-push" ;;
        *) echo "Unknown argument: $arg" >&2; exit 2 ;;
    esac
done

info()  { echo "==> $*"; }
warn()  { echo "!! $*" >&2; }
fatal() { echo "XX $*" >&2; exit 1; }

cd "$REPO_ROOT"

# ── 1. Fetch ──────────────────────────────────────────────────────────────

info "Fetching origin + upstream..."
git fetch origin --tags --quiet
git fetch upstream --tags --quiet

# ── 2. Find latest stable upstream tag ───────────────────────────────────

LATEST_TAG=$(git tag --merged upstream/main | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1)
[[ -n "$LATEST_TAG" ]] || fatal "Could not determine latest stable upstream tag."
info "Latest stable upstream release: $LATEST_TAG"

# ── 3. Find the tag our current integration branch is based on ──────────

CURRENT_INTEGRATION=$(git for-each-ref --format='%(refname:short)' 'refs/remotes/origin/integration*' \
    | sed 's#^origin/##' | grep -E '^integration[0-9]+$' | sort -t n -k1.12 -V | tail -1)
[[ -n "$CURRENT_INTEGRATION" ]] || fatal "No origin/integration<N> branch found — is this the right repo?"

CURRENT_BASE=$(git merge-base "origin/$CURRENT_INTEGRATION" upstream/main | xargs git describe --tags --abbrev=0)
info "Current $CURRENT_INTEGRATION is based on: $CURRENT_BASE"

if [[ "$CURRENT_BASE" == "$LATEST_TAG" ]]; then
    info "Already up to date with $LATEST_TAG. Nothing to do."
    exit 0
fi

NEXT_N=$(( ${CURRENT_INTEGRATION#integration} + 1 ))
NEXT_INTEGRATION="integration${NEXT_N}"
info "New release available: $CURRENT_BASE -> $LATEST_TAG"
info "Will build: $NEXT_INTEGRATION"

if [[ "$MODE" == "check" ]]; then
    info "(--check mode, stopping here)"
    exit 0
fi

# ── 4. Rebase each feature branch onto the new tag ───────────────────────

REBASED_BRANCHES=()
FAILED_BRANCHES=()

for branch in "${FEATURE_BRANCHES[@]}"; do
    src_ref="origin/${branch}"
    # Use the most recently rebased copy of this branch as the source if one
    # exists (feat/foo-vX.Y.Z), so we're never rebasing further back than
    # necessary. Falls back to the plain feat/foo ref on the first run.
    latest_existing=$(git for-each-ref --format='%(refname:short)' "refs/remotes/origin/${branch}-v*" \
        | sed 's#^origin/##' | sort -V | tail -1)
    [[ -n "$latest_existing" ]] && src_ref="origin/${latest_existing}"

    new_branch="${branch}-${LATEST_TAG}"
    info "Rebasing ${src_ref} -> ${new_branch} (onto ${LATEST_TAG})"

    git branch -f "$new_branch" "$src_ref" >/dev/null
    git checkout --quiet "$new_branch"

    if git rebase "$LATEST_TAG" >/tmp/rebase-${branch//\//_}.log 2>&1; then
        info "  clean rebase: $new_branch"
        REBASED_BRANCHES+=("$new_branch")
    else
        warn "  CONFLICT rebasing $branch — left in conflicted state on branch $new_branch"
        warn "  git status / git diff there to resolve, then: git rebase --continue"
        warn "  log: /tmp/rebase-${branch//\//_}.log"
        git rebase --abort 2>/dev/null || true
        FAILED_BRANCHES+=("$branch")
    fi
done

git checkout --quiet "origin/$CURRENT_INTEGRATION" -- . 2>/dev/null || true
git checkout --quiet main 2>/dev/null || git checkout --quiet "$LATEST_TAG"

if [[ ${#FAILED_BRANCHES[@]} -gt 0 ]]; then
    fatal "${#FAILED_BRANCHES[@]} branch(es) need manual conflict resolution before continuing: ${FAILED_BRANCHES[*]}. Not building or pushing anything."
fi

# ── 5. Merge all rebased branches into a fresh integration<N+1> ─────────

info "Building $NEXT_INTEGRATION from $LATEST_TAG..."
git branch -f "$NEXT_INTEGRATION" "$LATEST_TAG" >/dev/null
git checkout --quiet "$NEXT_INTEGRATION"

for branch in "${REBASED_BRANCHES[@]}"; do
    info "Merging $branch..."
    if ! git merge --no-edit "$branch" >/tmp/merge-${branch//\//_}.log 2>&1; then
        warn "MERGE CONFLICT merging $branch into $NEXT_INTEGRATION — repo left mid-merge for inspection."
        warn "log: /tmp/merge-${branch//\//_}.log"
        fatal "Stopping. This needs a human/Claude to read the conflict and resolve it (see script header for the known conflict patterns)."
    fi
done

info "$NEXT_INTEGRATION assembled cleanly: $(git rev-parse --short HEAD)"

# ── 6. Build smoke test (this is the step that has caught every silent ──
#      breakage so far — upstream renaming/removing something our code
#      depends on, with zero git conflict to warn us)

info "Cross-building ($BUILD_TARGET, stub catalog) to smoke-test the result..."
export BUILD_CATALOG_STUB=1
if ! bash build.sh --target "$BUILD_TARGET" >/tmp/fork-update-build.log 2>&1; then
    warn "BUILD FAILED. Nothing was pushed. $NEXT_INTEGRATION and the rebased feat/* branches exist"
    warn "locally only — fix the code, then re-run this script (it'll pick up where the branches are)."
    warn "Full log: /tmp/fork-update-build.log"
    echo "--- last 40 lines ---"
    tail -40 /tmp/fork-update-build.log
    exit 1
fi
info "Build OK."

if [[ "$MODE" == "no-push" ]]; then
    info "(--no-push mode, stopping here — nothing pushed to origin)"
    exit 0
fi

# ── 7. Push ───────────────────────────────────────────────────────────────

info "Pushing rebased branches + $NEXT_INTEGRATION to origin..."
git push origin "${REBASED_BRANCHES[@]}" "$NEXT_INTEGRATION"

info "Done. $NEXT_INTEGRATION ($LATEST_TAG base) is on origin, ready to deploy."
info "Deploy: build.sh already produced target/${BUILD_TARGET}*/release/replay-control-app + target/site/"
info "        copy those to /opt/replay-fork/ on the Pi and restart replay-control-fork.service."
