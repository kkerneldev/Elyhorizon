import unittest
from tools import check_ely_oauth as gate


class ReleaseConfigurationTests(unittest.TestCase):
    def test_missing_id_fails_without_contacting_the_network(self):
        for client_id in (None, "", "  "):
            with self.assertRaisesRegex(ValueError, "Actions Variable"):
                gate.check(client_id, lambda *_: self.fail("network used without an ID"))

    def test_public_flow_checks_pkce_and_secret_free_exchange(self):
        def service(path, fields=None, post=False):
            if fields:
                self.assertNotIn("client_secret", fields)
            if path.startswith("/.well-known/"):
                return 200, dict(issuer=gate.ISSUER, code_challenge_methods_supported=["S256"], token_endpoint_auth_methods_supported=["none"])
            if post:
                self.assertEqual(path, "/api/oauth2/v1/token")
                self.assertEqual(fields["grant_type"], "authorization_code")
                return 400, dict(error="invalid_grant")
            if "code_challenge" not in fields:
                return 400, dict(parameter="code_challenge")
            if fields["code_challenge_method"] == "plain":
                return 400, dict(parameter="code_challenge_method")
            return 200, dict(success=True)
        gate.check("public-desktop", service)

    def test_website_registration_is_rejected_even_if_pkce_is_supported_globally(self):
        def service(path, *_):
            if path.startswith("/.well-known/"):
                return 200, dict(issuer=gate.ISSUER, code_challenge_methods_supported=["S256"], token_endpoint_auth_methods_supported=["none"])
            return 200, dict(success=True)
        with self.assertRaisesRegex(ValueError, "Desktop application"):
            gate.check("website-app", service)

    def test_invalid_public_id_reports_the_provider_status_without_asking_for_a_secret(self):
        def service(path, *_):
            if path.startswith("/.well-known/"):
                return 200, dict(issuer=gate.ISSUER, code_challenge_methods_supported=["S256"], token_endpoint_auth_methods_supported=["none"])
            return 401, dict(error="invalid_client", error_description="reflected-private-value")
        with self.assertRaisesRegex(ValueError, "HTTP 401 invalid_client") as raised:
            gate.check("elyhorizondesktop", service)
        self.assertNotIn("reflected-private-value", str(raised.exception))
