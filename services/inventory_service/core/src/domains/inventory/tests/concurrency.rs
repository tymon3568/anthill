#[cfg(test)]
mod tests {
    use crate::services::distributed_lock::DistributedLockService;
    use crate::Result;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    /// Mock implementation of DistributedLockService for testing core logic
    pub struct MockCoreDistributedLockService {
        locks: Mutex<HashMap<String, String>>,
    }

    impl MockCoreDistributedLockService {
        pub fn new() -> Self {
            Self {
                locks: Mutex::new(HashMap::new()),
            }
        }

        fn lock_key(&self, tenant_id: Uuid, resource_type: &str, resource_id: &str) -> String {
            format!("lock:{}:{}:{}", tenant_id, resource_type, resource_id)
        }
    }

    #[async_trait]
    impl DistributedLockService for MockCoreDistributedLockService {
        async fn acquire_lock(
            &self,
            tenant_id: Uuid,
            resource_type: &str,
            resource_id: &str,
            _ttl_seconds: u32,
        ) -> Result<Option<String>> {
            let key = self.lock_key(tenant_id, resource_type, resource_id);
            let mut locks = self.locks.lock().unwrap();

            if locks.contains_key(&key) {
                return Ok(None);
            }

            let token = Uuid::now_v7().to_string();
            locks.insert(key, token.clone());
            Ok(Some(token))
        }

        async fn release_lock(
            &self,
            tenant_id: Uuid,
            resource_type: &str,
            resource_id: &str,
            lock_token: &str,
        ) -> Result<bool> {
            let key = self.lock_key(tenant_id, resource_type, resource_id);
            let mut locks = self.locks.lock().unwrap();

            if let Some(token) = locks.get(&key) {
                if token == lock_token {
                    locks.remove(&key);
                    return Ok(true);
                }
            }
            Ok(false)
        }

        async fn is_locked(
            &self,
            tenant_id: Uuid,
            resource_type: &str,
            resource_id: &str,
        ) -> Result<bool> {
            let key = self.lock_key(tenant_id, resource_type, resource_id);
            let locks = self.locks.lock().unwrap();
            Ok(locks.contains_key(&key))
        }

        async fn extend_lock(
            &self,
            tenant_id: Uuid,
            resource_type: &str,
            resource_id: &str,
            lock_token: &str,
            _ttl_seconds: u32,
        ) -> Result<bool> {
            let key = self.lock_key(tenant_id, resource_type, resource_id);
            let locks = self.locks.lock().unwrap();

            if let Some(token) = locks.get(&key) {
                if token == lock_token {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        async fn force_release_lock(
            &self,
            tenant_id: Uuid,
            resource_type: &str,
            resource_id: &str,
        ) -> Result<bool> {
            let key = self.lock_key(tenant_id, resource_type, resource_id);
            let mut locks = self.locks.lock().unwrap();
            Ok(locks.remove(&key).is_some())
        }
    }

    // This test verifies that the DistributedLockService mock behaves as expected
    // regarding concurrency control logic, which is critical for inventory operations.
    #[tokio::test]
    async fn test_lock_service_concurrency_behavior() {
        let lock_service = MockCoreDistributedLockService::new();
        let tenant_id = Uuid::new_v4();
        let resource_type = "product";
        let resource_id = "prod-123";

        // 1. Thread A acquires lock
        let token_a = lock_service
            .acquire_lock(tenant_id, resource_type, resource_id, 30)
            .await
            .expect("Failed to acquire lock A");

        assert!(token_a.is_some(), "Thread A should get a lock token");

        // 2. Thread B tries to acquire same lock -> Should fail
        let token_b = lock_service
            .acquire_lock(tenant_id, resource_type, resource_id, 30)
            .await
            .expect("Failed to check lock B");

        assert!(token_b.is_none(), "Thread B should NOT get a lock token");

        // 3. Thread A releases lock
        let released = lock_service
            .release_lock(tenant_id, resource_type, resource_id, &token_a.unwrap())
            .await
            .expect("Failed to release lock A");

        assert!(released, "Lock A should be released");

        // 4. Thread B tries again -> Should succeed
        let token_b_retry = lock_service
            .acquire_lock(tenant_id, resource_type, resource_id, 30)
            .await
            .expect("Failed to acquire lock B retry");

        assert!(token_b_retry.is_some(), "Thread B should now get a lock token");
    }
}
