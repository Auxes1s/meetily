# Enhancement source/output review sheet

Semantic assessments below are **Codex review**. Human acceptance is pending for every run. Read complete source labels and raw response in the linked JSON when a row is ambiguous. Structural acceptance creates an unapproved candidate, never human acceptance.

## before-fixes/builtin-ai-qwen3.5_4b-fx-01-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-01-r1.json).

Expected decisions/actions: fx-01-e1: Keep offline access in the pilot. [agreed]; fx-01-e2: Test the offline checklist. [agreed]

User priorities: fx-01-p1: offline access; fx-01-p2: existing budget

Forbidden inferences: Inventing a budget amount; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting confirmed that the pilot project must protect offline access and remain within the existing budget. The group agreed to keep offline access in the pilot, and Mina was assigned to test the offline checklist, though no due date was set. (owner=None; deadline=None) | note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget.; transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot.; transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | unsupported: Only offline access was agreed in t1; n1 is a priority. The summary invents a confirmed budget commitment. | Pending |
| accepted-i1 / validated candidate | action / review_required: Mina will test the offline checklist. (owner=Mina; deadline=None) | transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |

Recall 2/2; priorities 1/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-01-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-01-r2.json).

Expected decisions/actions: fx-01-e1: Keep offline access in the pilot. [agreed]; fx-01-e2: Test the offline checklist. [agreed]

User priorities: fx-01-p1: offline access; fx-01-p2: existing budget

Forbidden inferences: Inventing a budget amount; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting concluded with a commitment to ensure the pilot protects offline access and remains within the existing budget, while assigning Mina the task of testing the offline checklist. (owner=None; deadline=None) | note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget.; transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | unsupported: Only offline access was agreed in t1; n1 is a priority. The summary invents a confirmed budget commitment. | Pending |
| accepted-i1 / validated candidate | decision / review_required: The group agreed to keep offline access in the pilot. (owner=None; deadline=None) | transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| accepted-i2 / validated candidate | action / review_required: Mina will test the offline checklist. (owner=Mina; deadline=None) | transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| accepted-i3 / validated candidate | highlight / notes_only: Priority constraints identified: the pilot must protect offline access and stay within the existing budget. (owner=None; deadline=None) | note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-02-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-02-r1.json).

Expected decisions/actions: fx-02-e1: Add keyboard navigation. [agreed]; fx-02-e2: Review consent wording before rollout. [agreed]

User priorities: fx-02-p1: accessibility and keyboard navigation; fx-02-p2: slow connections; fx-02-p3: unresolved consent wording

Forbidden inferences: Treating rollout as a calendar deadline; Inventing an assignee

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: Priority topics include accessibility, slow connections, and unresolved consent wording. Agreed to add keyboard navigation; consent wording requires review before rollout with no assigned owner or date set. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording.; transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation.; transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i1 / validated candidate | decision / review_required: Decision made to add keyboard navigation. (owner=None; deadline=None) | transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i2 / validated candidate | question / review_required: Consent wording requires review before rollout, but no owner or deadline has been assigned. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i3 / validated candidate | action / review_required: Review consent wording before rollout. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |

Recall 2/2; priorities 3/3. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-02-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-02-r2.json).

Expected decisions/actions: fx-02-e1: Add keyboard navigation. [agreed]; fx-02-e2: Review consent wording before rollout. [agreed]

User priorities: fx-02-p1: accessibility and keyboard navigation; fx-02-p2: slow connections; fx-02-p3: unresolved consent wording

Forbidden inferences: Treating rollout as a calendar deadline; Inventing an assignee

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | highlight / notes_only: Priority topics identified: accessibility, slow connections, and unresolved consent wording. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a0-i1 / raw | decision / review_required: Keyboard navigation was agreed to be added. (owner=None; deadline=None) | transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a0-i2 / raw | open_question / notes_only: Consent wording requires review before rollout; no person was assigned and no date was set. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i0 / validated candidate | summary / notes_only: Priority topics identified as accessibility, slow connections, and unresolved consent wording. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i1 / validated candidate | decision / review_required: Keyboard navigation was agreed to be added. (owner=None; deadline=None) | transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i2 / validated candidate | question / review_required: Consent wording requires review before rollout; no owner or date was assigned. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |

Recall 2/2; priorities 3/3. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-03-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-03-r1.json).

Expected decisions/actions: fx-03-e1: Measure latency first. [agreed]; fx-03-e2: Switch to Redis was recommended but not accepted. [proposed]

User priorities: fx-03-p1: advice versus acceptance; fx-03-p2: measure latency first

Forbidden inferences: Confirmed migration to Redis; Assigning work to the consultant

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The consultant recommended switching the cache to Redis, but this was explicitly stated as advice only and not accepted. The group agreed that latency must be measured first before any changes are made. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first.; transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only.; transcript:fx-03-t2: quoted `Lead: We have not accepted Redis. We agreed to measure latency first.`; source: Lead: We have not accepted Redis. We agreed to measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-03-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-03-r2.json).

Expected decisions/actions: fx-03-e1: Measure latency first. [agreed]; fx-03-e2: Switch to Redis was recommended but not accepted. [proposed]

User priorities: fx-03-p1: advice versus acceptance; fx-03-p2: measure latency first

Forbidden inferences: Confirmed migration to Redis; Assigning work to the consultant

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | highlight / notes_only: Priority: advice must remain separate from accepted changes; measure latency first. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| accepted-i1 / validated candidate | question / review_required: Recommendation to switch cache to Redis was provided as advice only. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| accepted-i2 / validated candidate | conflict / conflict: Redis acceptance was explicitly rejected; consensus is to measure latency first before acceptance. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first.; transcript:fx-03-t2: quoted `Lead: We have not accepted Redis. We agreed to measure latency first.`; source: Lead: We have not accepted Redis. We agreed to measure latency first. | unsupported: t2 says Redis was not accepted; it does not say explicitly rejected. Latency measurement is supported, but the rejection is fabricated. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-04-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](builtin-ai-qwen3.5_4b-fx-04-r1.json).

Expected decisions/actions: fx-04-e1: Expansion to Site B remains an unapproved proposal. [proposed]; fx-04-e2: Approve the safety review. [agreed]

User priorities: fx-04-p1: expansion proposal status; fx-04-p2: approved safety review

Forbidden inferences: Expansion approved; Inventing a resolution number, vote count or quorum

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: The meeting notes distinguish between a proposed expansion to Site B and an approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| c1-a0-i1 / raw | decision / uncertain: Expansion to Site B was proposed but not approved. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| c1-a0-i2 / raw | decision / notes_only: The safety review is approved. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved. There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| c1-a0-i3 / raw | risk / uncertain: No recorded vote count or quorum statement exists for the safety review approval. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved. There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| c1-a1-i0 / repair | decision / notes_only: Expansion to Site B was proposed but not approved. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| c1-a1-i1 / repair | decision / notes_only: Safety review is approved. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| c1-a1-i2 / repair | priority / notes_only: Distinguish the proposed expansion from the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |

Recall 0/2; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-04-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-04-r2.json).

Expected decisions/actions: fx-04-e1: Expansion to Site B remains an unapproved proposal. [proposed]; fx-04-e2: Approve the safety review. [agreed]

User priorities: fx-04-p1: expansion proposal status; fx-04-p2: approved safety review

Forbidden inferences: Expansion approved; Inventing a resolution number, vote count or quorum

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / notes_only: The meeting noted a priority to distinguish the proposed expansion from the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i1 / validated candidate | decision / review_required: Expansion to Site B was proposed but not approved. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i2 / validated candidate | decision / review_required: The safety review is approved, with no recorded vote count or quorum statement. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved. There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-05-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-05-r1.json).

Expected decisions/actions: fx-05-e1: Submit the inventory checklist. [agreed]

User priorities: fx-05-p1: inventory checklist; fx-05-p2: exact owner and date

Forbidden inferences: Changing the owner; Changing or converting the literal date

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | action / review_required: Ana Cruz to submit the inventory checklist by 20 October 2026. (owner=Ana Cruz; deadline=20 October 2026) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist.; transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| accepted-i1 / validated candidate | summary / review_required: The meeting concluded with an agreement that Ana Cruz will submit an inventory checklist by 20 October 2026, with a priority to retain the exact owner and literal due date. (owner=None; deadline=None) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist.; transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-05-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-05-r2.json).

Expected decisions/actions: fx-05-e1: Submit the inventory checklist. [agreed]

User priorities: fx-05-p1: inventory checklist; fx-05-p2: exact owner and date

Forbidden inferences: Changing the owner; Changing or converting the literal date

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | action / review_required: Ana Cruz will submit the inventory checklist by 20 October 2026. (owner=None; deadline=None) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist.; transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-06-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-06-r1.json).

Expected decisions/actions: fx-06-e1: Prepare the options memo. [agreed]

User priorities: fx-06-p1: office assignment; fx-06-p2: options memo

Forbidden inferences: Inventing a named staff member; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | action / review_required: The board assigned the Procurement Office to prepare the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i1 / validated candidate | summary / review_required: No deadline was specified for the assignment. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i2 / validated candidate | summary / notes_only: The assignment is to an office; no individual was named. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-06-r2

Outcome: rejected_generation; error: Structured output failed validation after one repair: Model output is not valid structured claims JSON. [Full evidence](builtin-ai-qwen3.5_4b-fx-06-r2.json).

Expected decisions/actions: fx-06-e1: Prepare the options memo. [agreed]

User priorities: fx-06-p1: office assignment; fx-06-p2: options memo

Forbidden inferences: Inventing a named staff member; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | assignment / notes_only: The Procurement Office was assigned to prepare the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a0-i1 / raw | summary / notes_only: No individual name was specified for the assignment; only the office was designated. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a0-i2 / raw | summary / notes_only: No deadline was specified for the task. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a1-i0 / repair | assignment / notes_only: The Procurement Office was assigned to prepare the options memo. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named.; transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a1-unparsed / unparsed response | None / None: {"claims":[{"kind":"assignment","text":"The Procurement Office was assigned to prepare the options memo.","section":"Assignments","status":"notes_only","sources":[{"source_id":"note:fx-06-n1","excerpt":"Priority: the assignment is to an office; no individual was named."},{"source_id":"transcript:fx-06-t1","excerpt":"The board assigns the Procurement Office to prepare the options memo."}],"owner":null,"deadline":null}]}" (owner=None; deadline=None) |  | unparseable: Raw assertion inspected; malformed response is retained and earns no delivered credit. t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |

Recall 0/1; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-07-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-07-r1.json).

Expected decisions/actions: fx-07-e1: Update the risk register. [agreed]

User priorities: fx-07-p1: risk register; fx-07-p2: missing owner and deadline

Forbidden inferences: Resolving I to a person or office; Inferring an owner from microphone/system provenance

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting concluded with an agreement to update the risk register, though no specific owner or deadline was assigned. A priority item identified is documenting this missing owner and due date. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `We agreed to update the risk register. Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i1 / validated candidate | action / notes_only: Document the missing owner and due date for the risk register. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: A participant offered to help update the risk register, but their identity was not recorded. (owner=None; deadline=None) | transcript:fx-07-t1: quoted `Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-07-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-07-r2.json).

Expected decisions/actions: fx-07-e1: Update the risk register. [agreed]

User priorities: fx-07-p1: risk register; fx-07-p2: missing owner and deadline

Forbidden inferences: Resolving I to a person or office; Inferring an owner from microphone/system provenance

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting agreed to update the risk register, with a proposal to document missing owner and due date details. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `We agreed to update the risk register. Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: A participant offered assistance to update the risk register but did not provide their name. (owner=None; deadline=None) | transcript:fx-07-t1: quoted `Someone said, I can help.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: No specific owner or deadline was assigned for updating the risk register. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i3 / validated candidate | action / notes_only: Document the missing owner and due date for the risk register update. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-08-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-08-r1.json).

Expected decisions/actions: fx-08-e1: Possibly review the prototype next Friday if data arrive. [proposed]

User priorities: fx-08-p1: tentative prototype review; fx-08-p2: unresolved relative date

Forbidden inferences: A confirmed review; Converting next Friday to a calendar date; Treating the relative date as a binding deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting concluded without agreement on reviewing the prototype next Friday due to pending data arrival, and the tentative schedule with its unresolved date anchor remains as the priority. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor.; transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-08-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-08-r2.json).

Expected decisions/actions: fx-08-e1: Possibly review the prototype next Friday if data arrive. [proposed]

User priorities: fx-08-p1: tentative prototype review; fx-08-p2: unresolved relative date

Forbidden inferences: A confirmed review; Converting next Friday to a calendar date; Treating the relative date as a binding deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: The meeting discussed reviewing a prototype next Friday pending data arrival, though this was not agreed upon. A priority was set to retain the tentative schedule with its unresolved date anchor. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor.; transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| c1-a0-i1 / raw | highlight / notes_only: Proposal to review the prototype next Friday pending data arrival was explicitly noted as not agreed upon. (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| c1-a0-i2 / raw | decision / notes_only: The tentative schedule and its unresolved date anchor were prioritized for retention. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| c1-a0-i3 / raw | action / notes_only: None explicitly assigned with owner or deadline. (owner=None; deadline=None) |  | qualified: No assignment is explicit in t1; raw no-source notes_only claim is rejected and the repaired output preserves tentativeness. | Pending |
| c1-a0-i4 / raw | conflict / notes_only: No explicit conflicts identified beyond the lack of agreement on the prototype review timeline. (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| accepted-i0 / validated candidate | highlight / notes_only: The tentative schedule and its unresolved date anchor are identified as a priority to retain. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| accepted-i1 / validated candidate | conflict / conflict: A proposal to review the prototype next Friday pending data arrival was not agreed upon. (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-09-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-09-r1.json).

Expected decisions/actions: fx-09-e1: Reject Option A. [rejected]; fx-09-e2: Use Option B. [agreed]

User priorities: fx-09-p1: rejection of Option A and offline-access reason; fx-09-p2: selection of Option B

Forbidden inferences: Option A approved; Option A action assigned

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | decision / review_required: Option A was rejected because it removes offline access; Option B was selected. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B.; transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access.; transcript:fx-09-t2: quoted `We agreed to use Option B.`; source: We agreed to use Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i1 / validated candidate | summary / notes_only: The meeting concluded with a priority to maintain the decision rejecting Option A and adopting Option B due to concerns over offline access removal. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | unsupported: The offline-access reason is absent from cited n1; actual t1 supports it, but this row cites only n1. No invented decision; citation does not support the whole row. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-09-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-09-r2.json).

Expected decisions/actions: fx-09-e1: Reject Option A. [rejected]; fx-09-e2: Use Option B. [agreed]

User priorities: fx-09-p1: rejection of Option A and offline-access reason; fx-09-p2: selection of Option B

Forbidden inferences: Option A approved; Option A action assigned

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The board rejected Option A because it removes offline access and agreed to use Option B. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B.; transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access.; transcript:fx-09-t2: quoted `We agreed to use Option B.`; source: We agreed to use Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-10-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-10-r1.json).

Expected decisions/actions: fx-10-e1: Launch approval is disputed: notes say approved, transcript says deferred. [conflict]

User priorities: fx-10-p1: approval conflict

Forbidden inferences: Unqualified approved launch; Silently discarding either source

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | conflict / conflict: There is a conflict between source notes claiming the launch was approved and the transcript stating it was discussed but not approved, with the decision deferred. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict. My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| accepted-i1 / validated candidate | summary / review_required: The meeting addressed the launch topic, resulting in conflicting records: one note claims approval while the transcript indicates the decision was deferred without approval. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict. My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |

Recall 1/1; priorities 1/1. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-10-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-10-r2.json).

Expected decisions/actions: fx-10-e1: Launch approval is disputed: notes say approved, transcript says deferred. [conflict]

User priorities: fx-10-p1: approval conflict

Forbidden inferences: Unqualified approved launch; Silently discarding either source

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | conflict / conflict: There is a conflict between notes stating the launch was approved and transcript stating it was discussed but not approved, with the decision deferred. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict. My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |

Recall 1/1; priorities 1/1. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-11-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-11-r1.json).

Expected decisions/actions: fx-11-e1: Approve Package Delta, the offline-access checklist. [agreed]; fx-11-e2: Luis Reyes will prepare Package Delta, the offline-access checklist. [agreed]

User priorities: fx-11-p1: Package Delta definition and final assignment; fx-11-p2: offline-access constraint

Forbidden inferences: Final agreement was only a proposal; Dropping the early definition; Assigning a deadline; A calibration log decision

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting established that Package Delta is a proposal representing the offline-access checklist, with a priority to connect this final assignment to its early definition while retaining the offline-access constraint. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.; transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i1 / validated candidate | summary / review_required: The meeting segment contained no relevant information; it consisted solely of repeated background calibration log entries stating 'Unrelated sensor baseline observation; no decision or action.' (owner=None; deadline=None) | transcript:fx-11-t2: quoted `Background calibration log: Unrelated sensor baseline observation; no decision or action.`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON] | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: No decisions, actions, owners, deadlines, or specific topics were discussed or recorded in this segment. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `no decision or action`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON] | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i3 / validated candidate | highlight / review_required: The transcript provided only contains background calibration logs with no actionable content. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `Unrelated sensor baseline observation; no decision or action`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON] | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i4 / validated candidate | decision / review_required: Package Delta is approved as defined at the start. (owner=None; deadline=None) | transcript:fx-11-t3: quoted `We now approve Package Delta as defined at the start.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i5 / validated candidate | action / review_required: Luis Reyes will prepare Package Delta. (owner=Luis Reyes; deadline=None) | transcript:fx-11-t3: quoted `Luis Reyes will prepare it;`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i6 / validated candidate | highlight / review_required: No deadline was set for the preparation of Package Delta. (owner=None; deadline=None) | transcript:fx-11-t3: quoted `no deadline was set.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i7 / validated candidate | summary / review_required: The meeting concluded with the approval of Package Delta and the assignment to Luis Reyes to prepare it, with no specific deadline established. Previous sensor baseline observations were noted as unrelated to decisions or actions. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `no decision or action. Unrelated sensor baseline observation`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON]; transcript:fx-11-t3: quoted `We now approve Package Delta as defined at the start.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-11-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-11-r2.json).

Expected decisions/actions: fx-11-e1: Approve Package Delta, the offline-access checklist. [agreed]; fx-11-e2: Luis Reyes will prepare Package Delta, the offline-access checklist. [agreed]

User priorities: fx-11-p1: Package Delta definition and final assignment; fx-11-p2: offline-access constraint

Forbidden inferences: Final agreement was only a proposal; Dropping the early definition; Assigning a deadline; A calibration log decision

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | highlight / notes_only: The priority is to connect the final assignment to the early definition of Package Delta while retaining the offline-access constraint. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: Package Delta is defined as the offline-access checklist and is currently only a proposal. (owner=None; deadline=None) | transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: The initial proposal is called currently only a proposal despite final t3 approval. Missing temporal qualification leaves inconsistent current status. | Pending |
| accepted-i2 / validated candidate | summary / review_required: The meeting segment contained no decisions, actions, or substantive discussion topics; it consisted solely of repeated unrelated sensor baseline observations. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `Background calibration log: Unrelated sensor baseline observation; no decision or action.`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON] | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i3 / validated candidate | highlight / review_required: The transcript explicitly states that the content was 'Unrelated sensor baseline observation' and 'no decision or action'. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `Unrelated sensor baseline observation; no decision or action.`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON] | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i4 / validated candidate | decision / review_required: Package Delta is approved as defined at the start. (owner=None; deadline=None) | transcript:fx-11-t3: quoted `We now approve Package Delta as defined at the start.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i5 / validated candidate | action / review_required: Luis Reyes will prepare Package Delta. (owner=Luis Reyes; deadline=None) | transcript:fx-11-t3: quoted `Luis Reyes will prepare it; no deadline was set.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i6 / validated candidate | highlight / review_required: No decisions or actions were taken regarding unrelated sensor baseline observations. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `; no decision or action. Unrelated sensor baseline observation;`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON] | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-12-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-12-r1.json).

Expected decisions/actions: fx-12-e1: Keep offline access. [agreed]; fx-12-e2: Someone might check 安全確認 next week; name and acceptance are uncertain. [proposed]

User priorities: fx-12-p1: offline access; fx-12-p2: uncertain name and 安全確認; fx-12-p3: Café résumé and Δ notation

Forbidden inferences: Confirmed Mara or Nara as owner; Confirmed safety-check assignment; Calendar date inferred from next week; Dropping Unicode content

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: Priority to retain offline access; Japanese label is '安全確認'. No names were inferred. Agreement reached on retaining offline access. Uncertainty remains regarding a name (possibly Mara or Nara) checking '安全確認' next week, as it was not yet confirmed and no agreement was made on that specific point. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ.; transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access.; transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a0-unparsed / unparsed response | None / None: {"claims":[{"kind":"summary","text":"Priority to retain offline access; Japanese label is '安全確認'. No names were inferred. Agreement reached on retaining offline access. Uncertainty remains regarding a name (possibly Mara or Nara) checking '安全確認' next week, as it was not yet confirmed and no agreement was made on that specific point.","section":"Highlights","status":"notes_only","sources":[{"source_id":"note:fx-12-n1","excerpt":"Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ."},{"source_id":"transcript:fx-12-t1","excerpt":"Napagkasunduan: panatilihin ang offline access."},{"source_id":"transcript:fx-12-t2","excerpt":"ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan."}],"owner":null,"deadline":null}]}" (owner=None; deadline=None) |  | unparseable: Raw assertion inspected; malformed response is retained and earns no delivered credit. t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i0 / validated candidate | highlight / review_required: Priority topic: secure offline access (panatilihin ang offline access). (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ.; transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i1 / validated candidate | highlight / notes_only: Japanese label for offline access is '安全確認' (Safety Confirmation). (owner=None; deadline=None) | note:fx-12-n1: quoted `Japanese label: 安全確認`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | unsupported: n1 gives a Japanese label separately. It does not say this is the label for offline access. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: Potential candidate name '[Mara? Nara?] might check Safety Confirmation next week', but this remains uncertain and unconfirmed. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i3 / validated candidate | conflict / conflict: There is a conflict between the priority to secure offline access and the lack of agreement on the specific name/person to verify. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ.; transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | unsupported: Offline-access priority and uncertain identity are compatible. The stated conflict is invented. | Pending |

Recall 1/2; priorities 2/3. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/builtin-ai-qwen3.5_4b-fx-12-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-12-r2.json).

Expected decisions/actions: fx-12-e1: Keep offline access. [agreed]; fx-12-e2: Someone might check 安全確認 next week; name and acceptance are uncertain. [proposed]

User priorities: fx-12-p1: offline access; fx-12-p2: uncertain name and 安全確認; fx-12-p3: Café résumé and Δ notation

Forbidden inferences: Confirmed Mara or Nara as owner; Confirmed safety-check assignment; Calendar date inferred from next week; Dropping Unicode content

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | highlight / notes_only: Priority: maintain offline access; do not speculate on names. Japanese label: Safety Confirmation. Café résumé — Δ. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: Agreed upon: maintain offline access. (owner=None; deadline=None) | transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i2 / validated candidate | conflict / conflict: ASR uncertain regarding potential check of Safety Confirmation next week; no agreement reached on this matter yet. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |

Recall 2/2; priorities 3/3. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-01-r1

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-01-r1.json).

Expected decisions/actions: fx-01-e1: Keep offline access in the pilot. [agreed]; fx-01-e2: Test the offline checklist. [agreed]

User priorities: fx-01-p1: offline access; fx-01-p2: existing budget

Forbidden inferences: Inventing a budget amount; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/2; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-01-r2

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-01-r2.json).

Expected decisions/actions: fx-01-e1: Keep offline access in the pilot. [agreed]; fx-01-e2: Test the offline checklist. [agreed]

User priorities: fx-01-p1: offline access; fx-01-p2: existing budget

Forbidden inferences: Inventing a budget amount; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/2; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-02-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-02-r1.json).

Expected decisions/actions: fx-02-e1: Add keyboard navigation. [agreed]; fx-02-e2: Review consent wording before rollout. [agreed]

User priorities: fx-02-p1: accessibility and keyboard navigation; fx-02-p2: slow connections; fx-02-p3: unresolved consent wording

Forbidden inferences: Treating rollout as a calendar deadline; Inventing an assignee

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | priority / notes_only: Accessibility, slow connections, and unresolved consent wording are priority topics. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i0 / validated candidate | highlight / notes_only: The top priority topics are accessibility, slow connections, and unresolved consent wording. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i1 / validated candidate | action / missing_source: Review consent wording before rollout (owner=None; deadline=None) |  | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: Nobody was assigned to review consent wording before rollout. No date was set. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i3 / validated candidate | decision / review_required: Consent wording review status: undecided (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | unsupported: Unassigned owner/date does not establish undecided consent review; t2 requests review. The uncertainty label is visible, but the cited source does not establish this status. | Pending |
| accepted-i4 / validated candidate | question / review_required: Who will review the consent wording? (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i5 / validated candidate | question / review_required: What is the deadline to review the consent wording? (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |

Recall 1/2; priorities 3/3. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-02-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-02-r2.json).

Expected decisions/actions: fx-02-e1: Add keyboard navigation. [agreed]; fx-02-e2: Review consent wording before rollout. [agreed]

User priorities: fx-02-p1: accessibility and keyboard navigation; fx-02-p2: slow connections; fx-02-p3: unresolved consent wording

Forbidden inferences: Treating rollout as a calendar deadline; Inventing an assignee

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c2-a0-unparsed / unparsed response | None / None: {<br>  "coverage": "committed_available",<br>  "instructions": "Prioritize the topics marked as priorities in original notes. Keep proposals, rejections, uncertainty and conflicts explicit. Do not infer names from audio channels.",<br>  "summary_language": "en",<br>  "template": {<br>    "description": "Source-linked discussion priorities, outcomes and open questions.",<br>    "name": "Highlights",<br>    "sections": [<br>      {<br>        "format": "list",<br>        "instruction": "Retain important user-priority topics and supporting source excerpts. Preserve uncertainty and disagreements.",<br>        "title": "Highlights"<br>      },<br>      {<br>        "format": "list",<br>        "instruction": "Separate accepted outcomes from proposals, rejected options and unanswered questions. Do not invent decisions.",<br>        "title": "Outcomes and Open Questions"<br>      },<br>      {<br>        "format": "list",<br>        "instruction": "Include only source-supported actions. Owner and deadline remain unspecified unless explicitly stated.",<br>        "title": "Actions"<br>      }<br>    ]<br>  },<br>  "data_sources_provided": [<br>    {<br>      "source_id": "transcript:fx-02-t1",<br>      "kind": "transcript",<br>      "text": "We agreed to add keyboard navigation.",<br>      "fingerprint": "cf5d12af06240d94:37",<br>      "audio_start_time": 0.0,<br>      "audio_end_time": 9.0,<br>      "audio_source": "unknown"<br>    }<br>  ],<br>  "extraction_result": {<br>    "claims": []<br>  },<br>  "system_message": "No valid claims could be extracted because the provided input lacks the full structured JSON schema (e.g., missing 'summary' or 'notes' section) and contains insufficient semantic detail to populate a claim text reliably without inventing details. The instruction 'Do not infer names' further prohibits creating an action item owner from the generic phrase 'We agreed'. Without explicit assignment of an owner name in the source excerpt, the 'owner' field must remain null. Similarly, no deadline is found."<br>} (owner=None; deadline=None) |  | unsupported: Rejected response falsely says no valid claims can be extracted; actual t1 explicitly agrees keyboard navigation. Repair extracts that decision. | Pending |
| accepted-i0 / validated candidate | highlight / notes_only: Priority topics identified: accessibility, slow connections, and unresolved consent wording. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i1 / validated candidate | question / notes_only: How to resolve unresolved consent wording? (owner=None; deadline=None) | note:fx-02-n1: quoted `unresolved consent wording`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i2 / validated candidate | action / notes_only: Address slow connections. (owner=None; deadline=None) | note:fx-02-n1: quoted `slow connections`; source: Priority: accessibility, slow connections, and unresolved consent wording. | unsupported: n1 states slow connections as a user priority. Address slow connections is a notes-only proposed response, not an agreed action. Source scope is disclosed; keep human review. | Pending |
| accepted-i3 / validated candidate | decision / review_required: Add keyboard navigation. (owner=None; deadline=None) | transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i4 / validated candidate | decision / conflict: No decision made on consent wording review rollout; nobody was assigned and no date was set. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | unsupported: t2 requests review before rollout; missing owner/date does not prove that no decision was made. | Pending |

Recall 1/2; priorities 3/3. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-03-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](ollama-qwen3.5_9b-mlx-fx-03-r1.json).

Expected decisions/actions: fx-03-e1: Measure latency first. [agreed]; fx-03-e2: Switch to Redis was recommended but not accepted. [proposed]

User priorities: fx-03-p1: advice versus acceptance; fx-03-p2: measure latency first

Forbidden inferences: Confirmed migration to Redis; Assigning work to the consultant

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | highlight / notes_only: Advice must remain separate from accepted changes. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i1 / raw | action / notes_only: Measure latency first. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i2 / raw | highlight / notes_only: Adhere to the priority that advice and accepted changes are distinct categories. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c2-a0-i0 / raw | advice / uncertain: Switch the cache to Redis. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c2-a1-i0 / repair | advice / notes_only: Switch the cache to Redis. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |

Recall 0/2; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-03-r2

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](ollama-qwen3.5_9b-mlx-fx-03-r2.json).

Expected decisions/actions: fx-03-e1: Measure latency first. [agreed]; fx-03-e2: Switch to Redis was recommended but not accepted. [proposed]

User priorities: fx-03-p1: advice versus acceptance; fx-03-p2: measure latency first

Forbidden inferences: Confirmed migration to Redis; Assigning work to the consultant

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | highlight / notes_only: Advice must remain separate from accepted changes. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i1 / raw | highlight / notes_only: Measure latency first. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c2-a0-i0 / raw | advice / uncertain: Switch the cache to Redis (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c2-a1-i0 / repair | advice / notes_only: Switch the cache to Redis. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |

Recall 0/2; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-04-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-04-r1.json).

Expected decisions/actions: fx-04-e1: Expansion to Site B remains an unapproved proposal. [proposed]; fx-04-e2: Approve the safety review. [agreed]

User priorities: fx-04-p1: expansion proposal status; fx-04-p2: approved safety review

Forbidden inferences: Expansion approved; Inventing a resolution number, vote count or quorum

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | question / notes_only: Distinguish the proposed expansion from the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i1 / validated candidate | decision / review_required: Expansion to Site B was proposed; we did not approve it. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i2 / validated candidate | decision / review_required: Expansion to Site B was not approved. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i3 / validated candidate | summary / review_required: The safety review is approved. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved. There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i4 / validated candidate | decision / conflict: Safety review approval noted without recorded vote count or quorum statement. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved. There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i5 / validated candidate | question / review_required: Is the absence of a recorded vote count or quorum statement an issue for the validity of the approval? (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved. There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-04-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-04-r2.json).

Expected decisions/actions: fx-04-e1: Expansion to Site B remains an unapproved proposal. [proposed]; fx-04-e2: Approve the safety review. [agreed]

User priorities: fx-04-p1: expansion proposal status; fx-04-p2: approved safety review

Forbidden inferences: Expansion approved; Inventing a resolution number, vote count or quorum

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | question / notes_only: Distinguish the proposed expansion from the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i1 / validated candidate | decision / review_required: Expansion to Site B was proposed but not approved. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i2 / validated candidate | decision / review_required: Proposal for Expansion to Site B was rejected. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | unsupported: t1 says not approved, not rejected. Explicit uncertainty remains visible, but the cited passage does not establish rejection. | Pending |
| accepted-i3 / validated candidate | decision / review_required: The safety review is approved. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `The safety review is approved.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-05-r1

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-05-r1.json).

Expected decisions/actions: fx-05-e1: Submit the inventory checklist. [agreed]

User priorities: fx-05-p1: inventory checklist; fx-05-p2: exact owner and date

Forbidden inferences: Changing the owner; Changing or converting the literal date

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-05-r2

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-05-r2.json).

Expected decisions/actions: fx-05-e1: Submit the inventory checklist. [agreed]

User priorities: fx-05-p1: inventory checklist; fx-05-p2: exact owner and date

Forbidden inferences: Changing the owner; Changing or converting the literal date

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-06-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-06-r1.json).

Expected decisions/actions: fx-06-e1: Prepare the options memo. [agreed]

User priorities: fx-06-p1: office assignment; fx-06-p2: options memo

Forbidden inferences: Inventing a named staff member; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | decision / review_required: The board assigns the Procurement Office to prepare the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i1 / validated candidate | question / review_required: No deadline was specified for preparing the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-06-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-06-r2.json).

Expected decisions/actions: fx-06-e1: Prepare the options memo. [agreed]

User priorities: fx-06-p1: office assignment; fx-06-p2: options memo

Forbidden inferences: Inventing a named staff member; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | decision / review_required: The board assigns the Procurement Office to prepare the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo. No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: No deadline was specified for preparation of the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo. No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |

Recall 1/1; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-07-r1

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-07-r1.json).

Expected decisions/actions: fx-07-e1: Update the risk register. [agreed]

User priorities: fx-07-p1: risk register; fx-07-p2: missing owner and deadline

Forbidden inferences: Resolving I to a person or office; Inferring an owner from microphone/system provenance

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-07-r2

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-07-r2.json).

Expected decisions/actions: fx-07-e1: Update the risk register. [agreed]

User priorities: fx-07-p1: risk register; fx-07-p2: missing owner and deadline

Forbidden inferences: Resolving I to a person or office; Inferring an owner from microphone/system provenance

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-08-r1

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-08-r1.json).

Expected decisions/actions: fx-08-e1: Possibly review the prototype next Friday if data arrive. [proposed]

User priorities: fx-08-p1: tentative prototype review; fx-08-p2: unresolved relative date

Forbidden inferences: A confirmed review; Converting next Friday to a calendar date; Treating the relative date as a binding deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-08-r2

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-08-r2.json).

Expected decisions/actions: fx-08-e1: Possibly review the prototype next Friday if data arrive. [proposed]

User priorities: fx-08-p1: tentative prototype review; fx-08-p2: unresolved relative date

Forbidden inferences: A confirmed review; Converting next Friday to a calendar date; Treating the relative date as a binding deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-09-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-09-r1.json).

Expected decisions/actions: fx-09-e1: Reject Option A. [rejected]; fx-09-e2: Use Option B. [agreed]

User priorities: fx-09-p1: rejection of Option A and offline-access reason; fx-09-p2: selection of Option B

Forbidden inferences: Option A approved; Option A action assigned

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | decision / notes_only: Rejection of Option A and selection of Option B is to be preserved as a priority. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i1 / validated candidate | decision / review_required: The board rejected Option A. (owner=None; deadline=None) | transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i2 / validated candidate | decision / review_required: The group agreed to use Option B. (owner=None; deadline=None) | transcript:fx-09-t2: quoted `We agreed to use Option B.`; source: We agreed to use Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |

Recall 2/2; priorities 1/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-09-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-09-r2.json).

Expected decisions/actions: fx-09-e1: Reject Option A. [rejected]; fx-09-e2: Use Option B. [agreed]

User priorities: fx-09-p1: rejection of Option A and offline-access reason; fx-09-p2: selection of Option B

Forbidden inferences: Option A approved; Option A action assigned

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | decision / notes_only: Preserve the rejection of Option A and the selection of Option B. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i1 / validated candidate | decision / conflict: The Board rejected Option A. (owner=None; deadline=None) | transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: Reason for rejecting Option A: it removes offline access. (owner=None; deadline=None) | transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i3 / validated candidate | decision / review_required: The group agreed to use Option B. (owner=None; deadline=None) | transcript:fx-09-t2: quoted `We agreed to use Option B.`; source: We agreed to use Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |

Recall 2/2; priorities 2/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-10-r1

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-10-r1.json).

Expected decisions/actions: fx-10-e1: Launch approval is disputed: notes say approved, transcript says deferred. [conflict]

User priorities: fx-10-p1: approval conflict

Forbidden inferences: Unqualified approved launch; Silently discarding either source

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/1. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-10-r2

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-10-r2.json).

Expected decisions/actions: fx-10-e1: Launch approval is disputed: notes say approved, transcript says deferred. [conflict]

User priorities: fx-10-p1: approval conflict

Forbidden inferences: Unqualified approved launch; Silently discarding either source

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/1. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-11-r1

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-11-r1.json).

Expected decisions/actions: fx-11-e1: Approve Package Delta, the offline-access checklist. [agreed]; fx-11-e2: Luis Reyes will prepare Package Delta, the offline-access checklist. [agreed]

User priorities: fx-11-p1: Package Delta definition and final assignment; fx-11-p2: offline-access constraint

Forbidden inferences: Final agreement was only a proposal; Dropping the early definition; Assigning a deadline; A calibration log decision

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/2; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-11-r2

Outcome: rejected_generation; error: Template and instructions leave insufficient source context. [Full evidence](ollama-qwen3.5_9b-mlx-fx-11-r2.json).

Expected decisions/actions: fx-11-e1: Approve Package Delta, the offline-access checklist. [agreed]; fx-11-e2: Luis Reyes will prepare Package Delta, the offline-access checklist. [agreed]

User priorities: fx-11-p1: Package Delta definition and final assignment; fx-11-p2: offline-access constraint

Forbidden inferences: Final agreement was only a proposal; Dropping the early definition; Assigning a deadline; A calibration log decision

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/2; priorities 0/2. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-12-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-12-r1.json).

Expected decisions/actions: fx-12-e1: Keep offline access. [agreed]; fx-12-e2: Someone might check 安全確認 next week; name and acceptance are uncertain. [proposed]

User priorities: fx-12-p1: offline access; fx-12-p2: uncertain name and 安全確認; fx-12-p3: Café résumé and Δ notation

Forbidden inferences: Confirmed Mara or Nara as owner; Confirmed safety-check assignment; Calendar date inferred from next week; Dropping Unicode content

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c3-a0-i0 / raw | uncertain_status / uncertain: Unclear whether the party will confirm safety status (安全確認) next week; confirmed as not yet determined and no mutual agreement was reached. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i0 / validated candidate | summary / notes_only: Priority is to address offline access; do not guess the name. Japanese label: Safety Confirmation. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i1 / validated candidate | decision / review_required: Agreement to maintain offline access. (owner=None; deadline=None) | transcript:fx-12-t1: quoted `panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: Identity verification for Nara/Mara is pending next week but remains uncertain and not yet agreed upon. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | unsupported: t2 concerns a tentative safety check, not identity verification. The repaired row changes the action; uncertainty does not repair that meaning error. | Pending |

Recall 1/2; priorities 1/3. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

## before-fixes/ollama-qwen3.5_9b-mlx-fx-12-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-12-r2.json).

Expected decisions/actions: fx-12-e1: Keep offline access. [agreed]; fx-12-e2: Someone might check 安全確認 next week; name and acceptance are uncertain. [proposed]

User priorities: fx-12-p1: offline access; fx-12-p2: uncertain name and 安全確認; fx-12-p3: Café résumé and Δ notation

Forbidden inferences: Confirmed Mara or Nara as owner; Confirmed safety-check assignment; Calendar date inferred from next week; Dropping Unicode content

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | highlight / notes_only: Priority is to translate offline access; do not guess the name. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | unsupported: panatilihin means retain/maintain, not translate. The row changes the user priority. | Pending |
| accepted-i1 / validated candidate | highlight / notes_only: Japanese label for résumé is '安全確認'. (owner=None; deadline=None) | note:fx-12-n1: quoted `Café résumé — Δ.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | unsupported: The cited Café résumé text does not label résumé as 安全確認; no source supports that relationship. | Pending |
| accepted-i2 / validated candidate | decision / review_required: Agreement to maintain offline access. (owner=None; deadline=None) | transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i3 / validated candidate | question / review_required: Whether Mara or Nara will check safety confirmation next week is uncertain. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |

Recall 2/2; priorities 2/3. Content recall can appear in a summary or multiple claims. Rejected generations receive zero credit. Human acceptance pending.

