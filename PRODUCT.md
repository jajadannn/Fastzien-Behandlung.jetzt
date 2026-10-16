# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary users are German-speaking adults (informal "du" address) in Schramberg-Sulgen and the surrounding region who feel tense and want to take time for mobility, relaxation, posture awareness, and wellbeing. They include people with desk-based or physical jobs and active adults. This is a self-pay wellness and bodywork service, not medical care. People with symptoms of disease must seek medical assessment before booking.

A second user role is the practitioner himself (Thilo Seifried) acting as admin: managing customers, appointments, payments, waitlist, site content (FAQ/reviews/settings), and calendar sync from an admin dashboard.

## Product Purpose

A public website plus booking/customer-portal system for a solo, certified Gantke®-method fascia-treatment practice. It lets prospective clients understand the method and book a first session with minimal friction, lets existing clients manage appointments/credits/waitlist themselves, and lets the practitioner run the whole operation (bookings, payments, calendar, communications) without office staff.

## Positioning

Personal, certified Gantke® fascia and bodywork delivered by Thilo Seifried, with an individual conversation, manual work with the whole fascia network, and exercises for everyday life. No promises of pain relief, cures, first-session results, comparative efficacy, or a fixed number of sessions.

## Operating Context

- Sessions are ~90 minutes, either in-practice (Sulgauer Straße 24, 78713 Schramberg-Sulgen) or as a home visit (+15 € travel surcharge) within the region.
- Opening hours: Mo–Fr 16:00–22:00, Sa 9:00–19:00 (configurable in admin settings).
- Pricing: single session (~195 €) or a 10-session credit pack (~169,90 €/session); both configurable in admin settings, not hardcoded truths.
- No reimbursement by health insurers or other insurers. A receipt/invoice is provided. Payment is cash or bank transfer.
- Cancellation is allowed up to 24 hours before the appointment via the customer portal; customers booking without portal access contact the practitioner by phone or email.
- Calendar sync via Nextcloud/CalDAV (OAuth2 or app-password), plus a per-customer ICS subscription link.
- Public booking at `/buchen` requires no registration: choose a slot, enter name/phone/email (address for home visits), and confirm through a 30-minute email link. The slot is not reserved before confirmation. The server rechecks availability and records the appointment, payment and token consumption atomically. Guest bookings do not consume existing package credits. Profile/credential data on existing accounts remains unchanged.
- New portal accounts must verify their e-mail before full portal access (booking, waitlist, credits) unlocks; profile completion (name) is required before booking; address is required only for a home visit.
- When the practitioner cancels, he can propose alternative time slots directly in the cancellation e-mail/portal flow.
- Transactional e-mail is a first-class, technically constrained surface: must render correctly in Outlook desktop and other clients with poor CSS support (inline styles, table-based layout only, no external stylesheets/fonts/SVG, gradients need a solid-color fallback).

## Capabilities and Constraints

- Stack: Rust (Actix-Web) backend, Tera server-rendered templates, SQLite (rusqlite), SMTP e-mail via `lettre`.
- Undecided/not product truth: exact prices and hours are admin-configurable, not fixed facts to hardcode into new copy.

## Brand Commitments

- Legal/trading name: "Faszienbehandlung Thilo Seifried", method name "Gantke® Faszienbehandlung" — the ® must always be preserved when the method name is written.
- Compliance-critical: the practice is explicitly **not** Heilkunde under the German Heilpraktikergesetz and does not replace medical diagnosis or treatment (stated in the Impressum). Copy must never imply medical diagnosis, cure, or treatment claims.
- Voice: informal "du", warm, calm, professional — never salesy or hyped.
- No emoji anywhere, on any surface (web, portal, admin, e-mail) — established and enforced this project across a full redesign pass. Decorative glyphs use a hand-built monochrome stroke-icon system (inline SVG sprite) on the web; e-mail cannot use that sprite (see Operating Context) and instead stays icon-free with clean typographic hierarchy.
- Existing visual identity (web): sky blue (#70AECD → #4d93b8) and plum (#964279 → #7a3661) as the two brand colors, off-white background (#f7fbfd), Cormorant Garamond for display/serif headings, DM Sans for body text, fully rounded ("pill") buttons, generous whitespace, calm/subtle motion (no flashy animation).
- Logo asset at `/Logo.png`.

## Evidence on Hand

- Existing customer reviews remain available in admin. Public display requires explicit approval of authenticity, publication consent, and suitability. Original quotations are not rewritten. No sample reviews are seeded into new databases; disease/cure testimonials are not published.
- Real contact details exist in admin settings (address, phone, e-mail) and must be sourced from there / kept consistent with them, not invented.
- The canonical domain is `faszienbehandlung.jetzt` (without a hyphen).
- October 2026 marketing guide is the revised product/copy brief. Three-session packages, new payment methods, a free pre-booking phone appointment, company offerings, and follow-up services are drafts only, not published or enabled services.
- Real portrait: `/Portrait.jpg`. Topic pages cover back tension, neck/shoulders, posture, sport, and price/appointment flow. No fabricated regional offices or local reviews.
- Optional acquisition-channel responses are collected in guest booking and shown as booking counts in admin statistics. No third-party analytics or marketing trackers.

## Product Principles

1. One coherent, calm, professional identity across every surface — web, customer portal, admin, and e-mail — with zero colorful emoji anywhere.
2. Personal and small-scale by design: a single named, certified practitioner, real reviews, no invented scale or fabricated social proof.
3. Compliance-aware copy: never cross from wellness/bodywork language into medical diagnosis or cure claims.
4. Low-friction booking is the core value; every surface should make discovering, booking, and managing an appointment as frictionless as possible.
5. E-mail is a constrained, high-stakes surface (transactional, must render correctly everywhere) — polish comes from typography, hierarchy, and restraint, not from techniques that break in Outlook.

## Accessibility & Inclusion

The October 9, 2026 website guideline request sets WCAG 2.2 AA as the technical test target. The operator confirmed fewer than ten employees and annual revenue below EUR 2 million, so the BFSG service exemption for microenterprises is documented. Automated browser checks are supplemented with keyboard testing; comprehensive screenreader/real-device assessment remains a separate evidence item.

## Website Guidelines — October 9, 2026

- The user confirmed self-hosting by Thilo Seifried, permission for logo/photos/reviews, and an existing VAT ID. The VAT ID is intentionally left for the user to enter under Admin → Settings → Legal information; it is rendered in the Impressum from that field.
- Consumer information: `/buchungsbedingungen`; two-step, no-login withdrawal: `/widerruf`, including without JavaScript. Withdrawal receipt and timestamp are stored before SMTP delivery; admin reviews identity/contract and handles cancellation/refund.
- Final booking checks total, duration and card use against the displayed quote. A service within 14 days requires explicit early-start consent. Versioned confirmation and withdrawal notice are persisted with the appointment, then emailed through a retrying outbox.
- Card allocation currently creates a per-session payment record. Do not claim a card is prepaid unless payment evidence and accounting behavior support that statement.
- Account erasure keeps necessary accounting data in a separate admin-only archive. Bookkeeping originals, retention exceptions and provider copies remain operational responsibilities.
- New/changed passwords require at least 12 characters (at most 72 UTF-8 bytes for bcrypt). JWTs carry unique session IDs; explicit POST logout revokes the session on the server.
- `WEBSITE-RICHTLINIEN-ABNAHME.md` contains the requirement mapping, test evidence and remaining operational evidence. No live deployment is implied by local verification.

- Data export includes recorded booking sources, pending requests, withdrawal processing data and queued contractual receipts. Email ownership must be verified before exporting or reading invoices/calendar data. A guest booking is not linked to a pre-existing unverified signup. Calendar credentials created before verification are replaced on first verification or after an email change; links of already-verified accounts remain stable.
