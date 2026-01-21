CHECKLIST WORKFLOW (DEFAULT FOR ALL MILESTONES)

1) PLAN
- Write a short plan for the milestone (3-7 steps).
- List exact files/modules expected to change.
- List acceptance checks (what "done" looks like).

2) IDENTIFY
- Identify shortfalls, open questions, and any missing inputs.
- If missing inputs are REQUIRED to proceed safely, STOP and ask targeted questions.
- If missing inputs are OPTIONAL, ask before proceeding or leave a comment/todo ect.

3) LOCK
- Restate the final plan in 3 bullets.
- State assumptions (if any).
- State what will NOT be changed (to avoid scope creep).

4) EXECUTE
- Apply only the locked plan.
- Do not add unrelated refactors.
- Adding dependencies is allowed ONLY if:
  - it is necessary for the milestone, AND
  - it is minimal, AND
  - it is recorded in the milestone notes.

5) VERIFY
- Run:
  - cargo fmt
  - cargo clippy
  - cargo test
- Also run:
  - app smoke (or manual smoke) if the change is observable.
- If verification fails, fix or revert; do not leave the repo broken.

STOP RULES
A) Missing info / shortfalls:
- If the milestone cannot be completed without additional info, STOP and ask questions.
- Ask only the minimum questions required to proceed.

B) Scope/budget limit:
- If you cannot finish within the milestone scope, STOP and leave TODOs, then ask questions.
- Do not expand scope or start adjacent milestones.
- If obvious assumptions are not contained in checklist and undone, stop an relay this information.
