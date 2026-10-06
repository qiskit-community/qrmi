# Vendor implementation guidelines

This document describes how we handle changes that touch vendor implementations in QRMI, and records each vendor's preferences.

The aim is to keep QRMI moving while respecting each vendor's priorities and bandwidth, and without sending more review requests than necessary.

> Status: draft, proposed at the QRMI Community Call on [date]. To be revisited on [date].

## Change tiers

Every PR that touches a vendor implementation falls into one of three tiers. State the tier in the PR description.

| Tier | Typical changes | Review rule |
|---|---|---|
| **A: Approval required** | Changes to behaviour or APIs, such as job submission, authentication, or configuration semantics | Vendor approval is required before merge. The timeout does not apply. |
| **B: Review requested** | Refactors, dependency bumps, updates that follow changes to the shared interface, small internal bug fixes | The vendor is requested as a reviewer. If there is no reply within the vendor's timeout(default 1 week), maintainers may merge. |
| **C: Notify only** | Typos, docs, lint, formatting, CI, example code changes | Maintainers may merge. The vendor is tagged for visibility only. |

If you are unsure which tier applies, choose the higher one.

A vendor can move its own default up or down a tier in the [vendor preferences](#vendor-preferences) table. For example, a vendor may ask for every change to be treated as Tier A, or may accept Tier C for most changes during a quiet period.

## Timeout

The timeout is how long a Tier B review request waits for the vendor.

- If the vendor approves or comments within the timeout, follow their feedback.
- If there is no reply within the timeout, maintainers may merge.
- A vendor can ask for more time at any point by commenting on the PR.
- The timeout never applies to Tier A changes.

The default timeout is **[2 weeks]** unless a vendor sets its own.

## Pull request expectations

Every PR that touches a vendor implementation should include:

- **Tier**: A, B or C
- **Expected merge date**: when the author hopes to merge
- **Urgency**: for example, low, medium or high, with a one-line reason if not low
- **Affected vendors**: which implementations the change touches

Prefer one PR for a change that touches several vendors in the same way, rather than one PR per vendor. This keeps review requests and notifications to a minimum.

## Review requests and CODEOWNERS

A review request means *action needed*. To keep it meaningful:

- CODEOWNERS for each vendor covers only paths with vendor-specific behaviour, not shared code or mechanical files such as tests, CI and dependency manifests. [Adjust to the actual repository layout.]
- For Tier C changes, tag the vendor in the PR description or with a label (for example `vendor:<name>`) instead of requesting review.
- A vendor that cannot review for a while can ask to be removed from CODEOWNERS temporarily and follow changes through the Open Call summary instead.

## Open Call summaries

When vendors are short on time, maintainers summarise recent changes to vendor implementations at the Open Call. Vendors can raise concerns there, or afterwards on the relevant PR.

## Vendor preferences

Each vendor fills in its own row and can update it at any time by opening a PR against this file.

| Vendor | Contact | Default tier | Timeout | Notes |
|---|---|---|---|---|
| AnB | Jamie Machin | C (interim) | [__] | Emulators only for now; revisit after the backend rework |
| Pasqal | Aleksander Wennersteen | [A / B / C] | [__] | [__] |
| IBM, IQM, OQTPUS | Munetaka Ohtani | A | [__] | [__] |
| [Vendor] | [Name] | [A / B / C] | [__] | [__] |
| [Vendor] | [Name] | [A / B / C] | [__] | [__] |

**Column guide**

- **Contact**: who to tag on PRs that touch your code.
- **Default tier**: the tier to apply to changes in your implementation when you want to differ from the general rules above.
- **Timeout**: how long a Tier B review request waits for you before maintainers may merge. Leave blank to use the default.
- **Notes**: anything else, such as busy periods, a planned rework, or a preferred way to be contacted.
