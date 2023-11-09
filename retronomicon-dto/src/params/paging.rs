use serde::{Deserialize, Serialize};

pub const PAGE_DEFAULT: i64 = 0;
pub const PAGE_MIN: i64 = 0;
pub const PAGE_MAX: i64 = i64::MAX;
pub const LIMIT_DEFAULT: i64 = 20;
pub const LIMIT_MIN: i64 = 10;
pub const LIMIT_MAX: i64 = 100;

/// Parameters for paging through a list of items.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct PagingParams {
    /// The page index to retrieve. The first page is 0. This will
    /// multiply by the limit to get the actual item offset.
    /// Defaults to 0.
    pub page: Option<i64>,

    /// The maximum number of items to retrieve. Must be between 10
    /// and 100. Defaults to 20.
    pub limit: Option<i64>,
}

impl PagingParams {
    pub fn validate(&self) -> Result<(i64, i64), String> {
        let page = self.page.unwrap_or(PAGE_DEFAULT);
        let limit = self.limit.unwrap_or(LIMIT_DEFAULT);

        if page < PAGE_MIN {
            Err(format!("Page must be greater than or equal to {PAGE_MIN}"))
        } else if limit < LIMIT_MIN {
            Err(format!(
                "Limit must be greater than or equal to {LIMIT_MIN}"
            ))
        } else if limit > 100 {
            Err(format!("Limit must be less than or equal to {LIMIT_MAX}"))
        } else {
            Ok((page, limit))
        }
    }
}