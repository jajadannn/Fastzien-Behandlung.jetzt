---
target: templates/landing.html
total_score: 29
max_score: 36
na_heuristics: 7
p0_count: 0
p1_count: 1
timestamp: 2026-08-30T14-55-49Z
slug: templates-landing-html
---
Method: dual-agent (A: design-review subagent · B: detector-scan subagent)

## Design Health Score

Mode: Persuade. Heuristic 7 n/a (no repeat-use workflow on a landing page).

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3 | No active-section state on in-page nav while scrolling. |
| 2 | Match Between System and Real World | 4 | Warm, informal German; jargon explained inline; section order matches real buyer decision flow. |
| 3 | User Control and Freedom | 4 | Mobile menu closes via X/Escape/outside-click/link-click; FAQ reversible; external link opens new tab. |
| 4 | Consistency and Standards | 3 | Three CTA labels, two button styles, one destination. |
| 5 | Error Prevention | 3 | No forms on-page; auth-branched hrefs handled correctly. |
| 6 | Recognition Rather Than Recall | 4 | Icons always text-labeled; price/hours/contact repeated near every CTA. |
| 7 | Flexibility and Efficiency of Use | n/a | No repeat-use workflow on this kind of page. |
| 8 | Aesthetic and Minimalist Design | 3 | Clean rhythm undercut by duplicated quick-answer/lead copy and 3 infinite hero animations. |
| 9 | Error Recovery | 2 | FAQ/reviews DB-driven with zero empty-state fallback; empty result = bare heading over blank space. |
| 10 | Help and Documentation | 3 | Real schema-marked FAQ, but one block at the bottom, not contextual. |
| Total | | 29/36 | Good (~81%) |

## Design Specificity Verdict

LLM assessment: copy is genuinely specific (names the actual competing category, states the real mechanism difference, volunteers a real contraindications list, real prices/drive-times). Visual grammar is template-shaped (hero/trust-bar/icon-grid/about-split/steps/testimonials/pricing/map/FAQ/CTA — swappable to any local-service business unchanged). The core differentiator (fascia = body-wide network, not local muscle) is argued in prose 3 times, shown nowhere.

Deterministic scan: zero findings, but Assessment B traced why in the tool's own source rather than trusting the clean exit code: .css files are structurally excluded from 8 rule types (incl. dark-glow) in this tool's page-analyzer gating, and landing.html itself has no literal shadow text/`<style>` block. So a real CSS-side issue would have been invisible to this scan by construction, not because it was checked and passed. Where the tool's line-based checks did run against main.css, they correctly found nothing (consistent with the earlier distill pass).

Correction: I told Assessment B a dark-glow suppression already existed in this branch's config.json for .btn-book's shadow; it doesn't — that suppression is on the main branch, never inherited by Rust-redesign. Assessment B independently re-verified the underlying claim against the rule's actual trigger logic anyway: 4px y-offset (not zero) on a confirmed-light background — genuine non-match, not a suppressed one.

## Overall Impression

Copy does real persuasive work (massage-contrast argument, volunteered contraindications). Design spends no visual budget on the one claim that needs it most. Three concrete implementation gaps sit under the polish: an incomplete reduced-motion guard (ironically affecting migraine sufferers, this page's own stated audience), sub-AA-contrast price text for an audience skewing older, and no empty-state handling for admin-managed FAQ/reviews content.

## What's Working

1. Symptom icon set is purpose-built, one-to-one with the six listed conditions.
2. Massage-contrast copy is unhedged and specific, not a vague superlative.
3. Contraindications disclosure is a real, rare credibility move.

## Priority Issues

[P1] Reduced-motion escape hatch doesn't reach 2 of 3 infinite hero animations — `*` selector misses `.hero::before`/`.hero::after` pseudo-elements; migraine-sufferers (the page's own stated audience) with prefers-reduced-motion set still get perpetual motion. Fix: extend selector to `*,*::before,*::after`. Suggested: /impeccable harden

[P2] Core differentiator argued in prose 3x, shown 0x. Fix: one simple diagram/body-outline motif. Suggested: /impeccable shape

[P2] Returning logged-in customers replay the entire cold-prospect pitch — hrefs branch on is_logged_in, content never does. Fix: returning-visitor hero variant or a "next appointment" shortcut. Suggested: /impeccable clarify

[P2] Secondary/price text fails WCAG AA (~3.5:1, ~3.2:1) — hits price-label/price-duration directly, worst for an older-skewing audience. Fix: darken --text-light, raise footer-bottom opacity. Suggested: /impeccable polish

[P2] Empty FAQ/reviews render as bare heading over blank space, no template guard; avg_rating hardcodes 5.0 even at review_count=0 (invalid schema). Fix: add {% else %} fallbacks; guard rating schema fields. Suggested: /impeccable harden

## Persona Red Flags

Jordan (confused first-timer): trademark badge appears before its explanation (4 sections early); quick-answer/section-lead repeat near-verbatim in 2 sections; contraindications box has no reassuring lead-in; promised "simple" booking, actual flow requires account+verification+profile.

Casey (distracted mobile): sole always-visible CTA sits top-right, least thumb-reachable; hamburger ~38px, under 44px tap-target minimum; hero photo's perpetual animation is an avoidable battery cost. Credit: hero content top-aligned (not centered) on mobile, a considerate thumb-reach choice.

"Ingrid," 58, chronic back pain (derived persona): lowest-contrast text on the page is exactly the pricing captions she needs most; base body text is font-weight:300 almost everywhere, costing readability for older eyes even where contrast passes; phone number never gets equal weight to online booking; no expectation-setting for the registration gate before "book now."

## Minor Observations

- Footer "© 2025" is stale (today is 2026-08-30).
- Final CTA's h2 is the only section heading not using class="section-title" (inline-styled instead, drift risk).
- Three CTA copy variants / two button styles for one action.
- Hero CTA nests a redundant is_logged_in check inside an already-true branch.
- No scroll-spy active state on in-page nav.
- The one scarcity/urgency line on the page lands at peak-end position, tonally inconsistent with the unhurried voice elsewhere.
- Admin accounts' in-body CTAs (not nav) still route to /portal/book, not /admin.

## Questions to Consider

- Where does a visitor actually see "body-wide network, not local muscle," rather than read it three times?
- What would this page look like if it assumed a meaningful fraction of visitors already have an account?
- What would CTA text look like if it matched the visitor's actual hesitation at each scroll position instead of repeating "jetzt buchen" six times?
