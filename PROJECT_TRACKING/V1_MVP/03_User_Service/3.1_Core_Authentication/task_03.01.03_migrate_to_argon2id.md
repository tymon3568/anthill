# Task: Migrate Password Hashing to Argon2id

**Task ID:** V1_MVP/03_User_Service/3.1_Core_Authentication/task_03.01.03_migrate_to_argon2id.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.1_Core_Authentication
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2025-10-21
**Last Updated:** 2025-10-21

## Detailed Description:
Migrate the password hashing algorithm from `bcrypt` to `Argon2id` for better security against modern hardware-based attacks. Argon2id is the recommended standard by OWASP.

## Specific Sub-tasks:
- [ ] 1. Add the `argon2` crate to `user_service_infra/Cargo.toml`.
- [ ] 2. Create a new password hashing service/module that uses `argon2`.
- [ ] 3. Update the registration and password change logic to use the new Argon2id hashing function.
- [ ] 4. Modify the login logic to handle both `bcrypt` and `Argon2id` hashes. If a user logs in with a `bcrypt` hash, verify it, and if successful, re-hash their password with `Argon2id` and update the database (lazy migration).

## Acceptance Criteria:
- [ ] The `argon2` crate is added as a dependency.
- [ ] Password hashing logic is updated to use Argon2id.
- [ ] New users are registered with Argon2id-hashed passwords.
- [ ] The login endpoint can still verify old bcrypt hashes and, upon success, re-hash the password with Argon2id.
- [ ] All relevant tests are updated and pass.

## Dependencies:
*   (Sẽ được cập nhật khi cần)

## Related Documents:
*   `user_service/infra/src/auth/password.rs` (or similar)

## Notes / Discussion:
---
*   (Khu vực dành cho các câu hỏi, thảo luận, hoặc ghi chú trong quá trình thực hiện)

## AI Agent Log:
---
*   (Log sẽ được AI agent tự động cập nhật khi bắt đầu và thực hiện task)
