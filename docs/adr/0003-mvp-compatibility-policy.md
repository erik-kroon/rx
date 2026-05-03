# ADR 0003: MVP Compatibility Policy

Status: Accepted

## Context

`rx` is regular-language-first by default, but legacy regex input can contain
features that are non-regular or tied to a specific engine. Parser, linter, CLI,
migration, editor, playground, and dialect work need one MVP policy before they
decide whether to accept, explain, convert, or emit those features.

The compatibility-sensitive constructs for the MVP are:

- Backreferences.
- Lookbehind assertions.
- Recursive patterns.
- Conditionals.
- Engine-specific escapes and extensions.

These constructs are useful for migration analysis, but accepting them as normal
core patterns would weaken the safe core and make portability unclear.

## Decision

The MVP rejects compatibility-only constructs with structured, actionable
diagnostics. It does not model them as accepted pattern nodes in the canonical
safe core.

The structured diagnostic contract must identify:

- The feature category, such as `backreference`, `lookbehind`,
  `recursive_pattern`, `conditional`, or `engine_specific_escape`.
- The source context when input came from a parser.
- The requested or inferred dialect when dialect support is relevant.
- A short explanation that the feature is outside the MVP safe core.
- A suggested next action, such as rewriting the pattern, choosing a different
  dialect, or keeping the original regex outside automatic conversion.

Future compatibility representation may be added, but it must be explicit and
separate from the safe regular-language core. It should not be introduced as
public raw AST construction. If added later, parser lowering should produce a
distinct compatibility representation or explicit compatibility node family that
is dialect-gated before emission.

## Surface Behavior

Legacy regex parsing should fail during lowering when it encounters a
compatibility-only construct. The failure should be recoverable by tooling, so a
caller can still report other lint or migration context when the parser
architecture supports recovery.

Linting should report these constructs as unsupported-for-MVP diagnostics rather
than attempting semantic lint rules that depend on engine-specific behavior.
Lint output should use the same feature categories as parser and dialect errors.

Dialect errors should use structured unsupported-feature failures. A dialect
must not silently drop, approximate, or rewrite a compatibility-only construct.
If the safe core later gains an explicit compatibility layer, each dialect must
opt in to supported compatibility features.

CLI explanation should explain that the input contains an unsupported
compatibility-only construct, identify the construct, and stop before producing a
readable rx form or emitted regex for that construct. The CLI may still show
source location and surrounding supported context when parser recovery makes that
available.

Migration suggestions should preserve user trust by refusing automatic
conversion for patterns that depend on compatibility-only constructs. They
should suggest leaving the original regex in place, manually rewriting into the
safe core, or selecting a future compatibility-aware workflow when one exists.

## Consequences

- The safe core remains regular, portable, and easier to validate.
- MVP parser, linter, CLI, migration, editor, and playground work can share one
  unsupported-feature diagnostic policy.
- The project can still add a compatibility layer later without contaminating
  the safe-core API.
- Unsupported compatibility features require focused diagnostic tests in parser,
  lint, dialect, CLI, migration, editor, and playground slices.
- Existing implementation issues for parser (#8), linting (#9), CLI (#10),
  migration suggestions (#13), playground (#14), and editor integration (#15)
  should implement this structured rejection behavior where their surfaces touch
  legacy regex input or dialect compatibility.
