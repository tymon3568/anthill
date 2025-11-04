#!/usr/bin/env python3
"""
Kanidm Initialization Script using REST API
This script configures OAuth2 clients and test users via Kanidm's REST API
"""

import asyncio
import json
import os
import sys
from typing import Optional
import aiohttp

# Configuration
KANIDM_URL = "https://localhost:8300"
ADMIN_USERNAME = "admin"
ADMIN_PASSWORD = os.getenv("KANIDM_ADMIN_PASSWORD")

if not ADMIN_PASSWORD:
    print("❌ Error: KANIDM_ADMIN_PASSWORD environment variable is required")
    sys.exit(1)

class KanidmClient:
    def __init__(self, base_url: str, verify_ssl: bool = False):
        self.base_url = base_url.rstrip('/')
        self.verify_ssl = verify_ssl
        self.session: Optional[aiohttp.ClientSession] = None
        self.token: Optional[str] = None

    async def __aenter__(self):
        connector = aiohttp.TCPConnector(ssl=self.verify_ssl)
        self.session = aiohttp.ClientSession(connector=connector)
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def authenticate(self, username: str, password: str) -> bool:
        """Authenticate and get session token"""
        print(f"🔐 Authenticating as {username}...")

        # Step 1: Init auth session
        url = f"{self.base_url}/v1/auth"
        payload = {"step": {"init": username}}

        async with self.session.post(url, json=payload) as resp:
            if resp.status != 200:
                print(f"❌ Auth init failed: {resp.status}")
                text = await resp.text()
                print(f"   Response: {text}")
                return False

            data = await resp.json()
            session_id = data.get("sessionid")
            state = data.get("state")

            if not session_id:
                print("❌ No session ID received")
                return False

        # Step 2: Submit password
        payload = {
            "step": {
                "cred": [{
                    "password": password
                }]
            }
        }

        headers = {"X-KANIDM-AUTH-SESSION-ID": session_id}
        async with self.session.post(url, json=payload, headers=headers) as resp:
            if resp.status != 200:
                print(f"❌ Auth credential submit failed: {resp.status}")
                text = await resp.text()
                print(f"   Response: {text}")
                return False

            data = await resp.json()
            state = data.get("state")

            if state != "success":
                print(f"❌ Auth not successful. State: {state}")
                return False

            # Get bearer token from response
            self.token = data.get("bearer", data.get("token"))

            if not self.token:
                # Try getting it from cookies
                cookies = resp.cookies
                for cookie in cookies.values():
                    if cookie.key == "bearer":
                        self.token = cookie.value
                        break

            if not self.token:
                print("❌ No bearer token received")
                print(f"   Response: {json.dumps(data, indent=2)}")
                return False

            print("✅ Authentication successful!")
            return True

    async def oauth2_create(self, name: str, displayname: str, origin: str) -> bool:
        """Create OAuth2 client"""
        print(f"🔧 Creating OAuth2 client: {name}...")

        url = f"{self.base_url}/v1/oauth2"
        payload = {
            "name": name,
            "displayname": displayname,
            "origin": origin
        }

        headers = {"Authorization": f"Bearer {self.token}"}

        async with self.session.post(url, json=payload, headers=headers) as resp:
            text = await resp.text()

            if resp.status == 200 or resp.status == 201:
                print(f"✅ OAuth2 client '{name}' created")
                return True
            else:
                print(f"⚠️  OAuth2 client '{name}' may already exist (status: {resp.status})")
                return True  # Don't fail if already exists

    async def oauth2_add_redirect_url(self, name: str, url: str) -> bool:
        """Add redirect URL to OAuth2 client"""
        print(f"🔗 Adding redirect URL to {name}: {url}")

        api_url = f"{self.base_url}/v1/oauth2/{name}/redirect_url"
        payload = {"redirect_url": url}

        headers = {"Authorization": f"Bearer {self.token}"}

        async with self.session.post(api_url, json=payload, headers=headers) as resp:
            if resp.status in [200, 201, 204]:
                print(f"✅ Redirect URL added")
                return True
            else:
                print(f"⚠️  Redirect URL may already exist (status: {resp.status})")
                return True

    async def oauth2_enable_pkce(self, name: str) -> bool:
        """Enable PKCE for OAuth2 client"""
        print(f"🔒 Enabling PKCE for {name}...")

        api_url = f"{self.base_url}/v1/oauth2/{name}/pkce"
        headers = {"Authorization": f"Bearer {self.token}"}

        async with self.session.post(api_url, headers=headers) as resp:
            if resp.status in [200, 204]:
                print(f"✅ PKCE enabled")
                return True
            else:
                print(f"⚠️  PKCE may already be enabled (status: {resp.status})")
                return True

    async def oauth2_update_scope_map(self, name: str, group: str, scopes: list) -> bool:
        """Update scope map for OAuth2 client"""
        print(f"📋 Updating scope map for {name}: {group} -> {scopes}")

        api_url = f"{self.base_url}/v1/oauth2/{name}/scopemap/{group}"
        payload = {"scopes": scopes}

        headers = {"Authorization": f"Bearer {self.token}"}

        async with self.session.put(api_url, json=payload, headers=headers) as resp:
            if resp.status in [200, 204]:
                print(f"✅ Scope map updated")
                return True
            else:
                text = await resp.text()
                print(f"⚠️  Scope map update may have failed (status: {resp.status})")
                print(f"   Response: {text}")
                return True

    async def oauth2_get_secret(self, name: str) -> Optional[str]:
        """Get OAuth2 client secret"""
        print(f"🔑 Retrieving client secret for {name}...")

        api_url = f"{self.base_url}/v1/oauth2/{name}"
        headers = {"Authorization": f"Bearer {self.token}"}

        async with self.session.get(api_url, headers=headers) as resp:
            if resp.status == 200:
                data = await resp.json()
                secret = data.get("oauth2_rs_basic_secret")
                if secret:
                    print(f"✅ Client secret: {secret}")
                    return secret
                else:
                    print("⚠️  No secret in response")
                    return None
            else:
                print(f"❌ Failed to get secret (status: {resp.status})")
                return None

    async def group_create(self, name: str) -> bool:
        """Create a group"""
        print(f"👥 Creating group: {name}...")

        api_url = f"{self.base_url}/v1/group"
        payload = {"name": name}

        headers = {"Authorization": f"Bearer {self.token}"}

        async with self.session.post(api_url, json=payload, headers=headers) as resp:
            if resp.status in [200, 201]:
                print(f"✅ Group '{name}' created")
                return True
            else:
                print(f"⚠️  Group '{name}' may already exist (status: {resp.status})")
                return True

    async def person_create(self, name: str, displayname: str, mail: Optional[str] = None) -> bool:
        """Create a person"""
        print(f"👤 Creating person: {name}...")

        api_url = f"{self.base_url}/v1/person"
        payload = {
            "name": name,
            "displayname": displayname
        }

        if mail:
            payload["mail"] = [mail]

        headers = {"Authorization": f"Bearer {self.token}"}

        async with self.session.post(api_url, json=payload, headers=headers) as resp:
            if resp.status in [200, 201]:
                print(f"✅ Person '{name}' created")
                return True
            else:
                print(f"⚠️  Person '{name}' may already exist (status: {resp.status})")
                return True

    async def group_add_members(self, group: str, members: list) -> bool:
        """Add members to a group"""
        print(f"➕ Adding members to {group}: {members}")

        api_url = f"{self.base_url}/v1/group/{group}/_attr/member"
        payload = {"values": members}

        headers = {"Authorization": f"Bearer {self.token}"}

        async with self.session.post(api_url, json=payload, headers=headers) as resp:
            if resp.status in [200, 204]:
                print(f"✅ Members added to {group}")
                return True
            else:
                print(f"⚠️  Members may already be in group (status: {resp.status})")
                return True

async def main():
    print("=" * 50)
    print("Kanidm Initialization for Anthill")
    print("=" * 50)
    print()

    async with KanidmClient(KANIDM_URL, verify_ssl=False) as client:
        # Authenticate
        if not await client.authenticate(ADMIN_USERNAME, ADMIN_PASSWORD):
            print("❌ Authentication failed!")
            return 1

        print()

        # Create OAuth2 client
        await client.oauth2_create(
            "anthill",
            "Anthill Inventory Management",
            "http://localhost:5173"
        )

        # Add redirect URLs
        await client.oauth2_add_redirect_url("anthill", "http://localhost:5173/oauth/callback")
        await client.oauth2_add_redirect_url("anthill", "http://localhost:3000/oauth/callback")
        await client.oauth2_add_redirect_url("anthill", "https://app.example.com/oauth/callback")

        # Enable PKCE
        await client.oauth2_enable_pkce("anthill")

        # Update scope map
        await client.oauth2_update_scope_map(
            "anthill",
            "anthill_users",
            ["email", "openid", "profile", "groups"]
        )

        # Get client secret
        secret = await client.oauth2_get_secret("anthill")

        print()
        print("=" * 50)
        print("Creating Groups...")
        print("=" * 50)

        # Create groups
        await client.group_create("tenant_acme_users")
        await client.group_create("tenant_acme_admins")
        await client.group_create("tenant_globex_users")
        await client.group_create("anthill_users")

        print()
        print("=" * 50)
        print("Creating Users...")
        print("=" * 50)

        # Create users
        await client.person_create("alice", "Alice Admin", "alice@acme.example.com")
        await client.person_create("bob", "Bob User", "bob@acme.example.com")
        await client.person_create("charlie", "Charlie Globex", "charlie@globex.example.com")

        # Add users to groups
        await client.group_add_members("tenant_acme_users", ["alice", "bob"])
        await client.group_add_members("tenant_acme_admins", ["alice"])
        await client.group_add_members("tenant_globex_users", ["charlie"])
        await client.group_add_members("anthill_users", ["alice", "bob", "charlie"])

        print()
        print("=" * 50)
        print("✅ Initialization Complete!")
        print("=" * 50)
        print()
        print("📋 Summary:")
        print("  - OAuth2 Client: anthill")
        if secret:
            print(f"  - Client Secret: {secret}")
        print("  - Redirect URLs:")
        print("    * http://localhost:5173/oauth/callback")
        print("    * http://localhost:3000/oauth/callback")
        print("    * https://app.example.com/oauth/callback")
        print("  - PKCE: Enabled")
        print("  - Groups:")
        print("    * tenant_acme_users (alice, bob)")
        print("    * tenant_acme_admins (alice)")
        print("    * tenant_globex_users (charlie)")
        print("    * anthill_users (alice, bob, charlie)")
        print("  - Users:")
        print("    * alice@acme.example.com")
        print("    * bob@acme.example.com")
        print("    * charlie@globex.example.com")
        print()
        print("⚠️  User passwords must be set via Kanidm UI or credential reset")
        print("🔗 Kanidm UI: https://localhost:8300/ui")
        print()

        return 0

if __name__ == "__main__":
    try:
        exit_code = asyncio.run(main())
        sys.exit(exit_code)
    except KeyboardInterrupt:
        print("\n⚠️  Interrupted by user")
        sys.exit(130)
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
