# Normal-User Product Smoke (grok branch - program operating guide in main README)

This checklist verifies the product path:

```text
Add Data -> Work -> Results -> Answer -> Report -> Feedback -> Outcome -> Detail Review
```

It is product verification, not generic smoke tooling. Use synthetic data only.
Do not dump runtime/private data, `.env`, logs, raw artifacts, or database
contents.

## Codex-Safe Check

Codex may run only the non-Docker marker check:

```bash
scripts/normal-user-product-smoke.sh --check
```

This confirms the expected normal-user surfaces are present in the UI source:

- guided manual Add Data flow;
- Work processing status;
- Results evidence-answer area;
- persisted answer save control;
- report workflow;
- feedback and outcome workflow;
- source/evidence history review.

Passing this check does not mean the live product path is verified.

## Owner WSL Runtime Verification

Run this in normal WSL, not in the Codex sandbox:

```bash
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run --record
scripts/operator-smoke-check.sh --latest-result
```

The owner-run smoke should use only synthetic data and should record a safe
summary under `.igy6-local/smoke-results/`.

## Synthetic Input

Use a clearly synthetic source:

```text
Source title: DIFF-232 Synthetic Product Smoke
Source type: manual_upload
Sensitivity: internal
Trust state: review-needed
```

Synthetic text:

```text
DIFF-232 synthetic product smoke note.
The support decision is to review router warranty steps before ordering parts.
The expected next action is to create an evidence brief with citations.
The outcome target is useful only if the report cites stored evidence.
```

Question:

```text
What did the DIFF-232 smoke note say needs review? Cite the evidence.
```

## Checklist

1. Add Data

   Create or select a supported `manual_upload` source from the normal Add Data
   tab. Submit the synthetic UTF-8 text. Do not use account scraping, browser
   scraping, connectors, binary files, or private data.

2. Work

   Open Work and verify a processing item is visible. Wait for processing to
   complete or record the failure honestly.

3. Chat / evidence results

   Open Chat and verify collection runs, artifact metadata, documents,
   chunks, and evidence records are visible where processing completed.

4. Answer

   Ask the synthetic question in Ask Over Evidence. Verify the answer packet
   separates cited facts, assumptions or uncertainty, missing information, and
   citation identifiers. If no evidence is found, record that as insufficient
   evidence rather than a pass.

5. Persisted Answer

   Save the answer record. Verify the saved answer appears in Chat history
   after reload.

6. Report

   Create an evidence brief or decision note report from the Chat report
   workflow. Render markdown only if the UI/API path supports it. Do not claim
   PDF export.

7. Feedback

   Record feedback against the saved answer, evidence, report, or other
   supported target. Use a synthetic note such as `DIFF-232 feedback: citation
   review useful`.

8. Outcome

   Record an outcome only for an API-supported target such as report, work item,
   prediction, recommendation, hypothesis, or pattern. Outcome records for
   answer records are not currently supported.

9. Detail Review

   Review source/evidence detail and lineage. Confirm the UI shows bounded
   previews and metadata rather than dumping raw private contents.

## Pass Criteria

The live product smoke can be marked passed only when the owner-run WSL smoke
and the manual checklist both succeed with synthetic data.

## Honest Failure States

- Docker unavailable in Codex is not a product failure.
- No evidence after upload is a processing or setup failure, not proof the
  real-world information is absent.
- Missing report render means report export remains pending for that path.
- Unsupported outcome targets must remain unsupported and should not be forced.
