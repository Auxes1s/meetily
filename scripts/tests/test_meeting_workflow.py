"""Private-library isolation contracts for the Q01 runner."""
import importlib.util
import pathlib
import sqlite3
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parents[1] / 'run-meeting-workflow.py'
SPEC = importlib.util.spec_from_file_location('meeting_workflow', SCRIPT)
workflow = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(workflow)


class WorkflowCopyTests(unittest.TestCase):
    def test_pending_jobs_and_all_session_paths_are_isolated_before_launch(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            source = root / 'original.sqlite'
            connection = sqlite3.connect(source)
            connection.executescript('''
                CREATE TABLE meetings(id TEXT PRIMARY KEY,status TEXT,folder_path TEXT);
                CREATE TABLE transcripts(id TEXT,meeting_id TEXT,transcript TEXT,audio_start_time REAL,audio_end_time REAL);
                CREATE TABLE settings(id TEXT,provider TEXT);
                CREATE TABLE recording_sessions(id TEXT,meeting_id TEXT,folder_path TEXT);
                CREATE TABLE meeting_deletion_jobs(job_id TEXT,folder_path TEXT);
                CREATE TABLE meeting_deletion_tombstones(meeting_id TEXT);
                CREATE TABLE meeting_deletion_sessions(id TEXT);
            ''')
            for table in ('meeting_notes', 'meeting_enhancement_versions', 'meeting_enhancement_settings',
                          'meeting_note_links', 'meeting_participants', 'meeting_tags'):
                connection.execute(f'CREATE TABLE {table}(meeting_id TEXT)')
            original_folders = [root / 'first-session', root / 'second-session']
            for ordinal, folder in enumerate(original_folders):
                folder.mkdir()
                (folder / 'audio.mp4').write_bytes(b'original source audio')
                (folder / 'metadata.json').write_text('{"meeting_id":"owner"}')
                connection.execute('INSERT INTO recording_sessions VALUES(?,?,?)',
                                   (f'session-{ordinal}', 'owner', str(folder)))
            connection.execute('INSERT INTO meetings VALUES(?,?,?)', ('owner', 'completed', str(original_folders[1])))
            connection.execute('INSERT INTO transcripts VALUES(?,?,?,?,?)', ('passage', 'owner', 'Preserved original', 0, 5))
            connection.execute('INSERT INTO meeting_deletion_jobs VALUES(?,?)', ('pending-original-job', str(original_folders[0])))
            connection.execute('INSERT INTO settings VALUES(?,?)', ('1', 'saved-external-provider'))
            connection.commit()
            before = workflow.meeting_digest(connection, 'owner')
            run = root / 'meetily-q01-fixture'
            run.mkdir()
            profile = root / 'isolated-profile'
            profile.mkdir()
            saved_profile = workflow.PROFILE
            workflow.PROFILE = profile
            try:
                meeting_id, count, digest, _ = workflow.prepare_copy(source, 'owner', run)
            finally:
                workflow.PROFILE = saved_profile
            copied = sqlite3.connect(profile / 'meeting_minutes.sqlite')
            self.assertEqual((meeting_id, count, digest), ('owner', 1, before))
            self.assertEqual(copied.execute('SELECT COUNT(*) FROM meeting_deletion_jobs').fetchone()[0], 0)
            paths = copied.execute('SELECT folder_path FROM meetings UNION SELECT folder_path FROM recording_sessions').fetchall()
            self.assertTrue(all(pathlib.Path(path).is_relative_to(run / 'recordings') for (path,) in paths))
            self.assertEqual(copied.execute('SELECT COUNT(*) FROM settings').fetchone()[0], 0)
            self.assertEqual(workflow.meeting_digest(connection, 'owner'), before)
            self.assertEqual(connection.execute('SELECT COUNT(*) FROM meeting_deletion_jobs').fetchone()[0], 1)
            self.assertTrue(all((folder / 'audio.mp4').read_bytes() == b'original source audio' for folder in original_folders))
            copied.close()
            connection.close()


if __name__ == '__main__':
    unittest.main()
