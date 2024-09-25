use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct Illustration {
    pub id: u32,
    pub tags: Vec<Tag>,
    pub visible: bool,
    pub r#type: String,
    pub title: String,
    pub caption: String,
    pub height: u32,
    pub width: u32,
    pub page_count: i32,
    pub user: User,
    pub tools: Vec<String>,
    pub series: Option<String>,
    pub restrict: i32,
    pub x_restrict: i32,
    pub image_urls: IllustrationURLs,
    pub meta_single_page: Option<IllustrationMetaURL>,
    pub meta_pages: Vec<IllustrationURLsWrapper>,
    pub total_view: u32,
    pub total_bookmarks: u32,
    pub is_bookmarked: bool,
    pub is_muted: bool,
    pub total_comments: u32,
    pub illust_ai_type: u32,
    pub illust_book_style: u32,
    pub comment_access_control: u32,
    pub create_date: String,
}

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct User {
    id: i32,
    name: String,
    account: String,
    profile_image_urls: UserProfileImages,
    is_followed: bool,
}

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct UserProfileImages {
    pub medium: String,
}

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct IllustrationURLsWrapper {
    pub image_urls: IllustrationURLs,
}

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct IllustrationMetaURL {
    pub original_image_url: Option<String>,
}

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct IllustrationURLs {
    pub square_medium: String,
    pub medium: String,
    pub large: String,
    pub original: Option<String>,
}

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct Tag {
    pub name: String,
    pub translated_name: Option<String>,
}

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct PixivResult {
    pub illust: Option<Illustration>,
    pub illusts: Option<Vec<Illustration>>,
    pub error: Option<PixivError>,
}

#[derive(Serialize, Deserialize, std::fmt::Debug)]
pub struct PixivError {
    user_message: String,
    message: String,
    reason: String,
}

impl std::error::Error for PixivError {}
impl std::fmt::Display for PixivError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.user_message, self.reason)
    }
}
