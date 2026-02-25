/// Paginated response containing both items and page information
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PaginatedResponse<T> {
    /// Items for the current page
    pub items: Vec<T>,
    /// Page information
    pub page_info: PageInfo,
}

/// Page information returned with paginated results
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PageInfo {
    /// Current page number (1-based)
    pub page: u64,
    /// Number of items per page
    pub page_size: u64,
    /// Total number of items across all pages
    pub total_items: u64,
    /// Total number of pages
    pub total_pages: u64,
}

impl<T> PaginatedResponse<T> {
    /// Creates a new `PaginatedResponse` with the given items and page information
    pub fn new(items: Vec<T>, page: u64, page_size: u64, total_items: u64) -> Self {
        let total_pages = (total_items + page_size - 1) / page_size;
        let page_info = PageInfo {
            page,
            page_size,
            total_items,
            total_pages,
        };
        PaginatedResponse { items, page_info }
    }
}
