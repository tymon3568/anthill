use crate::client::{CreateOAuth2Request, KanidmAdminClient};
use anyhow::Result;
use tracing::info;

pub struct KanidmSetup {
    client: KanidmAdminClient,
}

impl KanidmSetup {
    pub fn new(client: KanidmAdminClient) -> Self {
        Self { client }
    }

    pub async fn setup_oauth2_client(&self) -> Result<()> {
        info!("═══════════════════════════════════════");
        info!("Setting up OAuth2 Client");
        info!("═══════════════════════════════════════");

        // Create OAuth2 client
        self.client
            .create_oauth2_client(CreateOAuth2Request {
                name: "anthill".to_string(),
                displayname: "Anthill Inventory Management".to_string(),
                origin: "http://localhost:5173".to_string(),
            })
            .await?;

        // Add redirect URLs
        self.client
            .add_redirect_url("anthill", "http://localhost:5173/oauth/callback")
            .await?;

        self.client
            .add_redirect_url("anthill", "http://localhost:8000/oauth/callback")
            .await?;

        self.client
            .add_redirect_url("anthill", "https://app.example.com/oauth/callback")
            .await?;

        // Enable PKCE
        self.client.enable_pkce("anthill").await?;

        // Update scope map
        self.client
            .update_scope_map(
                "anthill",
                "anthill_users",
                vec![
                    "email".to_string(),
                    "openid".to_string(),
                    "profile".to_string(),
                    "groups".to_string(),
                ],
            )
            .await?;

        // Get client details
        let client_info = self.client.get_oauth2_client("anthill").await?;

        info!("");
        info!("🔑 OAuth2 Client Secret: {:?}", client_info.oauth2_rs_basic_secret);
        info!("");

        Ok(())
    }

    pub async fn setup_groups(&self) -> Result<()> {
        info!("═══════════════════════════════════════");
        info!("Setting up Groups");
        info!("═══════════════════════════════════════");

        // Create tenant groups
        self.client.create_group("tenant_acme_users").await?;
        self.client.create_group("tenant_acme_admins").await?;
        self.client.create_group("tenant_globex_users").await?;

        // Create application group
        self.client.create_group("anthill_users").await?;

        Ok(())
    }

    pub async fn setup_test_users(&self) -> Result<()> {
        info!("═══════════════════════════════════════");
        info!("Setting up Test Users");
        info!("═══════════════════════════════════════");

        // Create users
        self.client
            .create_person("alice", "Alice Admin", Some("alice@acme.example.com"))
            .await?;

        self.client
            .create_person("bob", "Bob User", Some("bob@acme.example.com"))
            .await?;

        self.client
            .create_person("charlie", "Charlie Globex", Some("charlie@globex.example.com"))
            .await?;

        // Add users to groups
        self.client
            .add_group_members("tenant_acme_users", vec!["alice".to_string(), "bob".to_string()])
            .await?;

        self.client
            .add_group_members("tenant_acme_admins", vec!["alice".to_string()])
            .await?;

        self.client
            .add_group_members("tenant_globex_users", vec!["charlie".to_string()])
            .await?;

        self.client
            .add_group_members(
                "anthill_users",
                vec![
                    "alice".to_string(),
                    "bob".to_string(),
                    "charlie".to_string(),
                ],
            )
            .await?;

        Ok(())
    }

    pub fn print_summary(&self) {
        info!("");
        info!("═══════════════════════════════════════");
        info!("✅ Kanidm Initialization Complete!");
        info!("═══════════════════════════════════════");
        info!("");
        info!("📋 Summary:");
        info!("  - OAuth2 Client: anthill");
        info!("  - Redirect URLs:");
        info!("    * http://localhost:5173/oauth/callback");
        info!("    * http://localhost:8000/oauth/callback");
        info!("    * https://app.example.com/oauth/callback");
        info!("  - PKCE: Enabled");
        info!("  - Scope Map: anthill_users -> [email, openid, profile, groups]");
        info!("");
        info!("  - Groups Created:");
        info!("    * tenant_acme_users (alice, bob)");
        info!("    * tenant_acme_admins (alice)");
        info!("    * tenant_globex_users (charlie)");
        info!("    * anthill_users (alice, bob, charlie)");
        info!("");
        info!("  - Test Users:");
        info!("    * alice@acme.example.com (Acme admin)");
        info!("    * bob@acme.example.com (Acme user)");
        info!("    * charlie@globex.example.com (Globex user)");
        info!("");
        info!("⚠️  User passwords must be set via Kanidm UI:");
        info!("   https://localhost:8300/ui");
        info!("");
        info!("🔗 OAuth2 Endpoints:");
        info!("   - Authorize: https://localhost:8300/ui/oauth2");
        info!("   - Token: https://localhost:8300/oauth2/token");
        info!("   - UserInfo: https://localhost:8300/oauth2/openid/anthill/userinfo");
        info!("   - JWKS: https://localhost:8300/oauth2/openid/anthill/public_key.jwk");
        info!("");
    }
}
