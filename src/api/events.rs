use crate::client::HexClient;
use crate::error::HexSdkError;
use crate::types::{EventDetail, EventListItem, Tag, TagDetail};

/// Query parameters for listing events.
#[derive(Debug, Default)]
pub struct ListEventsParams {
    pub tag: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl HexClient {
    /// List events (no auth required).
    pub fn list_events(
        &self,
        params: &ListEventsParams,
    ) -> Result<Vec<EventListItem>, HexSdkError> {
        let mut url = self.url("/api/v1/events");
        let mut sep = '?';
        if let Some(ref t) = params.tag {
            url.push_str(&format!("{}tag={}", sep, t));
            sep = '&';
        }
        if let Some(ref s) = params.status {
            url.push_str(&format!("{}status={}", sep, s));
            sep = '&';
        }
        if let Some(l) = params.limit {
            url.push_str(&format!("{}limit={}", sep, l));
            sep = '&';
        }
        if let Some(o) = params.offset {
            url.push_str(&format!("{}offset={}", sep, o));
            let _ = sep;
        }

        self.get(&url)
    }

    /// Get event detail by slug.
    pub fn get_event(&self, slug: &str) -> Result<EventDetail, HexSdkError> {
        self.get(&self.url(&format!("/api/v1/events/{}", slug)))
    }

    /// List top-level tags.
    pub fn list_tags(&self) -> Result<Vec<Tag>, HexSdkError> {
        self.get(&self.url("/api/v1/tags"))
    }

    /// Get a tag by slug with its children.
    pub fn get_tag(&self, slug: &str) -> Result<TagDetail, HexSdkError> {
        self.get(&self.url(&format!("/api/v1/tags/{}", slug)))
    }
}
