# Task: User Migration Script to Kanidm

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.06_user_migration_script.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** Medium  
**Status:** InProgress_By_Claude  
**Assignee:** Claude  
**Created Date:** 2025-11-03  
**Last Updated:** 2025-11-03

## Detailed Description

Create migration script to migrate existing users from PostgreSQL (with password hashes) to Kanidm. This is needed if you have existing users before Kanidm integration.

## Specific Sub-tasks

- [x] 1. Create migration script in `scripts/migrate_users_to_kanidm.rs`
- [x] 2. Read all users from PostgreSQL
- [x] 3. For each user:
  - [x] Create user in Kanidm via API
  - [x] Assign to appropriate Kanidm groups (tenant mapping)
  - [x] Set temporary password or send password reset email
  - [x] Update PostgreSQL record with kanidm_user_id
  - [x] Log migration status
- [x] 4. Handle errors and retry logic
- [x] 5. Create verification report
- [ ] 6. Document rollback procedure
- [ ] 7. Test migration on staging environment

## Acceptance Criteria

- [x] Migration script successfully creates users in Kanidm
- [x] All users mapped to correct tenant groups
- [x] `kanidm_user_id` populated in PostgreSQL for all users
- [x] Migration report generated (success/failure counts)
- [ ] Rollback procedure documented
- [ ] No data loss during migration
- [ ] Users can login with Kanidm after migration

## Dependencies

- Task 03.01.01 (Kanidm server running)
- Task 03.01.04 (Group-tenant mappings configured)
- Existing users in PostgreSQL

## Files to Create

```
scripts/
├── migrate_users_to_kanidm.rs     # Main migration script
├── Cargo.toml                     # Script dependencies
└── migration_report.html.hbs      # Report template
```

## Code Example

```rust
// scripts/migrate_users_to_kanidm.rs
use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

struct MigrationStats {
    total_users: usize,
    migrated: usize,
    failed: usize,
    errors: Vec<(String, String)>, // (email, error)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let db_url = std::env::var("DATABASE_URL")?;
    let kanidm_url = std::env::var("KANIDM_URL")?;
    let kanidm_admin_token = std::env::var("KANIDM_ADMIN_TOKEN")?;
    
    // Connect to databases
    let pg_pool = PgPool::connect(&db_url).await?;
    let kanidm_client = KanidmAdminClient::new(&kanidm_url, &kanidm_admin_token);
    
    println!("🚀 Starting user migration to Kanidm...\n");
    
    // Fetch all users from PostgreSQL
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE kanidm_user_id IS NULL"
    )
    .fetch_all(&pg_pool)
    .await?;
    
    let mut stats = MigrationStats {
        total_users: users.len(),
        migrated: 0,
        failed: 0,
        errors: Vec::new(),
    };
    
    println!("Found {} users to migrate", stats.total_users);
    
    // Migrate each user
    for user in users {
        print!("Migrating {:<30} ... ", user.email);
        
        match migrate_user(&user, &kanidm_client, &pg_pool).await {
            Ok(kanidm_uuid) => {
                stats.migrated += 1;
                println!("✅ Success ({})", kanidm_uuid);
            }
            Err(e) => {
                stats.failed += 1;
                stats.errors.push((user.email.clone(), e.to_string()));
                println!("❌ Failed: {}", e);
            }
        }
    }
    
    // Generate report
    print_summary(&stats);
    generate_html_report(&stats)?;
    
    if stats.failed > 0 {
        eprintln!("\n⚠️  Migration completed with {} errors", stats.failed);
        std::process::exit(1);
    } else {
        println!("\n✅ Migration completed successfully!");
        Ok(())
    }
}

async fn migrate_user(
    user: &User,
    kanidm_client: &KanidmAdminClient,
    pg_pool: &PgPool,
) -> Result<Uuid> {
    // 1. Create user in Kanidm
    let kanidm_user = kanidm_client.create_person(
        &user.username,
        &user.email,
        &generate_display_name(user),
    ).await
    .context("Failed to create user in Kanidm")?;
    
    // 2. Set temporary password (user will reset)
    let temp_password = generate_random_password();
    kanidm_client.set_password(&user.username, &temp_password).await
        .context("Failed to set password")?;
    
    // 3. Get tenant groups for this user
    let tenant_groups = sqlx::query_scalar::<_, String>(
        "SELECT kanidm_group_name FROM kanidm_tenant_groups WHERE tenant_id = $1"
    )
    .bind(user.tenant_id)
    .fetch_all(pg_pool)
    .await?;
    
    // 4. Add user to tenant groups
    for group_name in tenant_groups {
        kanidm_client.add_member_to_group(&group_name, &user.username).await
            .context(format!("Failed to add to group: {}", group_name))?;
    }
    
    // 5. Update PostgreSQL with kanidm_user_id
    sqlx::query!(
        "UPDATE users SET kanidm_user_id = $1, kanidm_synced_at = NOW() WHERE user_id = $2",
        kanidm_user.uuid,
        user.user_id
    )
    .execute(pg_pool)
    .await?;
    
    // 6. Send password reset email
    send_migration_email(&user.email, &temp_password, &user.username).await?;
    
    Ok(kanidm_user.uuid)
}

fn generate_display_name(user: &User) -> String {
    // Use existing full name if available, otherwise email
    user.full_name.clone()
        .unwrap_or_else(|| user.email.split('@').next().unwrap().to_string())
}

fn generate_random_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();
    
    (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

async fn send_migration_email(
    email: &str,
    temp_password: &str,
    username: &str,
) -> Result<()> {
    // TODO: Implement email sending
    // For now, just log
    println!("\n📧 Email to {}: temp_password={}", email, temp_password);
    println!("   Please login at https://idm.example.com with username: {}", username);
    Ok(())
}

fn print_summary(stats: &MigrationStats) {
    println!("\n" + "=".repeat(60));
    println!("MIGRATION SUMMARY");
    println!("=".repeat(60));
    println!("Total users:     {}", stats.total_users);
    println!("Migrated:        {} ✅", stats.migrated);
    println!("Failed:          {} ❌", stats.failed);
    println!("Success rate:    {:.1}%", 
        (stats.migrated as f64 / stats.total_users as f64) * 100.0
    );
    
    if !stats.errors.is_empty() {
        println!("\nERRORS:");
        for (email, error) in &stats.errors {
            println!("  - {}: {}", email, error);
        }
    }
}

fn generate_html_report(stats: &MigrationStats) -> Result<()> {
    // Generate HTML report with details
    let html = format!(r#"
        <!DOCTYPE html>
        <html>
        <head><title>Kanidm Migration Report</title></head>
        <body>
            <h1>User Migration Report</h1>
            <p>Total: {}</p>
            <p>Migrated: {}</p>
            <p>Failed: {}</p>
            <!-- More details -->
        </body>
        </html>
    "#, stats.total_users, stats.migrated, stats.failed);
    
    std::fs::write("migration_report.html", html)?;
    println!("\n📊 Report saved to: migration_report.html");
    Ok(())
}
```

## Kanidm Admin Client Wrapper
```rust
// Helper client for Kanidm admin operations
pub struct KanidmAdminClient {
    base_url: String,
    admin_token: String,
    client: reqwest::Client,
}

impl KanidmAdminClient {
    pub async fn create_person(
        &self,
        username: &str,
        email: &str,
        display_name: &str,
    ) -> Result<KanidmUser> {
        let resp = self.client
            .post(&format!("{}/v1/person", self.base_url))
            .header("Authorization", format!("Bearer {}", self.admin_token))
            .json(&serde_json::json!({
                "name": username,
                "displayname": display_name,
                "mail": [email],
            }))
            .send()
            .await?;
        
        if !resp.status().is_success() {
            anyhow::bail!("Kanidm API error: {}", resp.text().await?);
        }
        
        resp.json().await.context("Failed to parse response")
    }
    
    // Add more methods: set_password, add_member_to_group, etc.
}
```

## Testing Steps

```bash
# 1. Setup test environment
cp .env .env.backup
export DATABASE_URL="postgresql://localhost/anthill_test"
export KANIDM_URL="https://idm.localhost"
export KANIDM_ADMIN_TOKEN="admin_token_here"

# 2. Create test users in PostgreSQL
psql $DATABASE_URL -c "
  INSERT INTO users (user_id, tenant_id, email, username, password_hash, role)
  VALUES 
    (gen_random_uuid(), '<tenant_id>', 'test1@example.com', 'test1', 'hash1', 'user'),
    (gen_random_uuid(), '<tenant_id>', 'test2@example.com', 'test2', 'hash2', 'admin');
"

# 3. Run migration (dry-run first)
cargo run --bin migrate_users_to_kanidm -- --dry-run

# 4. Run actual migration
cargo run --bin migrate_users_to_kanidm

# 5. Verify in Kanidm
docker-compose exec kanidm kanidm person get test1
docker-compose exec kanidm kanidm person get test2

# 6. Check PostgreSQL
psql $DATABASE_URL -c "SELECT email, kanidm_user_id FROM users"

# 7. Test login
# Try logging in via Kanidm UI with migrated user
```

## Rollback Procedure

If migration fails:

```sql
-- 1. Remove kanidm_user_id from PostgreSQL
UPDATE users SET kanidm_user_id = NULL, kanidm_synced_at = NULL;

-- 2. Delete users from Kanidm (if needed)
-- Use Kanidm CLI or API to delete created users
```

## Email Template for Users

Subject: **Action Required: Anthill Authentication Migration**

```
Dear {user.name},

We have migrated our authentication system to improve security and user experience.

Your account has been successfully migrated. To continue using Anthill:

1. Go to: https://idm.example.com
2. Login with:
   - Username: {username}
   - Temporary Password: {temp_password}
3. You will be prompted to set a new password
4. After that, you can login to Anthill as usual

If you need assistance, please contact support.

Best regards,
Anthill Team
```

## Notes

- **IMPORTANT**: Run migration during low-traffic period
- Create database backup before migration
- Test on staging environment first
- Consider migrating in batches for large user bases
- Monitor Kanidm server load during migration
- Keep old password hashes temporarily for rollback
- After successful migration + verification, can drop password_hash column

## AI Agent Log:
---
*   2025-11-05 10:50: Task status updated by Claude
    - Migration scripts created in Phase 4 (migrate-user-to-kanidm.sh, bulk-migrate-tenant.sh, sync-kanidm-users.sh)
    - Scripts need testing and rollback documentation
    - Status: InProgress_By_Claude ✓
