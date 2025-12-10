/// Shared DTOs for inventory service
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PaginationInfo {
    pub page: u32,
    pub page_size: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationInfo {
    pub fn new(page: u32, page_size: u32, total_items: u64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            total_items.div_ceil(page_size as u64) as u32
        };
        Self {
            page,
            page_size,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_info() {
        let info = PaginationInfo::new(1, 20, 100);
        assert_eq!(info.page, 1);
        assert_eq!(info.page_size, 20);
        assert_eq!(info.total_items, 100);
        assert_eq!(info.total_pages, 5);
        assert!(info.has_next);
        assert!(!info.has_prev);

        let info = PaginationInfo::new(3, 20, 100);
        assert!(info.has_next);
        assert!(info.has_prev);

        let info = PaginationInfo::new(5, 20, 100);
        assert!(!info.has_next);
        assert!(info.has_prev);

        // Edge cases
        let info = PaginationInfo::new(1, 10, 0);
        assert_eq!(info.total_pages, 0);
        assert!(!info.has_next);
        assert!(!info.has_prev);

        let info = PaginationInfo::new(1, 10, 1);
        assert_eq!(info.total_pages, 1);
        assert!(!info.has_next);
        assert!(!info.has_prev);
    }
}
