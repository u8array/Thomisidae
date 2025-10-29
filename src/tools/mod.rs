pub mod fetch_links;
// legacy file-based module remains, route new code through fetch_text_new
pub mod fetch_text;
pub mod meta;
pub mod utils;
pub mod google_search;
pub mod robots;
pub mod policy;

pub use fetch_links::FetchLinksHandler;
pub use fetch_text::FetchTextHandler;
pub use google_search::GoogleSearchHandler;
pub use meta::{ToolMeta, ToolsMeta};
pub use robots::Robots;
pub use policy::DomainPolicy;
pub use fetch_links::meta as fetch_links_meta;
pub use fetch_text::meta as fetch_text_meta;
pub use google_search::meta as google_search_meta;
