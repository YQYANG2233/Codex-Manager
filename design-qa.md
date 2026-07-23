# Skills Repository UI Design QA

## Evidence

- Source visual truth:
  - `/tmp/codex-clipboard-N72mW9.png` — cc-switch repository catalog.
  - `/tmp/codex-clipboard-0Yn2wn.png` — cc-switch repository management.
- Browser-rendered implementation:
  - `/tmp/codex-design-qa/skills-repository-catalog-final3.png`.
  - `/tmp/codex-design-qa/skills-repository-management-final3.png`.
- Viewport: 2048 × 1189 for final visual captures; the same flow also passed at 1440 × 900.
- State: Simplified Chinese, `tech` theme with classic appearance, connected service, Skills installation selected, repository catalog populated with the four built-in repositories. The management capture shows the repository dialog open.
- Full-view comparison evidence:
  - `/tmp/codex-design-qa/comparison-final-catalog-full.png`.
  - `/tmp/codex-design-qa/comparison-final-management-full.png`.
- Focused comparison evidence:
  - `/tmp/codex-design-qa/comparison-final-catalog-focus.png` — search, filters, grid density, cards, and actions.
  - `/tmp/codex-design-qa/comparison-final-management-focus.png` — add-repository form and synchronized repository rows.

## Findings

No actionable P0, P1, or P2 differences remain.

- Fonts and typography: both targets use a compact system sans-serif hierarchy. CodexManager keeps its existing Segoe UI/PingFang SC stack, weights, monospaced paths, truncation, and line heights; all labels remain readable without collision.
- Spacing and layout rhythm: the final wide layout uses three catalog columns, aligned search and filter controls, consistent card heights, and a wider repository dialog. The 1440 × 900 run correctly falls back to the denser two-column layout.
- Colors and visual tokens: the implementation intentionally retains CodexManager's blue primary and glass-console tokens instead of copying cc-switch's green accent. Contrast, selected tabs, status badges, and destructive actions remain semantically clear.
- Image and icon fidelity: the reference contains no required raster product imagery. All visible actions use the repository's Lucide icon system; no placeholder imagery, CSS drawings, emoji, or hand-authored SVG substitutes were introduced.
- Copy and content: the menu is “Skills 与插件”, the outer tabs distinguish “Skills 安装” from “Codex 插件安装”, and repository/status filters now render localized labels rather than raw `all` values. The four built-in repositories match the source behavior.
- Interaction and accessibility: tabs, search, repository dialog open/close, built-in delete protection, plugin scrolling, install confirmation, and long error-toast scrolling were exercised. Controls have role/name coverage in Playwright, and the final run recorded no page or console errors.
- Intentional adaptation: repository management is a modal instead of a dedicated page, and the CodexManager shell remains visible. This follows the existing app navigation and dialog system while preserving the source workflow and information architecture.

## Comparison History

1. Initial capture: `/tmp/codex-design-qa/skills-repository-catalog.png` showed the workspace stuck at the fade animation's zero-opacity state in the static-export browser. Removed the redundant PageWorkspace entrance animation; the post-fix catalog is fully opaque in `skills-repository-catalog-final3.png`.
2. First post-opacity pass: filters exposed raw `all` values, the wide catalog stayed at two columns, and repository management was cramped. Added localized selected-value rendering, a three-column 2XL grid, four representative built-in repository rows, and a responsive 980px dialog.
3. Final comparison: the full and focused comparison images above show no remaining P0/P1/P2 mismatch. Functional Playwright coverage passed at both tested desktop sizes with console-error collection enabled.

## Implementation Checklist

- [x] Preserve the existing CodexManager design system and navigation shell.
- [x] Match the repository / skills.sh discovery structure and searchable card catalog.
- [x] Show all four built-in repositories with sync state and refresh controls.
- [x] Separate standalone Skills installation from full Codex plugin installation.
- [x] Verify primary interactions and browser console state.

## Follow-up Polish

- P3: a future iteration may add optional compact/list density controls for repositories containing hundreds of Skills. This is not required for the current workflow.

final result: passed
