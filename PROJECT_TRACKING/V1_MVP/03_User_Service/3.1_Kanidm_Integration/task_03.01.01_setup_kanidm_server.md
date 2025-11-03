# Task: Setup Kanidm Server for Identity Management

**Task ID:** V1_MVP/03_User_Service/3.1_Kanidm_Integration/task_03.01.01_setup_kanidm_server.md  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.1_Kanidm_Integration  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2025-11-03  
**Last Updated:** 2025-11-03

## Detailed Description

Deploy and configure Kanidm as the Identity Provider for Anthill platform. Kanidm will handle all user authentication, session management, and OAuth2/OIDC flows.

## Specific Sub-tasks

- [ ] 1. Add Kanidm to `infra/docker_compose/docker-compose.yml`
- [ ] 2. Configure Kanidm environment variables (domain, admin password, TLS)
- [ ] 3. Create persistent volume for Kanidm database
- [ ] 4. Initialize Kanidm and create admin account
- [ ] 5. Configure Kanidm domain and TLS certificates
- [ ] 6. Create OAuth2 client for Anthill application
- [ ] 7. Configure OAuth2 redirect URLs (callback, localhost for dev)
- [ ] 8. Enable PKCE for the OAuth2 client
- [ ] 9. Document Kanidm admin commands and credentials

## Acceptance Criteria

- [ ] Kanidm service running in docker-compose
- [ ] Kanidm accessible at `https://idm.localhost` (dev) or configured domain
- [ ] Admin account created and documented
- [ ] OAuth2 client `anthill` created with:
  - [ ] Client ID and secret generated
  - [ ] PKCE enabled
  - [ ] Redirect URLs configured
  - [ ] Scopes: `openid`, `profile`, `email`, `groups`
- [ ] Can access Kanidm UI and login with admin

## Dependencies

- Docker and docker-compose installed
- TLS certificates (can use self-signed for dev)
- Domain name configured (or localhost)

## Files to Create/Modify

- `infra/docker_compose/docker-compose.yml`
- `infra/docker_compose/kanidm/server.toml` (config file)
- `infra/docker_compose/kanidm/setup.sh` (initialization script)
- `.env.example` (add Kanidm variables)

## Testing Steps

```bash
# 1. Start Kanidm
docker-compose up -d kanidm

# 2. Check Kanidm is running
docker-compose logs kanidm

# 3. Access Kanidm UI
open https://idm.localhost

# 4. Login with admin credentials
# username: admin
# password: (from .env)

# 5. Create OAuth2 client via CLI
docker-compose exec kanidm kanidm system oauth2 create anthill "Anthill Inventory" https://app.localhost
docker-compose exec kanidm kanidm system oauth2 enable-pkce anthill
docker-compose exec kanidm kanidm system oauth2 add-redirect-url anthill https://app.localhost/oauth/callback
docker-compose exec kanidm kanidm system oauth2 add-redirect-url anthill http://localhost:5173/oauth/callback
docker-compose exec kanidm kanidm system oauth2 show-basic-secret anthill
```

## References

- Kanidm Documentation: https://kanidm.github.io/kanidm/
- OAuth2 Setup: https://kanidm.github.io/kanidm/master/integrations/oauth2.html
- Docker Deployment: https://kanidm.github.io/kanidm/master/installing_the_server.html

## Notes

- Use docker image: `kanidm/server:latest` or specific version
- Store OAuth2 client secret securely (use .env, never commit)
- For production, use proper domain and valid TLS certificates
- Consider backup strategy for Kanidm database
