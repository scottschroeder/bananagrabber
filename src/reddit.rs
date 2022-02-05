use crate::media::Media;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct ApiResponse {
    data: Vec<ApiObject>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum ApiObject {
    Listing(ApiListing),
    #[serde(rename = "t3")]
    Post(PostInfo),
    #[serde(rename = "t1")]
    Comment(CommentInfo),
    #[serde(rename = "more")]
    More(MoreInfo),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiListing {
    children: Vec<ApiObject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoreInfo {}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentInfo {}

#[derive(Debug, Clone, Deserialize)]
pub struct PostInfo {
    subreddit: String,
    title: String,
    is_reddit_media_domain: bool,
    secure_media: Option<RedditMedia>,
    media: Option<RedditMedia>,
    domain: String,
    over_18: bool,
    is_video: bool,
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditMedia {
    reddit_video: Option<RedditVideo>,
    oembed: Option<OEmbedVideo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OEmbedVideo {}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditVideo {
    scrubber_media_url: String,
    fallback_url: String,
    duration: f32,
    is_gif: bool,
}

// TODO Rate limiting
pub struct RedditClient;

impl RedditClient {
    pub async fn get_info(&self, url: &str) -> Result<ApiResponse> {
        let full_url = make_url_json(url);
        log::debug!("url: {:?}", full_url);
        let resp = reqwest::get(full_url).await?.json::<ApiResponse>().await?;
        Ok(resp)
    }
}

pub fn get_post_from_response(resp: &ApiResponse) -> anyhow::Result<&PostInfo> {
    match &resp.data[0] {
        ApiObject::Listing(listing) => match &listing.children[0] {
            ApiObject::Post(post) => Ok(post),
            _ => Err(anyhow::anyhow!("expected post")),
        },
        _ => Err(anyhow::anyhow!("expected listing")),
    }
}

pub fn scan_for_media(post: &PostInfo) -> Result<Option<Media>> {
    if let Some(media) = &post.media {
        if let Some(reddit_video) = &media.reddit_video {
            return Ok(Some(Media {
                url: strip_query_params(&reddit_video.fallback_url)?,
            }));
        }
    }

    if post.domain.starts_with("self.") {
        return Ok(None);
    }

    Ok(Some(Media {
        url: post.url.clone(),
    }))
}

fn strip_query_params(s: &str) -> anyhow::Result<String> {
    let mut x = url::Url::parse(s)?;
    x.query_pairs_mut().clear().finish();

    Ok(x.as_str().to_string())
}

fn make_url_json(s: &str) -> String {
    let mut url = s.to_string();
    url.push_str("/.json");
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_TEXT: &str = include_str!("../sample_responses/empty_text.json");
    const GFYCAT: &str = include_str!("../sample_responses/gfycat.json");
    const IMGUR: &str = include_str!("../sample_responses/imgur.json");
    const IREDDIT: &str = include_str!("../sample_responses/ireddit.json");
    const JGIFS: &str = include_str!("../sample_responses/jgifs.json");
    const TEXT: &str = include_str!("../sample_responses/text.json");
    const TOO_MANY_REQUESTS: &str = include_str!("../sample_responses/too_many_requests.json");
    const VREDDIT: &str = include_str!("../sample_responses/vreddit.json");
    const VREDDIT_PREVIEW: &str = include_str!("../sample_responses/vreddit_preview.json");

    fn check_parse_and_subreddit(json: &str, subreddit: &str) {
        let resp = serde_json::from_str::<ApiResponse>(json).unwrap();
        let post = get_post_from_response(&resp).unwrap();
        assert_eq!(post.subreddit, subreddit);
    }

    fn check_parse_and_media(json: &str, media: Option<&str>) {
        let resp = serde_json::from_str::<ApiResponse>(json).unwrap();
        let post = get_post_from_response(&resp).unwrap();

        let wrapped_media = media.map(|s| Media { url: s.to_string() });
        assert_eq!(scan_for_media(post).unwrap(), wrapped_media);
    }

    #[test]
    fn parse_empty_text() {
        check_parse_and_subreddit(EMPTY_TEXT, "AskReddit");
    }

    #[test]
    fn parse_gfycat() {
        check_parse_and_subreddit(GFYCAT, "SpaceGifs");
    }

    #[test]
    fn parse_imgur() {
        check_parse_and_subreddit(IMGUR, "SpaceGifs");
    }

    #[test]
    fn parse_ireddit() {
        check_parse_and_subreddit(IREDDIT, "interestingasfuck");
    }

    #[test]
    fn parse_jgifs() {
        check_parse_and_subreddit(JGIFS, "SpaceGifs");
    }

    #[test]
    fn parse_text() {
        check_parse_and_subreddit(TEXT, "Jokes");
    }

    #[test]
    fn parse_vreddit() {
        check_parse_and_subreddit(VREDDIT, "gifs");
    }

    #[test]
    fn parse_vreddit_preview() {
        check_parse_and_subreddit(VREDDIT_PREVIEW, "SpaceGifs");
    }

    #[test]
    fn scan_media_empty_text() {
        check_parse_and_media(EMPTY_TEXT, None);
    }

    #[test]
    fn scan_media_gfycat() {
        check_parse_and_media(
            GFYCAT,
            Some("https://gfycat.com/DistinctHonestIaerismetalmark"),
        );
    }

    #[test]
    fn scan_media_imgur() {
        check_parse_and_media(IMGUR, Some("http://i.imgur.com/wSME5Xy.gif"));
    }

    #[test]
    fn scan_media_ireddit() {
        check_parse_and_media(IREDDIT, Some("https://i.redd.it/kaopcso5hqw61.jpg"));
    }

    #[test]
    fn scan_media_jgifs() {
        check_parse_and_media(JGIFS, Some("https://j.gifs.com/m8bLeJ.gif"));
    }

    #[test]
    fn scan_media_text() {
        check_parse_and_media(TEXT, None);
    }

    #[test]
    fn scan_media_vreddit() {
        check_parse_and_media(VREDDIT, Some("https://v.redd.it/6zyfsfjjlxz11/DASH_4_8_M?"));
    }

    #[test]
    fn scan_media_vreddit_preview() {
        check_parse_and_media(
            VREDDIT_PREVIEW,
            Some("https://v.redd.it/u23a45f7pcd81/DASH_720.mp4?"),
        );
    }
}
