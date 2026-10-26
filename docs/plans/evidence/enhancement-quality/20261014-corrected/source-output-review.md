# Enhancement source/output review sheet

Semantic assessments below are **Codex review**. Human acceptance is pending for every run. Read complete source labels and raw response in the linked JSON when a row is ambiguous. Structural acceptance creates an unapproved candidate, never human acceptance.

## corrected/builtin-ai-qwen3.5_4b-fx-01-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-01-r1.json).

Expected decisions/actions: fx-01-e1: Keep offline access in the pilot. [agreed]; fx-01-e2: Test the offline checklist. [agreed]

User priorities: fx-01-p1: offline access; fx-01-p2: existing budget

Forbidden inferences: Inventing a budget amount; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: The meeting concluded that the pilot must prioritize protecting offline access while remaining within the existing budget, with Mina assigned to test the offline checklist. (owner=None; deadline=None) | note:fx-01-n1: quoted `"Priority: the pilot must protect offline access and stay within the existing budget."`; source: Priority: the pilot must protect offline access and stay within the existing budget.; transcript:fx-01-t2: quoted `"Mina will test the offline checklist. No due date was set."`; source: Mina will test the offline checklist. No due date was set. | unsupported: Raw summary also converts the note budget priority to a meeting conclusion; quoted excerpts fail exact matching. | Pending |
| c1-a0-i1 / raw | decision / notes_only: The group agreed to keep offline access in the pilot. (owner=None; deadline=None) | transcript:fx-01-t1: quoted `"The group agreed to keep offline access in the pilot."`; source: The group agreed to keep offline access in the pilot. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| c1-a0-i2 / raw | action / notes_only: Mina will test the offline checklist. (owner=Mina; deadline=None) | transcript:fx-01-t2: quoted `"Mina will test the offline checklist. No due date was set."`; source: Mina will test the offline checklist. No due date was set. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| c1-a0-i3 / raw | highlight / notes_only: The pilot must protect offline access and stay within the existing budget. (owner=None; deadline=None) | note:fx-01-n1: quoted `"Priority: the pilot must protect offline access and stay within the existing budget."`; source: Priority: the pilot must protect offline access and stay within the existing budget. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| accepted-i0 / validated candidate | summary / review_required: The meeting concluded that the pilot must prioritize protecting offline access while remaining within the existing budget. The group formally agreed to retain offline access in the pilot phase. Additionally, Mina was assigned to test the offline checklist, though no specific due date was established for this task. (owner=None; deadline=None) | note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget.; transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot.; transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | unsupported: n1 is a user budget priority, not a confirmed meeting conclusion/commitment. t1 agrees only offline access. | Pending |
| accepted-i1 / validated candidate | decision / review_required: The group agreed to keep offline access in the pilot. (owner=None; deadline=None) | transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| accepted-i2 / validated candidate | action / review_required: Mina will test the offline checklist. (owner=Mina; deadline=None) | transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| accepted-i3 / validated candidate | highlight / notes_only: Priority identified as protecting offline access and staying within the existing budget. (owner=None; deadline=None) | note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-01-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-01-r2.json).

Expected decisions/actions: fx-01-e1: Keep offline access in the pilot. [agreed]; fx-01-e2: Test the offline checklist. [agreed]

User priorities: fx-01-p1: offline access; fx-01-p2: existing budget

Forbidden inferences: Inventing a budget amount; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: The meeting prioritized protecting offline access for the pilot while staying within the existing budget. The group explicitly agreed to maintain offline access, and Mina was assigned to test the offline checklist, though no due date was set. (owner=None; deadline=None) | note:fx-01-n1: quoted `"Priority: the pilot must protect offline access and stay within the existing budget."`; source: Priority: the pilot must protect offline access and stay within the existing budget.; transcript:fx-01-t1: quoted `"The group agreed to keep offline access in the pilot."`; source: The group agreed to keep offline access in the pilot.; transcript:fx-01-t2: quoted `"Mina will test the offline checklist. No due date was set."`; source: Mina will test the offline checklist. No due date was set. | qualified: n1 supplies the user budget priority; offline agreement and Mina are explicitly supported by t1/t2. Meeting-priority attribution should be reviewed; no budget amount or approval is asserted. | Pending |
| c1-a0-i1 / raw | decision / notes_only: The group agreed to keep offline access in the pilot. (owner=None; deadline=None) | transcript:fx-01-t1: quoted `"The group agreed to keep offline access in the pilot."`; source: The group agreed to keep offline access in the pilot. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| c1-a0-i2 / raw | action / notes_only: Mina will test the offline checklist. (owner=Mina; deadline=None) | transcript:fx-01-t2: quoted `"Mina will test the offline checklist. No due date was set."`; source: Mina will test the offline checklist. No due date was set. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| accepted-i0 / validated candidate | summary / review_required: The meeting prioritized ensuring the pilot protects offline access while staying within budget. The group agreed to keep offline access in the pilot, and Mina was assigned to test the offline checklist without a set due date. (owner=None; deadline=None) | note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget.; transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot.; transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | qualified: n1 supplies the user budget priority; offline agreement and Mina are explicitly supported by t1/t2. Meeting-priority attribution should be reviewed; no budget amount or approval is asserted. | Pending |
| accepted-i1 / validated candidate | action / review_required: Mina will test the offline checklist. (owner=Mina; deadline=None) | transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-02-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-02-r1.json).

Expected decisions/actions: fx-02-e1: Add keyboard navigation. [agreed]; fx-02-e2: Review consent wording before rollout. [agreed]

User priorities: fx-02-p1: accessibility and keyboard navigation; fx-02-p2: slow connections; fx-02-p3: unresolved consent wording

Forbidden inferences: Treating rollout as a calendar deadline; Inventing an assignee

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: Priority topics include accessibility, slow connections, and unresolved consent wording. An agreement was reached to add keyboard navigation. Consent wording requires review before rollout without an assigned owner or date. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording.; transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation.; transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a0-i1 / raw | outcomes_and_questions / notes_only: Decision made to add keyboard navigation. Consent wording is pending review with no assignment or deadline. (owner=None; deadline=None) | transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation.; transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a0-i2 / raw | actions / notes_only: None explicitly stated with owner and deadline. (owner=None; deadline=None) |  | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i0 / validated candidate | summary / review_required: Priority topics identified are accessibility, slow connections, and unresolved consent wording. An agreement was made to add keyboard navigation. Consent wording is pending review before rollout with no assigned owner or date. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording.; transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation.; transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i1 / validated candidate | action / review_required: Review consent wording before rollout. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |

Recall 2/2; priorities 3/3. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-02-r2

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](builtin-ai-qwen3.5_4b-fx-02-r2.json).

Expected decisions/actions: fx-02-e1: Add keyboard navigation. [agreed]; fx-02-e2: Review consent wording before rollout. [agreed]

User priorities: fx-02-p1: accessibility and keyboard navigation; fx-02-p2: slow connections; fx-02-p3: unresolved consent wording

Forbidden inferences: Treating rollout as a calendar deadline; Inventing an assignee

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | highlight / notes_only: Priority topics identified: accessibility, slow connections, and unresolved consent wording. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a0-i1 / raw | decision / notes_only: Keyboard navigation was agreed to be added. (owner=None; deadline=None) | transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a0-i2 / raw | open_question / notes_only: Consent wording requires review before rollout; no owner assigned and no date set. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a0-i3 / raw | action / missing_source: No actions extracted as none were explicitly stated with an owner or deadline in the source text. (owner=None; deadline=None) |  | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a1-i0 / repair | highlight / notes_only: Priority topics identified: accessibility, slow connections, and unresolved consent wording. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c1-a1-i1 / repair | outcome / notes_only: Added keyboard navigation. (owner=None; deadline=None) | transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation. | unsupported: Agreed to add keyboard navigation is not evidence that it was already added. | Pending |
| c1-a1-i2 / repair | open_question / notes_only: Review consent wording before rollout; nobody was assigned and no date was set. (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |

Recall 0/2; priorities 0/3. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-03-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Model output is not valid structured claims JSON. [Full evidence](builtin-ai-qwen3.5_4b-fx-03-r1.json).

Expected decisions/actions: fx-03-e1: Measure latency first. [agreed]; fx-03-e2: Switch to Redis was recommended but not accepted. [proposed]

User priorities: fx-03-p1: advice versus acceptance; fx-03-p2: measure latency first

Forbidden inferences: Confirmed migration to Redis; Assigning work to the consultant

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | highlight / notes_only: Priority: advice must remain separate from accepted changes; measure latency first. (owner=None; deadline=None) | note:fx-03-n1: quoted `"Priority: advice must remain separate from accepted changes; measure latency first."`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i1 / raw | decision / conflict: Redis switch proposal is rejected/deferred with instruction to measure latency first. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `"Consultant: I recommend switching the cache to Redis. This is advice only."`; source: Consultant: I recommend switching the cache to Redis. This is advice only.; transcript:fx-03-t2: quoted `"Lead: We have not accepted Redis. We agreed to measure latency first."`; source: Lead: We have not accepted Redis. We agreed to measure latency first. | unsupported: Not accepted does not establish rejected/deferred; the raw conflict/rejection language changes the status. Rejected output is retained. | Pending |
| c1-a0-i2 / raw | action / notes_only: Measure latency. (owner=None; deadline=None) | note:fx-03-n1: quoted `"Priority: advice must remain separate from accepted changes; measure latency first."`; source: Priority: advice must remain separate from accepted changes; measure latency first.; transcript:fx-03-t2: quoted `"Lead: We have not accepted Redis. We agreed to measure latency first."`; source: Lead: We have not accepted Redis. We agreed to measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a1-i0 / repair | highlight / notes_only: Advice (Redis switch) must remain separate from accepted changes; the decision to measure latency first was not acceptance of Redis. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first.; transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only.; transcript:fx-03-t2: quoted `Lead: We have not accepted Redis. We agreed to measure latency first.`; source: Lead: We have not accepted Redis. We agreed to measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a1-unparsed / unparsed response | None / None: {"claims":[{"kind":"highlight","text":"Advice (Redis switch) must remain separate from accepted changes; the decision to measure latency first was not acceptance of Redis.","section":"Questions and Evidence","status":"notes_only","sources":[{"source_id":"note:fx-03-n1","excerpt":"Priority: advice must remain separate from accepted changes; measure latency first."},{"source_id":"transcript:fx-03-t1","excerpt":"Consultant: I recommend switching the cache to Redis. This is advice only."},{"source_id":"transcript:fx-03-t2","excerpt":"Lead: We have not accepted Redis. We agreed to measure latency first."}],"owner":null,"deadline":null}]}" (owner=None; deadline=None) |  | unparseable: Readable assertions inspected; malformed response is retained. t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |

Recall 0/2; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-03-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-03-r2.json).

Expected decisions/actions: fx-03-e1: Measure latency first. [agreed]; fx-03-e2: Switch to Redis was recommended but not accepted. [proposed]

User priorities: fx-03-p1: advice versus acceptance; fx-03-p2: measure latency first

Forbidden inferences: Confirmed migration to Redis; Assigning work to the consultant

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | highlight / notes_only: Priority: advice must remain separate from accepted changes; measure latency first. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i1 / raw | question / uncertain: Consultant recommends switching the cache to Redis, but this is stated as advice only. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i2 / raw | decision / review_required: Redis has not been accepted; it was agreed to measure latency first instead. (owner=None; deadline=None) | transcript:fx-03-t2: quoted `Lead: We have not accepted Redis. We agreed to measure latency first.`; source: Lead: We have not accepted Redis. We agreed to measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i3 / raw | next_step / notes_only: Measure latency before proceeding with the cache switch decision. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first.; transcript:fx-03-t2: quoted `Lead: We have not accepted Redis. We agreed to measure latency first.`; source: Lead: We have not accepted Redis. We agreed to measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-unparsed / unparsed response | None / None: ```json<br>[<br>  {<br>    "kind": "highlight",<br>    "text": "Priority: advice must remain separate from accepted changes; measure latency first.",<br>    "section": "Questions and Evidence",<br>    "status": "notes_only",<br>    "sources": [<br>      {<br>        "source_id": "note:fx-03-n1",<br>        "excerpt": "Priority: advice must remain separate from accepted changes; measure latency first."<br>      }<br>    ],<br>    "owner": null,<br>    "deadline": null<br>  },<br>  {<br>    "kind": "question",<br>    "text": "Consultant recommends switching the cache to Redis, but this is stated as advice only.",<br>    "section": "Advice and Recommendations",<br>    "status": "uncertain",<br>    "sources": [<br>      {<br>        "source_id": "transcript:fx-03-t1",<br>        "excerpt": "Consultant: I recommend switching the cache to Redis. This is advice only."<br>      }<br>    ],<br>    "owner": null,<br>    "deadline": null<br>  },<br>  {<br>    "kind": "decision",<br>    "text": "Redis has not been accepted; it was agreed to measure latency first instead.",<br>    "section": "Accepted Changes",<br>    "status": "review_required",<br>    "sources": [<br>      {<br>        "source_id": "transcript:fx-03-t2",<br>        "excerpt": "Lead: We have not accepted Redis. We agreed to measure latency first."<br>      }<br>    ],<br>    "owner": null,<br>    "deadline": null<br>  },<br>  {<br>    "kind": "next_step",<br>    "text": "Measure latency before proceeding with the cache switch decision.",<br>    "section": "Next Steps and Gaps",<br>    "status": "notes_only",<br>    "sources": [<br>      {<br>        "source_id": "note:fx-03-n1",<br>        "excerpt": "Priority: advice must remain separate from accepted changes; measure latency first."<br>      },<br>      {<br>        "source_id": "transcript:fx-03-t2",<br>        "excerpt": "Lead: We have not accepted Redis. We agreed to measure latency first."<br>      }<br>    ],<br>    "owner": null,<br>    "deadline": null<br>  }<br>]<br>``` (owner=None; deadline=None) |  | unparseable: Readable assertions inspected; malformed response is retained. t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| accepted-i0 / validated candidate | summary / review_required: Advice to switch the cache to Redis is not accepted; instead, measuring latency first is required. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first.; transcript:fx-03-t2: quoted `Lead: We have not accepted Redis. We agreed to measure latency first.`; source: Lead: We have not accepted Redis. We agreed to measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: Consultant recommendation to switch cache to Redis is explicitly identified as advice only. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-04-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-04-r1.json).

Expected decisions/actions: fx-04-e1: Expansion to Site B remains an unapproved proposal. [proposed]; fx-04-e2: Approve the safety review. [agreed]

User priorities: fx-04-p1: expansion proposal status; fx-04-p2: approved safety review

Forbidden inferences: Expansion approved; Inventing a resolution number, vote count or quorum

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting recorded a proposal to expand to Site B, which was not approved. Separately, the safety review for this expansion was approved by the Board. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review.; transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it.; transcript:fx-04-t2: quoted `Board: The safety review is approved.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | unsupported: No source connects the approved safety review specifically to the expansion. The summary adds that decision scope. | Pending |
| accepted-i1 / validated candidate | decision / review_required: Expansion to Site B was proposed but explicitly rejected/not approved. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | unsupported: Not approved does not mean explicitly rejected. The additional confirmed rejection is unsupported. | Pending |
| accepted-i2 / validated candidate | decision / review_required: The Board approved the safety review for the expansion. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | unsupported: Approval of safety review is stated, but approval of safety review for the expansion adds an unsupported scope. | Pending |
| accepted-i3 / validated candidate | highlight / review_required: No vote count or quorum statement was recorded in the transcript. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |

Recall 0/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-04-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-04-r2.json).

Expected decisions/actions: fx-04-e1: Expansion to Site B remains an unapproved proposal. [proposed]; fx-04-e2: Approve the safety review. [agreed]

User priorities: fx-04-p1: expansion proposal status; fx-04-p2: approved safety review

Forbidden inferences: Expansion approved; Inventing a resolution number, vote count or quorum

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / notes_only: The meeting prioritized distinguishing the proposed expansion from the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i1 / validated candidate | decision / uncertain: Expansion to Site B was proposed but not approved. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i2 / validated candidate | decision / review_required: The safety review is approved. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved. There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i3 / validated candidate | conflict / conflict: A conflict exists between the proposed expansion and the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | unsupported: n1 says distinguish the proposal and approval, not that a conflict exists between them. The conflict is invented. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-05-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Model claim lacks a required field. [Full evidence](builtin-ai-qwen3.5_4b-fx-05-r1.json).

Expected decisions/actions: fx-05-e1: Submit the inventory checklist. [agreed]

User priorities: fx-05-p1: inventory checklist; fx-05-p2: exact owner and date

Forbidden inferences: Changing the owner; Changing or converting the literal date

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | action / notes_only: Ana Cruz will submit the inventory checklist by 20 October 2026. (owner=Ana Cruz; deadline=20 October 2026) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist. | unsupported: Owner/date are true in actual t1, but the action cites only n1, which contains neither value. Literal-attribute validation rejects it. | Pending |
| c1-a0-i1 / raw | summary / uncertain: The meeting concluded with an agreement that Ana Cruz is responsible for submitting the inventory checklist by 20 October 2026, with a specific instruction to preserve the exact owner and due date details. (owner=None; deadline=None) | transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| c1-a1-i0 / repair | action / notes_only: Ana Cruz will submit the inventory checklist by 20 October 2026. (owner=Ana Cruz; deadline=20 October 2026) | transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| c1-a1-i1 / repair | highlight / notes_only: Priority: retain the exact owner and literal due date for the inventory checklist. (owner=None; deadline=None) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |

Recall 0/1; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-05-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-05-r2.json).

Expected decisions/actions: fx-05-e1: Submit the inventory checklist. [agreed]

User priorities: fx-05-p1: inventory checklist; fx-05-p2: exact owner and date

Forbidden inferences: Changing the owner; Changing or converting the literal date

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | action / review_required: Ana Cruz will submit the inventory checklist by 20 October 2026. (owner=None; deadline=None) | transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| accepted-i1 / validated candidate | highlight / notes_only: Priority: retain the exact owner and literal due date for the inventory checklist. (owner=None; deadline=None) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-06-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Model output is not valid structured claims JSON. [Full evidence](builtin-ai-qwen3.5_4b-fx-06-r1.json).

Expected decisions/actions: fx-06-e1: Prepare the options memo. [agreed]

User priorities: fx-06-p1: office assignment; fx-06-p2: options memo

Forbidden inferences: Inventing a named staff member; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | assignment / notes_only: The Procurement Office was assigned to prepare the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a0-i1 / raw | summary / notes_only: The assignment is to an office; no individual was named. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a0-i2 / raw | decision / notes_only: No deadline was specified for the task. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a1-i0 / repair | summary / notes_only: The board assigned the task of preparing an options memo to the Procurement Office, with no individual named or deadline specified. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named.; transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo. No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a1-unparsed / unparsed response | None / None: {"claims":[{"kind":"summary","text":"The board assigned the task of preparing an options memo to the Procurement Office, with no individual named or deadline specified.","section":"Matters Presented","status":"notes_only","sources":[{"source_id":"note:fx-06-n1","excerpt":"Priority: the assignment is to an office; no individual was named."},{"source_id":"transcript:fx-06-t1","excerpt":"The board assigns the Procurement Office to prepare the options memo. No deadline was specified."}],"owner":null,"deadline":null}]}" (owner=None; deadline=None) |  | unparseable: Readable assertions inspected; malformed response is retained. t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |

Recall 0/1; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-06-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-06-r2.json).

Expected decisions/actions: fx-06-e1: Prepare the options memo. [agreed]

User priorities: fx-06-p1: office assignment; fx-06-p2: options memo

Forbidden inferences: Inventing a named staff member; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: The board assigned the task of preparing the options memo to the Procurement Office, noting that no individual name was specified and no deadline was set. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named.; transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo. No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| c1-a0-i1 / raw | assignment / notes_only: Procurement Office (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i0 / validated candidate | summary / review_required: The board assigned the Procurement Office to prepare an options memo, noting that the assignment is to an office rather than an individual and no deadline was specified. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named.; transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo. No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-07-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-07-r1.json).

Expected decisions/actions: fx-07-e1: Update the risk register. [agreed]

User priorities: fx-07-p1: risk register; fx-07-p2: missing owner and deadline

Forbidden inferences: Resolving I to a person or office; Inferring an owner from microphone/system provenance

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting prioritized documenting the missing owner and due date for the risk register. While it was agreed to update the risk register, no specific name or deadline was stated by any participant. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `We agreed to update the risk register. Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: A participant offered assistance with updating the risk register but did not provide their name. (owner=None; deadline=None) | transcript:fx-07-t1: quoted `Someone said, I can help.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: The specific owner and deadline for the risk register update remain undocumented. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-07-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-07-r2.json).

Expected decisions/actions: fx-07-e1: Update the risk register. [agreed]

User priorities: fx-07-p1: risk register; fx-07-p2: missing owner and deadline

Forbidden inferences: Resolving I to a person or office; Inferring an owner from microphone/system provenance

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting prioritized documenting the missing owner and due date for the risk register. While it was agreed to update the risk register with a contribution from an unnamed individual, no specific name or deadline was stated. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `We agreed to update the risk register. Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | unsupported: t1 records an offer of help, not agreement to a contribution from that unnamed individual. The summary invents acceptance of the contribution. | Pending |
| accepted-i1 / validated candidate | highlight / notes_only: The meeting focused on the need to finalize the owner and due date for the risk register as a priority topic. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i2 / validated candidate | conflict / conflict: There is an agreement to update the risk register and a willingness to help, but explicit details regarding the owner and deadline are absent. (owner=None; deadline=None) | transcript:fx-07-t1: quoted `We agreed to update the risk register. Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-08-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-08-r1.json).

Expected decisions/actions: fx-08-e1: Possibly review the prototype next Friday if data arrive. [proposed]

User priorities: fx-08-p1: tentative prototype review; fx-08-p2: unresolved relative date

Forbidden inferences: A confirmed review; Converting next Friday to a calendar date; Treating the relative date as a binding deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting concluded with a tentative schedule regarding the prototype review, specifically noting that reviewing it next Friday is conditional on data arrival and has not been formally agreed upon. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor.; transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| accepted-i1 / validated candidate | highlight / uncertain: A proposal to review the prototype next Friday was made but explicitly stated as not agreed upon, pending data arrival. (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| accepted-i2 / validated candidate | action / notes_only: Retain the tentative schedule and its unresolved date anchor. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| accepted-i3 / validated candidate | question / uncertain: Has the agreement to review the prototype next Friday been finalized? (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-08-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-08-r2.json).

Expected decisions/actions: fx-08-e1: Possibly review the prototype next Friday if data arrive. [proposed]

User priorities: fx-08-p1: tentative prototype review; fx-08-p2: unresolved relative date

Forbidden inferences: A confirmed review; Converting next Friday to a calendar date; Treating the relative date as a binding deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting prioritized retaining a tentative schedule with an unresolved date anchor, noting that a review of the prototype next Friday was proposed but explicitly not agreed upon pending data arrival. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor.; transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: A proposal to review the prototype next Friday remains unagreed upon and is conditional on data arrival. (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| accepted-i2 / validated candidate | decision / notes_only: The tentative schedule with its unresolved date anchor is being retained as a priority. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-09-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-09-r1.json).

Expected decisions/actions: fx-09-e1: Reject Option A. [rejected]; fx-09-e2: Use Option B. [agreed]

User priorities: fx-09-p1: rejection of Option A and offline-access reason; fx-09-p2: selection of Option B

Forbidden inferences: Option A approved; Option A action assigned

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / notes_only: Priority: preserve the rejection of Option A and the selection of Option B. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i1 / validated candidate | decision / review_required: The board rejected Option A because it removes offline access. (owner=None; deadline=None) | transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i2 / validated candidate | decision / review_required: We agreed to use Option B. (owner=None; deadline=None) | transcript:fx-09-t2: quoted `We agreed to use Option B.`; source: We agreed to use Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-09-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-09-r2.json).

Expected decisions/actions: fx-09-e1: Reject Option A. [rejected]; fx-09-e2: Use Option B. [agreed]

User priorities: fx-09-p1: rejection of Option A and offline-access reason; fx-09-p2: selection of Option B

Forbidden inferences: Option A approved; Option A action assigned

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / notes_only: Priority is to preserve the rejection of Option A and the selection of Option B. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i1 / validated candidate | decision / review_required: The board rejected Option A because it removes offline access. (owner=None; deadline=None) | transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i2 / validated candidate | decision / review_required: We agreed to use Option B. (owner=None; deadline=None) | transcript:fx-09-t2: quoted `We agreed to use Option B.`; source: We agreed to use Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-10-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-10-r1.json).

Expected decisions/actions: fx-10-e1: Launch approval is disputed: notes say approved, transcript says deferred. [conflict]

User priorities: fx-10-p1: approval conflict

Forbidden inferences: Unqualified approved launch; Silently discarding either source

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | conflict / conflict: Source notes state the launch was approved, while the transcript states the launch was discussed but not approved and the decision was deferred. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict. My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |

Recall 1/1; priorities 1/1. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-10-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-10-r2.json).

Expected decisions/actions: fx-10-e1: Launch approval is disputed: notes say approved, transcript says deferred. [conflict]

User priorities: fx-10-p1: approval conflict

Forbidden inferences: Unqualified approved launch; Silently discarding either source

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | conflict / conflict: Notes state the launch was approved, while transcript states it was discussed but not approved and deferred. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict. My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| accepted-i1 / validated candidate | summary / uncertain: The meeting covered a launch proposal where conflicting records exist: notes indicate approval, whereas the transcript indicates the decision was deferred because it was not approved. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict. My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |

Recall 1/1; priorities 1/1. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-11-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-11-r1.json).

Expected decisions/actions: fx-11-e1: Approve Package Delta, the offline-access checklist. [agreed]; fx-11-e2: Luis Reyes will prepare Package Delta, the offline-access checklist. [agreed]

User priorities: fx-11-p1: Package Delta definition and final assignment; fx-11-p2: offline-access constraint

Forbidden inferences: Final agreement was only a proposal; Dropping the early definition; Assigning a deadline; A calibration log decision

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: The meeting prioritized connecting the final assignment to the early definition of Package Delta while retaining an offline-access constraint, noting that Package Delta currently represents only a proposal for the offline-access checklist. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.; transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: Malformed initial summary likewise calls the early proposal current despite the later approval in t3. | Pending |
| c1-a0-unparsed / unparsed response | None / None: {"claims":[{"kind":"summary","text":"The meeting prioritized connecting the final assignment to the early definition of Package Delta while retaining an offline-access constraint, noting that Package Delta currently represents only a proposal for the offline-access checklist.","section":"Summary","status":"notes_only","sources":[{"source_id":"note:fx-11-n1","excerpt":"Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint."},{"source_id":"transcript:fx-11-t1","excerpt":"Package Delta means the offline-access checklist. It is only a proposal for now."}],"owner":null,"deadline":null}]}" (owner=None; deadline=None) |  | unparseable: Readable assertions inspected; malformed response is retained. t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c2-a0-i0 / raw | summary / notes_only: The meeting segment contained only a background calibration log noting an unrelated sensor baseline observation, with no decisions or actions recorded. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `Background calibration log: Unrelated sensor baseline observation; no decision or action.`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON] | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c2-a0-unparsed / unparsed response | None / None: ```json<br>[{"kind":"summary","text":"The meeting segment contained only a background calibration log noting an unrelated sensor baseline observation, with no decisions or actions recorded.","section":"Summary","status":"notes_only","sources":[{"source_id":"transcript:fx-11-t2","excerpt":"Background calibration log: Unrelated sensor baseline observation; no decision or action."}],"owner":null,"deadline":null}]<br>``` (owner=None; deadline=None) |  | unparseable: Readable assertions inspected; malformed response is retained. t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i0 / validated candidate | summary / review_required: The meeting prioritized connecting the final assignment to the early definition of Package Delta while retaining an offline-access constraint. It was noted that Package Delta currently refers to the offline-access checklist and is only a proposal. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.; transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: Initial t1 proposal is described as current despite final t3 approval. Sources are not reconciled in the combined candidate. | Pending |
| accepted-i1 / validated candidate | summary / review_required: The meeting discussed sensor baseline observations and approved Package Delta, assigning it to Luis Reyes without setting a deadline. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `Unrelated sensor baseline observation; no decision or action.`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON]; transcript:fx-11-t3: quoted `We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i2 / validated candidate | decision / review_required: Package Delta is approved as defined at the start. (owner=None; deadline=None) | transcript:fx-11-t3: quoted `We now approve Package Delta as defined at the start.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i3 / validated candidate | action / review_required: Luis Reyes will prepare Package Delta. (owner=Luis Reyes; deadline=None) | transcript:fx-11-t3: quoted `Luis Reyes will prepare it;`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-11-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-11-r2.json).

Expected decisions/actions: fx-11-e1: Approve Package Delta, the offline-access checklist. [agreed]; fx-11-e2: Luis Reyes will prepare Package Delta, the offline-access checklist. [agreed]

User priorities: fx-11-p1: Package Delta definition and final assignment; fx-11-p2: offline-access constraint

Forbidden inferences: Final agreement was only a proposal; Dropping the early definition; Assigning a deadline; A calibration log decision

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | summary / review_required: The meeting addressed the priority of connecting the final assignment to the early definition of Package Delta while retaining an offline-access constraint, noting that Package Delta is currently defined as the offline-access checklist and remains a proposal. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.; transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: Initial t1 proposal is described as current despite final t3 approval. Sources are not reconciled in the combined candidate. | Pending |
| accepted-i1 / validated candidate | highlight / notes_only: Priority topic: Connect the final assignment to the early definition of Package Delta while retaining the offline-access constraint. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: Package Delta is defined as the offline-access checklist and is currently only a proposal. (owner=None; deadline=None) | transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: Currently only a proposal contradicts final t3 approval; it requires an initial-stage qualifier. | Pending |
| accepted-i3 / validated candidate | decision / review_required: Package Delta is approved as defined at the start. (owner=None; deadline=None) | transcript:fx-11-t3: quoted `We now approve Package Delta as defined at the start.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i4 / validated candidate | action / review_required: Luis Reyes will prepare Package Delta. (owner=Luis Reyes; deadline=None) | transcript:fx-11-t3: quoted `Luis Reyes will prepare it; no deadline was set.`; source: We now approve Package Delta as defined at the start. Luis Reyes will prepare it; no deadline was set. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| accepted-i5 / validated candidate | highlight / review_required: Unrelated sensor baseline observation noted multiple times with no decision or action. (owner=None; deadline=None) | transcript:fx-11-t2: quoted `Unrelated sensor baseline observation; no decision or action.`; source: Background calibration log: Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; no decision or action. Unrelated sensor baseline observation; [full passage in JSON] | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-12-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Unknown source or excerpt outside frozen input. [Full evidence](builtin-ai-qwen3.5_4b-fx-12-r1.json).

Expected decisions/actions: fx-12-e1: Keep offline access. [agreed]; fx-12-e2: Someone might check 安全確認 next week; name and acceptance are uncertain. [proposed]

User priorities: fx-12-p1: offline access; fx-12-p2: uncertain name and 安全確認; fx-12-p3: Café résumé and Δ notation

Forbidden inferences: Confirmed Mara or Nara as owner; Confirmed safety-check assignment; Calendar date inferred from next week; Dropping Unicode content

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | highlight / notes_only: Priority is to retain offline access; names should not be guessed. Japanese label: 安全確認. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a0-i1 / raw | highlight / notes_only: Agreement reached to retain offline access. (owner=None; deadline=None) | transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a0-i2 / raw | conflict / uncertain: Uncertainty exists regarding the name 'Mara? Nara?' which might check 安全確認 next week; no agreement has been reached on this point. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a0-i3 / raw | action / notes_only: None specified. (owner=None; deadline=None) |  | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a1-i0 / repair | summary / notes_only: Priority topic is to retain offline access; names should not be guessed. Japanese label: 安全確認. (owner=None; deadline=None) | note:fx-12-n1: quoted `"Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認."`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a1-i1 / repair | decision / notes_only: Agreement reached to retain offline access. (owner=None; deadline=None) | transcript:fx-12-t1: quoted `"Napagkasunduan: panatilihin ang offline access."`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a1-i2 / repair | conflict / uncertain: Uncertainty regarding potential check of 安全確認 next week; no agreement yet reached on this matter. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `"ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan."`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a1-i3 / repair | action / notes_only: No specific actions listed in the source text. (owner=None; deadline=None) |  | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |

Recall 0/2; priorities 0/3. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/builtin-ai-qwen3.5_4b-fx-12-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](builtin-ai-qwen3.5_4b-fx-12-r2.json).

Expected decisions/actions: fx-12-e1: Keep offline access. [agreed]; fx-12-e2: Someone might check 安全確認 next week; name and acceptance are uncertain. [proposed]

User priorities: fx-12-p1: offline access; fx-12-p2: uncertain name and 安全確認; fx-12-p3: Café résumé and Δ notation

Forbidden inferences: Confirmed Mara or Nara as owner; Confirmed safety-check assignment; Calendar date inferred from next week; Dropping Unicode content

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | highlight / notes_only: Priority topic: retain offline access; do not speculate on names. Japanese label: Safety Confirmation. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: Agreement reached to retain offline access. (owner=None; deadline=None) | transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i2 / validated candidate | conflict / conflict: Uncertainty regarding 'Safety Confirmation' check next week; no agreement reached on this specific point. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i3 / validated candidate | action / review_required: Retain offline access. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ.; transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |

Recall 2/2; priorities 2/3. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-01-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-01-r1.json).

Expected decisions/actions: fx-01-e1: Keep offline access in the pilot. [agreed]; fx-01-e2: Test the offline checklist. [agreed]

User priorities: fx-01-p1: offline access; fx-01-p2: existing budget

Forbidden inferences: Inventing a budget amount; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | decision / review_required: The group agreed to keep offline access in the pilot. (owner=None; deadline=None) | transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| accepted-i1 / validated candidate | summary / review_required: The meeting focused on establishing that the pilot must protect offline access while remaining within the existing budget. The group reached a decision to maintain offline access in the proposed pilot. (owner=None; deadline=None) | note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget.; transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot. | unsupported: The model attributes establishment of the user budget priority to the meeting. n1 is a priority; t1 agrees offline access only. | Pending |

Recall 1/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-01-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-01-r2.json).

Expected decisions/actions: fx-01-e1: Keep offline access in the pilot. [agreed]; fx-01-e2: Test the offline checklist. [agreed]

User priorities: fx-01-p1: offline access; fx-01-p2: existing budget

Forbidden inferences: Inventing a budget amount; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / notes_only: The group agreed to maintain offline access in the pilot, ensuring it protects against service disruption while remaining within the existing budget. (owner=None; deadline=None) | transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot.; note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget. | unsupported: The raw summary turns budget priority into part of group agreement and promises service-disruption protection. Neither is established; repair drops these additions. | Pending |
| accepted-i0 / validated candidate | summary / review_required: The group agreed to keep offline access in the pilot. (owner=None; deadline=None) | note:fx-01-n1: quoted `Priority: the pilot must protect offline access and stay within the existing budget.`; source: Priority: the pilot must protect offline access and stay within the existing budget.; transcript:fx-01-t1: quoted `The group agreed to keep offline access in the pilot.`; source: The group agreed to keep offline access in the pilot. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |
| accepted-i1 / validated candidate | action / review_required: Mina will test the offline checklist. (owner=None; deadline=None) | transcript:fx-01-t2: quoted `Mina will test the offline checklist. No due date was set.`; source: Mina will test the offline checklist. No due date was set. | supported: Transcript t1 agrees offline access; t2 explicitly names Mina and leaves the date absent. Note n1 states a budget priority, not a group budget commitment. | Pending |

Recall 2/2; priorities 1/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-02-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](ollama-qwen3.5_9b-mlx-fx-02-r1.json).

Expected decisions/actions: fx-02-e1: Add keyboard navigation. [agreed]; fx-02-e2: Review consent wording before rollout. [agreed]

User priorities: fx-02-p1: accessibility and keyboard navigation; fx-02-p2: slow connections; fx-02-p3: unresolved consent wording

Forbidden inferences: Treating rollout as a calendar deadline; Inventing an assignee

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-unparsed / unparsed response | None / None: {"claims":[{"kind":"highlight","text":"Priority focus areas: accessibility, slow connections, and unresolved consent wording.","section":"Highlights","status":"notes_only","sources":[{"source_id":"note:fx-02-n1","excerpt":"Priority: accessibility, slow connections, and unresolved consent wording."}][{"sort": "cf5d12af06240d94:37"}, {"source_id":"transcript:fx-02-t1", "excerpt": "We agreed to add keyboard navigation."}]}, {"kind":"summary","text":"The team agreed to implement keyboard navigation as a priority for accessibility.","section":"Highlights","status":"notes_only","sources":[{"source_id":"note:fx-02-n1", "excerpt": "Priority: accessibility, slow connections, and unresolved consent wording."}, {"sort": "cf5d12af06240d94:37"}, {"source_id": "transcript: fx-02-t1", "excerpt": "We agreed to add keyboard navigation."}]}]} (owner=None; deadline=None) |  | unparseable: The readable priority and keyboard-navigation assertions are supported by n1/t1, but the broken citation structure and transcript: fx-02-t1 ID are invalid. No candidate is delivered. | Pending |
| c1-a1-i0 / repair | priority / notes_only: Accessibility, slow connections, and unresolved consent wording are prioritized topics. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |

Recall 0/2; priorities 0/3. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-02-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-02-r2.json).

Expected decisions/actions: fx-02-e1: Add keyboard navigation. [agreed]; fx-02-e2: Review consent wording before rollout. [agreed]

User priorities: fx-02-p1: accessibility and keyboard navigation; fx-02-p2: slow connections; fx-02-p3: unresolved consent wording

Forbidden inferences: Treating rollout as a calendar deadline; Inventing an assignee

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c2-a0-i0 / raw | action / notes_only: Review consent wording before rollout (owner=None; deadline=None) | transcript:fx-02-t2: quoted `Review consent wording before rollout; nobody was assigned and no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| c2-a0-i1 / raw | action / uncertain: Assign owner for reviewing consent wording (owner=None; deadline=None) | transcript:fx-02-t2: quoted `nobody was assigned`; source: Review consent wording before rollout; nobody was assigned and no date was set. | qualified: These are proposed gap-resolution steps, not stated assignments. Explicit uncertain status and null owner/date avoid confirmed fabrication; they were discarded in repair. | Pending |
| c2-a0-i2 / raw | action / uncertain: Set deadline for reviewing consent wording (owner=None; deadline=None) | transcript:fx-02-t2: quoted `no date was set.`; source: Review consent wording before rollout; nobody was assigned and no date was set. | qualified: These are proposed gap-resolution steps, not stated assignments. Explicit uncertain status and null owner/date avoid confirmed fabrication; they were discarded in repair. | Pending |
| accepted-i0 / validated candidate | decision / review_required: We agreed to add keyboard navigation. (owner=None; deadline=None) | transcript:fx-02-t1: quoted `We agreed to add keyboard navigation.`; source: We agreed to add keyboard navigation. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i1 / validated candidate | highlight / notes_only: Priority: accessibility, slow connections, and unresolved consent wording. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i2 / validated candidate | question / uncertain: Accessibility needs to be addressed. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i3 / validated candidate | question / uncertain: Connection performance issues need to be addressed. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |
| accepted-i4 / validated candidate | question / uncertain: Unresolved consent wording needs to be addressed. (owner=None; deadline=None) | note:fx-02-n1: quoted `Priority: accessibility, slow connections, and unresolved consent wording.`; source: Priority: accessibility, slow connections, and unresolved consent wording. | supported: Note n1 gives the three priority topics; t1 agrees keyboard navigation; t2 requests consent review and explicitly lacks owner/date. | Pending |

Recall 1/2; priorities 3/3. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-03-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](ollama-qwen3.5_9b-mlx-fx-03-r1.json).

Expected decisions/actions: fx-03-e1: Measure latency first. [agreed]; fx-03-e2: Switch to Redis was recommended but not accepted. [proposed]

User priorities: fx-03-p1: advice versus acceptance; fx-03-p2: measure latency first

Forbidden inferences: Confirmed migration to Redis; Assigning work to the consultant

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | question / notes_only: What is the current latency of the system? (owner=None; deadline=None) | note:fx-03-n1: quoted `measure latency first`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i1 / raw | question / notes_only: Must any advice be separated from accepted changes? (owner=None; deadline=None) | note:fx-03-n1: quoted `advice must remain separate from accepted changes`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c2-a0-i0 / raw | advice_and_recommendations / notes_only: Consultant recommends switching the cache to Redis, explicitly stating this is advice only. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c2-a1-i0 / repair | advice_and_recommendations / notes_only: The consultant recommends switching the cache to Redis. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |

Recall 0/2; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-03-r2

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](ollama-qwen3.5_9b-mlx-fx-03-r2.json).

Expected decisions/actions: fx-03-e1: Measure latency first. [agreed]; fx-03-e2: Switch to Redis was recommended but not accepted. [proposed]

User priorities: fx-03-p1: advice versus acceptance; fx-03-p2: measure latency first

Forbidden inferences: Confirmed migration to Redis; Assigning work to the consultant

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | priority / notes_only: Advice must remain separate from accepted changes; measure latency first. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c1-a0-i1 / raw | highlight / notes_only: Measure latency first before determining accepted changes. (owner=None; deadline=None) | note:fx-03-n1: quoted `Priority: advice must remain separate from accepted changes; measure latency first.`; source: Priority: advice must remain separate from accepted changes; measure latency first. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c2-a0-i0 / raw | advice / notes_only: Switch the cache to Redis. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |
| c2-a1-i0 / repair | advice_and_recommendations / uncertain: Recommendation to switch the cache to Redis provided by the consultant, explicitly stated as advice only. (owner=None; deadline=None) | transcript:fx-03-t1: quoted `Consultant: I recommend switching the cache to Redis. This is advice only.`; source: Consultant: I recommend switching the cache to Redis. This is advice only. | supported: t1 is Redis advice only; t2 says not accepted and agrees latency measurement. Not accepted does not establish rejection. | Pending |

Recall 0/2; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-04-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-04-r1.json).

Expected decisions/actions: fx-04-e1: Expansion to Site B remains an unapproved proposal. [proposed]; fx-04-e2: Approve the safety review. [agreed]

User priorities: fx-04-p1: expansion proposal status; fx-04-p2: approved safety review

Forbidden inferences: Expansion approved; Inventing a resolution number, vote count or quorum

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c2-a0-i0 / raw | decision / notes_only: The safety review is approved. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `Board: The safety review is approved. There is no recorded vote count or quorum statement.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i0 / validated candidate | question / notes_only: Distinguish the proposed expansion from the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i1 / validated candidate | decision / conflict: Expansion to Site B was proposed; we did not approve it. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i2 / validated candidate | decision / review_required: The safety review is approved. (owner=None; deadline=None) | transcript:fx-04-t2: quoted `The safety review is approved.`; source: Board: The safety review is approved. There is no recorded vote count or quorum statement. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-04-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-04-r2.json).

Expected decisions/actions: fx-04-e1: Expansion to Site B remains an unapproved proposal. [proposed]; fx-04-e2: Approve the safety review. [agreed]

User priorities: fx-04-p1: expansion proposal status; fx-04-p2: approved safety review

Forbidden inferences: Expansion approved; Inventing a resolution number, vote count or quorum

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / uncertain: Distinction made between proposed expansion to Site B and the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review.; transcript:fx-04-t1: quoted `Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i0 / validated candidate | decision / review_required: Expansion to Site B was proposed. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i1 / validated candidate | decision / review_required: Expansion to Site B was not approved. (owner=None; deadline=None) | transcript:fx-04-t1: quoted `Chair: Expansion to Site B was proposed; we did not approve it.`; source: Chair: Expansion to Site B was proposed; we did not approve it. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i2 / validated candidate | highlight / notes_only: Need to distinguish the proposed expansion from the approved safety review. (owner=None; deadline=None) | note:fx-04-n1: quoted `Priority: distinguish the proposed expansion from the approved safety review.`; source: Priority: distinguish the proposed expansion from the approved safety review. | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |
| accepted-i3 / validated candidate | decision / missing_source: Safety review is approved. (owner=None; deadline=None) |  | supported: t1 proposes expansion without approval. t2 approves safety review without quorum or vote count. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-05-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-05-r1.json).

Expected decisions/actions: fx-05-e1: Submit the inventory checklist. [agreed]

User priorities: fx-05-p1: inventory checklist; fx-05-p2: exact owner and date

Forbidden inferences: Changing the owner; Changing or converting the literal date

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | action / notes_only: Submit the inventory checklist. (owner=Ana Cruz; deadline=20 October 2026) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist. | unsupported: Actual t1 gives Ana/date, but cited n1 does not. Attribute validation rejects the raw citation error. | Pending |
| c1-a0-i1 / raw | summary / confirmed: The meeting included a finalization of tasks where the team agreed that Ana Cruz will submit the inventory checklist by 20 October 2026. (owner=None; deadline=None) | transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| accepted-i0 / validated candidate | decision / review_required: Ana Cruz will submit the inventory checklist by 20 October 2026. (owner=None; deadline=None) | transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| accepted-i1 / validated candidate | action / review_required: Submit the inventory checklist. (owner=None; deadline=None) | transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| accepted-i2 / validated candidate | question / uncertain: How to retain the exact owner and due date for the inventory checklist as per priority? (owner=None; deadline=None) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| accepted-i3 / validated candidate | highlight / notes_only: The team discussed prioritizing the retention of the exact owner and literal due date for the inventory checklist. (owner=None; deadline=None) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist. | unsupported: n1 is a user priority; it does not establish that the team discussed this retention instruction. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-05-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-05-r2.json).

Expected decisions/actions: fx-05-e1: Submit the inventory checklist. [agreed]

User priorities: fx-05-p1: inventory checklist; fx-05-p2: exact owner and date

Forbidden inferences: Changing the owner; Changing or converting the literal date

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | action / review_required: Ana Cruz will submit the inventory checklist by 20 October 2026. (owner=Ana Cruz; deadline=20 October 2026) | transcript:fx-05-t1: quoted `We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026.`; source: We agreed: Ana Cruz will submit the inventory checklist by 20 October 2026. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |
| accepted-i1 / validated candidate | question / notes_only: Must the exact owner and literal due date for the inventory checklist be retained as per priority instruction? (owner=None; deadline=None) | note:fx-05-n1: quoted `Priority: retain the exact owner and literal due date for the inventory checklist.`; source: Priority: retain the exact owner and literal due date for the inventory checklist. | supported: t1 explicitly states Ana Cruz, inventory checklist and 20 October 2026. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-06-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-06-r1.json).

Expected decisions/actions: fx-06-e1: Prepare the options memo. [agreed]

User priorities: fx-06-p1: office assignment; fx-06-p2: options memo

Forbidden inferences: Inventing a named staff member; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | action / notes_only: Matters Presented topic: Priority assignment is to an office, with no individual named. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i1 / validated candidate | summary / review_required: The board assigns the Procurement Office to prepare the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i2 / validated candidate | action / uncertain: Procurement Office to prepare the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-06-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-06-r2.json).

Expected decisions/actions: fx-06-e1: Prepare the options memo. [agreed]

User priorities: fx-06-p1: office assignment; fx-06-p2: options memo

Forbidden inferences: Inventing a named staff member; Inventing a deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | question / uncertain: The assignment is to an office with no individual named. (owner=None; deadline=None) | note:fx-06-n1: quoted `Priority: the assignment is to an office; no individual was named.`; source: Priority: the assignment is to an office; no individual was named. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i1 / validated candidate | action / review_required: The Procurement Office is assigned to prepare an options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `The board assigns the Procurement Office to prepare the options memo.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |
| accepted-i2 / validated candidate | highlight / review_required: No deadline was specified for the preparation of the options memo. (owner=None; deadline=None) | transcript:fx-06-t1: quoted `No deadline was specified.`; source: The board assigns the Procurement Office to prepare the options memo. No deadline was specified. | supported: t1 assigns Procurement Office, the options memo and no deadline. n1 expressly names no individual. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-07-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-07-r1.json).

Expected decisions/actions: fx-07-e1: Update the risk register. [agreed]

User priorities: fx-07-p1: risk register; fx-07-p2: missing owner and deadline

Forbidden inferences: Resolving I to a person or office; Inferring an owner from microphone/system provenance

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | action / missing_source: Agreed to update the risk register. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `We agreed to update the risk register.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| c1-a0-i1 / raw | action / conflict: Update the risk register. No name or deadline was stated. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| c1-a0-i2 / raw | discussion_highlight / conflict: Participants agreed to update the risk register with no identified owner or deadline. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register.; transcript:fx-07-t1: quoted `We agreed to update the risk register. Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| c1-a0-i3 / raw | priority / uncertain: Document the missing owner and due date for the risk register. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i0 / validated candidate | decision / review_required: The team agreed to update the risk register. (owner=None; deadline=None) | transcript:fx-07-t1: quoted `We agreed to update the risk register.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i1 / validated candidate | question / uncertain: The name of the person who can help with updating the risk register was not stated. (owner=None; deadline=None) | transcript:fx-07-t1: quoted `Someone said, I can help. No name or deadline was stated.`; source: We agreed to update the risk register. Someone said, I can help. No name or deadline was stated. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |
| accepted-i2 / validated candidate | highlight / notes_only: It is a priority to document the missing owner and due date for the risk register. (owner=None; deadline=None) | note:fx-07-n1: quoted `Priority: document the missing owner and due date for the risk register.`; source: Priority: document the missing owner and due date for the risk register. | supported: t1 agrees risk-register update and contains an unnamed offer of help, with no name/date. Note n1 prioritizes these gaps. | Pending |

Recall 1/1; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-07-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-07-r2.json).

Expected decisions/actions: fx-07-e1: Update the risk register. [agreed]

User priorities: fx-07-p1: risk register; fx-07-p2: missing owner and deadline

Forbidden inferences: Resolving I to a person or office; Inferring an owner from microphone/system provenance

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-08-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](ollama-qwen3.5_9b-mlx-fx-08-r1.json).

Expected decisions/actions: fx-08-e1: Possibly review the prototype next Friday if data arrive. [proposed]

User priorities: fx-08-p1: tentative prototype review; fx-08-p2: unresolved relative date

Forbidden inferences: A confirmed review; Converting next Friday to a calendar date; Treating the relative date as a binding deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / uncertain: The meeting reviewed a tentative schedule, noting that while it is to be retained, specific dates remain unresolved and certain proposals (such as reviewing the prototype next Friday) were not yet agreed upon. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor.; transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| c1-a1-i0 / repair | decision / conflict: There is a tentative decision to review the prototype next Friday, contingent on data arrival. (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| c1-a1-i1 / repair | decision / conflict: The decision to review the prototype next Friday remains unresolved and unagreed upon. (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| c1-a1-i2 / repair | action / uncertain: Retain the tentative schedule and its unresolved date anchor. (owner=None; deadline=None) | note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |
| c1-a1-i3 / repair | discussion_highlight / notes_only: Discussion centered on revisiting the prototype review scheduled for next Friday, which is conditional upon data arrival and currently unresolved. (owner=None; deadline=None) | transcript:fx-08-t1: quoted `Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.`; source: Maybe we could review the prototype next Friday, if the data arrive. This was not agreed.; note:fx-08-n1: quoted `Priority: retain the tentative schedule and its unresolved date anchor.`; source: Priority: retain the tentative schedule and its unresolved date anchor. | supported: t1 is a conditional, unagreed proposal for next Friday. Note n1 prioritizes the unresolved date anchor. | Pending |

Recall 0/1; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-08-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-08-r2.json).

Expected decisions/actions: fx-08-e1: Possibly review the prototype next Friday if data arrive. [proposed]

User priorities: fx-08-p1: tentative prototype review; fx-08-p2: unresolved relative date

Forbidden inferences: A confirmed review; Converting next Friday to a calendar date; Treating the relative date as a binding deadline

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |

Recall 0/1; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-09-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-09-r1.json).

Expected decisions/actions: fx-09-e1: Reject Option A. [rejected]; fx-09-e2: Use Option B. [agreed]

User priorities: fx-09-p1: rejection of Option A and offline-access reason; fx-09-p2: selection of Option B

Forbidden inferences: Option A approved; Option A action assigned

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | decision / conflict: The board rejected Option A because it removes offline access. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B.; transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| c1-a0-i1 / raw | decision / uncertain: Option B was selected. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| c1-a0-i2 / raw | summary / notes_only: Matters presented include the rejection of Option A due to lack of offline access and the prioritization of selecting Option B. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B.; transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| c1-a0-i3 / raw | risk / uncertain: Option A was rejected due to the removal of offline access. (owner=None; deadline=None) | transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i0 / validated candidate | decision / review_required: The board rejected Option A because it removes offline access. (owner=None; deadline=None) | transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i1 / validated candidate | decision / notes_only: Priority: preserve the rejection of Option A and the selection of Option B. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i2 / validated candidate | highlight / uncertain: Option B is to be selected as per priority instructions. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | unsupported: n1 asks to preserve an already selected B; it does not direct a future selection. Final agreement is separately retained, but this extra row changes the temporal status. | Pending |
| accepted-i3 / validated candidate | decision / review_required: The group agreed to use Option B. (owner=None; deadline=None) | transcript:fx-09-t2: quoted `We agreed to use Option B.`; source: We agreed to use Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-09-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-09-r2.json).

Expected decisions/actions: fx-09-e1: Reject Option A. [rejected]; fx-09-e2: Use Option B. [agreed]

User priorities: fx-09-p1: rejection of Option A and offline-access reason; fx-09-p2: selection of Option B

Forbidden inferences: Option A approved; Option A action assigned

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| accepted-i0 / validated candidate | decision / uncertain: The board rejected Option A because it removes offline access. (owner=None; deadline=None) | transcript:fx-09-t1: quoted `The board rejected Option A because it removes offline access.`; source: The board rejected Option A because it removes offline access. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i1 / validated candidate | summary / uncertain: Priority: preserve the rejection of Option A and the selection of Option B. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i2 / validated candidate | highlight / uncertain: Preserve the rejection of Option A. (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |
| accepted-i3 / validated candidate | highlight / uncertain: Select Option B (preservation required). (owner=None; deadline=None) | note:fx-09-n1: quoted `Priority: preserve the rejection of Option A and the selection of Option B.`; source: Priority: preserve the rejection of Option A and the selection of Option B. | unsupported: The user priority is to preserve the recorded B selection, not perform a new selection. Explicit uncertainty remains, but the row is not source-supported. | Pending |
| accepted-i4 / validated candidate | decision / review_required: Agreement was reached to use Option B. (owner=None; deadline=None) | transcript:fx-09-t2: quoted `We agreed to use Option B.`; source: We agreed to use Option B. | supported: t1 rejects A for losing offline access; t2 agrees B. n1 states the priority to retain both outcomes. | Pending |

Recall 2/2; priorities 2/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-10-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Invalid claim content or kind. [Full evidence](ollama-qwen3.5_9b-mlx-fx-10-r1.json).

Expected decisions/actions: fx-10-e1: Launch approval is disputed: notes say approved, transcript says deferred. [conflict]

User priorities: fx-10-p1: approval conflict

Forbidden inferences: Unqualified approved launch; Silently discarding either source

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | summary / conflict: The meeting discussed the launch topic; while one set of notes indicates approval, the primary transcript states the decision was deferred and not approved. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict. My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| c1-a0-i1 / raw | decision / review_required: Deferral of the launch decision. (owner=None; deadline=None) | transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| c1-a1-i0 / repair | conflict / conflict: Conflict exists regarding launch approval status: Note indicates approval while transcript states deferral. (owner=None; deadline=None) | note:fx-10-n1: quoted `My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| c1-a1-i1 / repair | notes_only / notes_only: Priority identified to resolve the conflict regarding launch decision. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict.`; source: Priority: resolve the conflict. My notes say the launch was approved. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| c1-a1-i2 / repair | summary / uncertain: The meeting discussed a launch but did not approve it; instead, the decision was deferred. A priority was noted to resolve this conflict. (owner=None; deadline=None) | transcript:fx-10-t1: quoted `We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision.; note:fx-10-n1: quoted `Priority: resolve the conflict.`; source: Priority: resolve the conflict. My notes say the launch was approved. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |

Recall 0/1; priorities 0/1. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-10-r2

Outcome: rejected_generation; error: Structured output failed validation after one repair: Model claim lacks a required field. [Full evidence](ollama-qwen3.5_9b-mlx-fx-10-r2.json).

Expected decisions/actions: fx-10-e1: Launch approval is disputed: notes say approved, transcript says deferred. [conflict]

User priorities: fx-10-p1: approval conflict

Forbidden inferences: Unqualified approved launch; Silently discarding either source

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | question / conflict: How should the conflict regarding the launch approval status be resolved? (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict. My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| c1-a0-i1 / raw | decision / notes_only: The launch was discussed but not approved; the decision was deferred. (owner=None; deadline=None) | transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| c1-a0-i2 / raw | action / review_required: Resolve the conflict regarding the launch approval status. (owner=None; deadline=None) | note:fx-10-n1: quoted `Priority: resolve the conflict.`; source: Priority: resolve the conflict. My notes say the launch was approved. | supported: n1 says approved while t1 says not approved/deferred; both records must remain in conflict. | Pending |
| c1-a1-i0 / repair | summary / uncertain: The meeting discussed the launch topic, resulting in a deferred decision rather than an approval. (owner=None; deadline=None) | note:fx-10-n1: quoted `My notes say the launch was approved.`; source: Priority: resolve the conflict. My notes say the launch was approved.; transcript:fx-10-t1: quoted `The launch was discussed but not approved. We deferred the decision.`; source: The launch was discussed but not approved. We deferred the decision. | qualified: The transcript supports deferral; n1 disagrees. Explicit uncertain status makes this a disputed conclusion requiring human reconciliation, not a confirmed resolution. | Pending |

Recall 0/1; priorities 0/1. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-11-r1

Outcome: rejected_generation; error: Structured output failed validation after one repair: Model claim lacks a required field. [Full evidence](ollama-qwen3.5_9b-mlx-fx-11-r1.json).

Expected decisions/actions: fx-11-e1: Approve Package Delta, the offline-access checklist. [agreed]; fx-11-e2: Luis Reyes will prepare Package Delta, the offline-access checklist. [agreed]

User priorities: fx-11-p1: Package Delta definition and final assignment; fx-11-p2: offline-access constraint

Forbidden inferences: Final agreement was only a proposal; Dropping the early definition; Assigning a deadline; A calibration log decision

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | decision / uncertain: Priority should be placed on connecting the final assignment to the early definition of Package Delta, while retaining the offline-access constraint. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c1-a0-i1 / raw | action / uncertain: The current understanding is that 'Package Delta' means the offline-access checklist and it remains only a proposal. (owner=None; deadline=None) | transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: The first-chunk proposal lacks initial-stage qualification relative to final t3 approval. Whole generation fails before later chunks; nothing is delivered. | Pending |
| c1-a0-i2 / raw | action / uncertain: Connect the final assignment to the early definition of Package Delta. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c1-a0-i3 / raw | action / uncertain: Retain the offline-access constraint. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c1-a0-i4 / raw | discussion_highlight / uncertain: Package Delta is being discussed as equivalent to the offline-access checklist. (owner=None; deadline=None) | transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c1-a0-i5 / raw | discussion_highlight / uncertain: The status of Package Delta (offline-access checklist) is currently only a proposal. (owner=None; deadline=None) | transcript:fx-11-t1: quoted `It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: The first-chunk proposal lacks initial-stage qualification relative to final t3 approval. Whole generation fails before later chunks; nothing is delivered. | Pending |
| c1-a1-i0 / repair | decision / notes_only: The requirement to connect the final assignment to the early definition of Package Delta is being retained. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c1-a1-i1 / repair | decision / notes_only: A constraint related to offline access is being retained for Package Delta. (owner=None; deadline=None) | note:fx-11-n1: quoted `retain the offline-access constraint`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c1-a1-i2 / repair | decision / notes_only: 'Package Delta' currently means the offline-access checklist and is only a proposal at this stage. (owner=None; deadline=None) | transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: The first-chunk proposal lacks initial-stage qualification relative to final t3 approval. Whole generation fails before later chunks; nothing is delivered. | Pending |

Recall 0/2; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-11-r2

Outcome: rejected_generation; error: Structured output failed validation after one repair: Model claim lacks a required field. [Full evidence](ollama-qwen3.5_9b-mlx-fx-11-r2.json).

Expected decisions/actions: fx-11-e1: Approve Package Delta, the offline-access checklist. [agreed]; fx-11-e2: Luis Reyes will prepare Package Delta, the offline-access checklist. [agreed]

User priorities: fx-11-p1: Package Delta definition and final assignment; fx-11-p2: offline-access constraint

Forbidden inferences: Final agreement was only a proposal; Dropping the early definition; Assigning a deadline; A calibration log decision

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | priority / notes_only: Connect the final assignment to the early definition of Package Delta and retain the offline-access constraint. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c1-a0-i1 / raw | highlight / notes_only: Package Delta is defined as the offline-access checklist but remains only a proposal at this stage. (owner=None; deadline=None) | transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: The first-chunk proposal lacks initial-stage qualification relative to final t3 approval. Whole generation fails before later chunks; nothing is delivered. | Pending |
| c1-a1-i0 / repair | highlight / notes_only: Priority is to connect the final assignment to the early definition of Package Delta while retaining the offline-access constraint. (owner=None; deadline=None) | note:fx-11-n1: quoted `Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint.`; source: Priority: connect the final assignment to the early definition of Package Delta; retain the offline-access constraint. | supported: t1 initially defines Delta as the offline checklist and a proposal. t3 later approves it and names Luis Reyes, without a date. t2 is only calibration filler. | Pending |
| c1-a1-i1 / repair | highlight / notes_only: Package Delta is currently defined as the offline-access checklist and remains only a proposal. (owner=None; deadline=None) | transcript:fx-11-t1: quoted `Package Delta means the offline-access checklist. It is only a proposal for now.`; source: Package Delta means the offline-access checklist. It is only a proposal for now. | unsupported: The first-chunk proposal lacks initial-stage qualification relative to final t3 approval. Whole generation fails before later chunks; nothing is delivered. | Pending |

Recall 0/2; priorities 0/2. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-12-r1

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-12-r1.json).

Expected decisions/actions: fx-12-e1: Keep offline access. [agreed]; fx-12-e2: Someone might check 安全確認 next week; name and acceptance are uncertain. [proposed]

User priorities: fx-12-p1: offline access; fx-12-p2: uncertain name and 安全確認; fx-12-p3: Café résumé and Δ notation

Forbidden inferences: Confirmed Mara or Nara as owner; Confirmed safety-check assignment; Calendar date inferred from next week; Dropping Unicode content

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c1-a0-i0 / raw | priority / notes_only: Maintain offline access. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a0-i1 / raw | summary / notes_only: Agreement to maintain offline access was reached. (owner=None; deadline=None) | transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a0-i2 / raw | highlight / notes_only: Do not guess the name. Japanese label: 安全確認 (Safety confirmation). (owner=None; deadline=None) | note:fx-12-n1: quoted `huwag hulaan ang pangalan. Japanese label: 安全確認.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| c1-a0-i3 / raw | highlight / notes_only: Café résumé — Δ (Delta/Change noted). (owner=None; deadline=None) | note:fx-12-n1: quoted `Café résumé — Δ.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | unsupported: Delta is a literal symbol in n1. Change noted interprets the symbol as a recorded change without source support; this raw chunk is rejected. | Pending |
| c2-a0-i0 / raw | highlight / uncertain: Safety verification might be checked next week but is not yet certain or agreed upon. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i0 / validated candidate | highlight / conflict: It is uncertain if [Mara/Nara] will check the safety confirmation (安全確認) next week, as this has not yet been clarified or agreed upon. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |

Recall 1/2; priorities 1/3. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

## corrected/ollama-qwen3.5_9b-mlx-fx-12-r2

Outcome: structurally_accepted_candidate; error: none. [Full evidence](ollama-qwen3.5_9b-mlx-fx-12-r2.json).

Expected decisions/actions: fx-12-e1: Keep offline access. [agreed]; fx-12-e2: Someone might check 安全確認 next week; name and acceptance are uncertain. [proposed]

User priorities: fx-12-p1: offline access; fx-12-p2: uncertain name and 安全確認; fx-12-p3: Café résumé and Δ notation

Forbidden inferences: Confirmed Mara or Nara as owner; Confirmed safety-check assignment; Calendar date inferred from next week; Dropping Unicode content

| Row / stage | Generated claim / fields | Actual supporting passage(s) | Codex review | Human verdict / initials |
| --- | --- | --- | --- | --- |
| c2-a0-i0 / raw | question / uncertain: Who might check security confirmation next week? (owner=None; deadline=None) | transcript:fx-12-t2: quoted `[Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i0 / validated candidate | highlight / notes_only: Priority topic: Maintain offline access; do not guess the name of the feature yet. (owner=None; deadline=None) | note:fx-12-n1: quoted `Priority: panatilihin ang offline access; huwag hulaan ang pangalan.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | unsupported: The uncertain name is an ASR person identity in t2. n1 does not say name of the feature yet; the model invents a feature-naming topic. | Pending |
| accepted-i1 / validated candidate | highlight / review_required: Agreement reached to maintain offline access. (owner=None; deadline=None) | transcript:fx-12-t1: quoted `Napagkasunduan: panatilihin ang offline access.`; source: Napagkasunduan: panatilihin ang offline access. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i2 / validated candidate | conflict / conflict: Uncertainty regarding future naming; proposal to not guess the name is explicitly noted. (owner=None; deadline=None) | note:fx-12-n1: quoted `huwag hulaan ang pangalan.`; source: Priority: panatilihin ang offline access; huwag hulaan ang pangalan. Japanese label: 安全確認. Café résumé — Δ. | unsupported: No source records future naming or a naming proposal. t2 is uncertainty about a person checking the safety label. | Pending |
| accepted-i3 / validated candidate | question / uncertain: Who might check safety confirmation next week? (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |
| accepted-i4 / validated candidate | conflict / conflict: The proposal regarding safety confirmation next week is uncertain and not agreed upon. (owner=None; deadline=None) | transcript:fx-12-t2: quoted `ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan.`; source: ASR uncertain: [Mara? Nara?] might check 安全確認 next week; hindi pa tiyak at hindi napagkasunduan. | supported: t1 agrees offline access. t2 is an unagreed, tentative safety-label check with ambiguous Mara/Nara ASR and relative week. n1 separately retains Café résumé and Delta notation. | Pending |

Recall 2/2; priorities 2/3. Content recall may use summaries or multiple delivered claims. Invalid additional claims are assessed separately. Empty and rejected generations earn zero credit. Human acceptance pending.

