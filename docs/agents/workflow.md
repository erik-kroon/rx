# Agent Workflow

## Context Routing

- Start with [AGENTS.md](../../AGENTS.md) for repo-local operating guidance.
- Use [CONTEXT.md](../../CONTEXT.md) for domain language, product surfaces, and MVP boundaries.
- Use [docs/roadmap.md](../roadmap.md) for the current milestone baseline and next delivery sequence.
- Use [docs/adr](../adr) for durable architecture decisions.
- Treat `.context/attachments/pasted_text_2026-05-03_11-28-52.txt` as the current full PRD when available in this workspace.

## Skill Guidance

- Use `repo-context-bootstrap` when the repo-local agent context is missing or stale.
- Use `governance-distill` when instructions, docs, skills, or project conventions conflict or need durable cleanup.
- Use architecture or review skills only after checking the domain rules in [CONTEXT.md](../../CONTEXT.md).

## Work Tracking

GitHub issues are available for product planning. The initial end-state PRD is tracked as issue #1.

- Keep local handoff notes in `.context/`.
- Record durable product or architecture decisions as ADRs.
- Use GitHub issues for PRDs and planned implementation work when a tracker entry is useful.
- Treat issues #2 through #13 as covered by the `0.1.0` kernel baseline unless a future comment or reopened issue narrows them into follow-up work.
- Do not invent labels, states, or milestone names in committed docs until the repo establishes them.

## Verification Expectations

For context-only changes:

- Check that links resolve.
- Check that guidance does not conflict with existing docs.
- Confirm `git status --short`.

For future code changes:

- Add tests that match the risk of the change.
- Prefer golden tests for regex emission.
- Add parser round-trip, lint, dialect compatibility, and compile-fail tests when touching those areas.
