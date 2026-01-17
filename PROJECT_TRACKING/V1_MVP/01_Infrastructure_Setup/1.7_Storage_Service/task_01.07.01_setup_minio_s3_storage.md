# Task: Setup MinIO S3 Storage Service

**Task ID:** V1_MVP/01_Infrastructure_Setup/1.7_Storage_Service/task_01.07.01_setup_minio_s3_storage.md
**Version:** V1_MVP
**Phase:** 01_Infrastructure_Setup
**Module:** 1.7_Storage_Service
**Priority:** High
**Status:** Done
**Assignee:** Claude 
**Created Date:** 2025-10-28
**Last Updated:** 2025-10-28

## Detailed Description:
Integrate and configure a MinIO service to provide S3-compatible object storage for the application. This service will be used for storing user-generated content, such as profile avatars.

## Specific Sub-tasks:
- [x] 1. Add MinIO service definition to `infra/docker-compose.yml`.
- [x] 2. Configure environment variables for MinIO (e.g., `MINIO_ROOT_USER`, `MINIO_ROOT_PASSWORD`).
- [x] 3. Define a volume for persistent storage.
- [x] 4. Expose the MinIO port (e.g., 9000 for API, 9001 for console).
- [ ] 5. Create a startup script or use a MinIO client (`mc`) in an entrypoint to automatically create the default 'avatars' bucket.
- [ ] 6. Update environment variable templates (`.env.example`) with S3/MinIO connection details (endpoint, access key, secret key, region).
- [ ] 7. Document the new service and its configuration in `ARCHITECTURE.md`.

## Acceptance Criteria:
- [ ] MinIO service starts correctly with `docker-compose up`.
- [ ] The service data persists across container restarts.
- [ ] The 'avatars' bucket is created automatically on startup.
- [ ] The MinIO console is accessible via its exposed port.
- [ ] Connection details are clearly documented and available in environment files.

## Dependencies:
- None

## Related Documents:
- `infra/docker-compose.yml`
- `.env.example`
- `ARCHITECTURE.md`

## Notes / Discussion:
---
*   Ensure the default bucket policy is configured appropriately (e.g., public-read for avatars).
*   The setup should be idempotent, meaning it can be run multiple times without causing errors.

## AI Agent Log:
---
*   2025-10-28 15:56: Cascade created this task as a dependency for implementing avatar uploads.
