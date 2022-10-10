use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};

pub const ENDPOINT: &str = "https://safebooru.donmai.us";

pub async fn posts<'a, T: Into<Option<&'a str>>, L: Into<Option<usize>>, R: Into<Option<bool>>>(
    tags: T,
    limit: L,
    random: R,
) -> Result<Posts, reqwest::Error> {
    let url = format!(
        "{}/posts.json?tags={}&limit={}&random={}",
        ENDPOINT,
        utf8_percent_encode(tags.into().unwrap_or_default(), NON_ALPHANUMERIC),
        limit.into().unwrap_or(100),
        random.into().unwrap_or(false)
    );
    log::debug!("Fetching {}", url);

    reqwest::get(url).await?.json::<Posts>().await
}

pub type Posts = Vec<Post>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: Option<i64>,
    // pub created_at: String,
    // pub uploader_id: i64,
    // pub score: i64,
    // pub source: String,
    // pub md5: Option<String>,
    // pub last_comment_bumped_at: Option<String>,
    // pub rating: Rating,
    // pub image_width: i64,
    // pub image_height: i64,
    // pub tag_string: String,
    // pub fav_count: i64,
    pub file_ext: Option<FileExt>,
    // pub last_noted_at: Option<String>,
    // pub parent_id: Option<i64>,
    // pub has_children: bool,
    // pub approver_id: Option<i64>,
    // pub tag_count_general: i64,
    // pub tag_count_artist: i64,
    // pub tag_count_character: i64,
    // pub tag_count_copyright: i64,
    // pub file_size: i64,
    // pub up_score: i64,
    // pub down_score: i64,
    // pub is_pending: bool,
    // pub is_flagged: bool,
    // pub is_deleted: bool,
    // pub tag_count: i64,
    // pub updated_at: String,
    // pub is_banned: bool,
    // pub pixiv_id: Option<i64>,
    // pub last_commented_at: Option<String>,
    // pub has_active_children: bool,
    // pub bit_flags: i64,
    // pub tag_count_meta: i64,
    // pub has_large: Option<bool>,
    // pub has_visible_children: bool,
    // pub tag_string_general: String,
    // pub tag_string_character: String,
    // pub tag_string_copyright: String,
    // pub tag_string_artist: String,
    // pub tag_string_meta: String,
    // pub file_url: Option<String>,
    pub large_file_url: Option<String>,
    // pub preview_file_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileExt {
    #[serde(rename = "jpg")]
    Jpg,
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "webp")]
    Webp,
    // We're only interested in the above filetypes
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Rating {
    #[serde(rename = "g")]
    General,
}
