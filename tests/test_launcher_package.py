import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from tools import check_launcher_package as smoke


class PackageSmokeTests(unittest.TestCase):
    def run_package(self, change=None):
        homes = []
        def run(args, **kwargs):
            env = kwargs['env']
            self.assertFalse(any(key.startswith('HORIZON_ELY_') for key in env))
            self.assertNotIn('HORIZON_MSA_CLIENT_ID', env)
            self.assertEqual(env['HORIZON_LAUNCHER_NO_SELF_UPDATE'], '1')
            self.assertTrue(all(Path(env[key]).is_dir() for key in smoke.HOMES))
            homes.append(kwargs['cwd'])
            if args[-1] == 'ely-status':
                result = dict(client_id='elyhorizondesktop', bundled_client_id='elyhorizondesktop',
                              flow='authorization_code_pkce', code_challenge_method='S256', client_secret_required=False)
                if change:
                    result.update(change)
                return SimpleNamespace(returncode=0, stdout=json.dumps(result))
            if args[-1] == 'list':
                return SimpleNamespace(returncode=0, stdout=json.dumps(dict(accounts=[], can_play=False)))
            return SimpleNamespace(returncode=0, stdout='horizon-launcher 0.2.2\n')
        smoke.check('launcher', 'client', 'elyhorizondesktop', run=run)
        self.assertEqual(len(homes), 5)
        self.assertEqual(len(set(homes)), 1)
        self.assertFalse(Path(homes[0]).exists())

    def test_installed_configuration_is_checked_without_runtime_credentials(self):
        self.run_package()

    def test_missing_embedded_id_or_insecure_flow_is_rejected(self):
        for change in [dict(bundled_client_id=None), dict(client_secret_required=True), dict(code_challenge_method='plain')]:
            with self.subTest(change=change), self.assertRaisesRegex(ValueError, 'bundled public'):
                self.run_package(change)
