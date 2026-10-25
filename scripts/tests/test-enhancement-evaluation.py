import importlib.util
import unittest
from pathlib import Path

spec=importlib.util.spec_from_file_location('review',Path(__file__).resolve().parents[1]/'review-meeting-enhancement.py')
review=importlib.util.module_from_spec(spec);spec.loader.exec_module(review)
spec=importlib.util.spec_from_file_location('producer',Path(__file__).resolve().parents[1]/'evaluate-meeting-enhancement.py')
producer=importlib.util.module_from_spec(spec);spec.loader.exec_module(producer)

class LocalProviderBoundaryTests(unittest.TestCase):
    def test_endpoint_validation_and_localhost_pinning(self):
        self.assertEqual(producer.local_endpoint('http://localhost:11434/'),'http://127.0.0.1:11434')
        for endpoint in ('http://127.0.0.1:11434','http://[::1]:11434'):
            self.assertEqual(producer.local_endpoint(endpoint),endpoint)
        for endpoint in ('https://example.invalid','http://localhost.example.invalid',
                         'http://192.168.1.2:11434','http://user@localhost:11434',
                         'http://localhost:11434?key=secret','file:///tmp/model'):
            with self.assertRaises(ValueError): producer.local_endpoint(endpoint)
    def test_redirects_cannot_forward_local_payloads(self):
        with self.assertRaises(ValueError):producer.NoRedirect().redirect_request(None,None,307,'',{},'https://example.invalid')

class ReviewMetricsTests(unittest.TestCase):
    def test_compact_passage_references_and_subdivided_attempts_remain_reviewable(self):
        import json
        run=self.run_record(False)
        raw=json.dumps({'claims':[{'kind':'highlight','text':'Review risks','citations':['p0']}]})
        source={'source_id':'note:original','text':'Review risks. 日本語'}
        run['attempts']=[{'chunk':1,'partition':part,'attempt':0,'raw_response':raw,
                         'validation_error':None,'sources':[source]} for part in ('0L','0R')]
        rows=review.claim_rows(run)
        self.assertEqual(len({row['row_id'] for row in rows}),2)
        self.assertEqual(rows[0]['claim']['sources'],[{'source_id':'note:original','excerpt':'Review risks. 日本語'}])

    def run_record(self,accepted):
        c={'kind':'action','text':'Review risks','status':'uncertain','sources':[],'owner':None,'deadline':None}
        return {'run_id':'synthetic','phase':'test','outcome':'structurally_accepted_candidate' if accepted else 'rejected_generation',
            'elapsed_ms':1,'error':None,'attempts':[{'chunk':1,'attempt':0,'raw_response':'{"claims":[]}',
                'validation_error':None,'sources':[]}], 'source_snapshot':{'sources':[]},
            'validated_candidate':{'claims':[c]} if accepted else None,
            'fixture':{'id':'fx-01','annotations':{'relevant_decisions_actions':[{'id':'e1'}],'user_priorities':[{'id':'p1'}]}}}
    def test_rejected_generation_cannot_receive_recall_credit(self):
        r=self.run_record(False); assessment={'claims':{},'recalled_expectations':{},'retained_priorities':{}}
        m=review.metrics(r,assessment);self.assertEqual((m['recalled'],m['retained']),(0,0))
        self.assertEqual(m['human_acceptance'],'pending');self.assertEqual(review.claim_rows(r),[])
        assessment['recalled_expectations']={'e1':['accepted-i0']}
        with self.assertRaises(AssertionError):review.metrics(r,assessment)
    def test_every_claim_requires_source_based_review(self):
        r=self.run_record(True);assessment={'claims':{},'recalled_expectations':{},'retained_priorities':{}}
        with self.assertRaises(AssertionError):review.metrics(r,assessment)
        assessment['claims']={'accepted-i0':{'support':'unsupported','notes':'No source supports it'}}
        m=review.metrics(r,assessment);self.assertEqual(m['accepted_violations']['unsupported_claim'],1)
    def test_matching_labels_need_delivered_claims_and_count_once(self):
        r=self.run_record(True);assessment={'claims':{'accepted-i0':{'support':'qualified','notes':'Missing source is explicit'}},
            'recalled_expectations':{'e1':['accepted-i0']},'retained_priorities':{'p1':['accepted-i0']}}
        m=review.metrics(r,assessment);self.assertEqual((m['recall_pct'],m['priority_pct']),(100,100))
        assessment['retained_priorities']={'invented-label':['accepted-i0']}
        with self.assertRaises(AssertionError):review.metrics(r,assessment)
    def test_malformed_raw_is_visible_empty_claims_are_not_parse_errors(self):
        r=self.run_record(False);r['attempts'][0]['raw_response']='{"claims":[oops]}'
        self.assertEqual(review.claim_rows(r)[0]['stage'],'unparsed response')
    def test_wrong_source_is_failure_even_with_exact_text_and_review_credit(self):
        r=self.run_record(True)
        r['source_snapshot']['sources']=[{'source_id':'transcript:local','text':'Exact source text'}]
        r['validated_candidate']['claims'][0]['sources']=[{'source_id':'transcript:foreign','excerpt':'Exact source text'}]
        assessment={'claims':{'accepted-i0':{'support':'supported','notes':'Deliberately wrong reviewer classification'}},
            'recalled_expectations':{'e1':['accepted-i0']},'retained_priorities':{'p1':['accepted-i0']}}
        m=review.metrics(r,assessment);self.assertEqual(m['accepted_violations']['invalid_reference'],1)
        self.assertEqual(m['codex_review_outcome'],'fails run quality checks')
    def test_invalid_envelope_readable_claims_are_reviewed_individually(self):
        r=self.run_record(False);r['attempts'][0]['raw_response']='[{"text":"First assertion"},{"text":"Second assertion"}]'
        rows=review.claim_rows(r)
        self.assertEqual(len(rows),3);self.assertEqual(rows[-1]['stage'],'unparsed response')
        self.assertEqual(review.parse_response(r['attempts'][0]['raw_response']),None)

if __name__=='__main__':unittest.main()
