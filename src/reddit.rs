use crate::media::Media;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

const REDIRECTS: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ApiResponse {
    data: Vec<ApiObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiListing {
    children: Vec<ApiObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoreInfo {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentInfo {}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditMedia {
    reddit_video: Option<RedditVideo>,
    oembed: Option<OEmbedVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OEmbedVideo {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditVideo {
    scrubber_media_url: String,
    fallback_url: String,
    duration: f32,
    is_gif: bool,
}

#[derive(Debug, PartialEq)]
pub enum PostMediaSource {
    Media(Media),
    CrossPost(String),
}

// TODO Rate limiting
pub struct RedditClient;

impl RedditClient {
    pub async fn get_info(&self, url: &str) -> Result<ApiResponse> {
        self.get_url_as(url).await
    }

    pub async fn get_url_as<T: DeserializeOwned + std::fmt::Debug>(&self, url: &str) -> Result<T> {
        let mut redirect_count = 0;
        let mut full_url = make_url_json(url)?;

        while redirect_count < REDIRECTS {
            redirect_count += 1;
            log::debug!("url: {:?}", full_url.as_str());
            let policy = if is_json(&full_url) {
                reqwest::redirect::Policy::default()
            } else {
                reqwest::redirect::Policy::none()
            };
            let http_client = reqwest::ClientBuilder::new().redirect(policy).build()?;
            let resp = http_client.get(full_url).send().await?;

            if resp.status().is_redirection() {
                let new_loc = resp.headers().get("location").ok_or_else(|| {
                    anyhow::anyhow!("redirect did not provide new location: {:?}", resp)
                })?;
                full_url = make_url_json(new_loc.to_str()?)?;
            } else {
                return Ok(resp.json::<T>().await?);
            }
        }
        Err(anyhow::anyhow!(
            "could not resolve reddit url after {} redirects",
            redirect_count
        ))
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

pub fn scan_for_media(post: &PostInfo) -> Result<Option<PostMediaSource>> {
    if let Some(media) = &post.media {
        if let Some(reddit_video) = &media.reddit_video {
            return Ok(Some(PostMediaSource::Media(Media {
                url: strip_query_params(&reddit_video.fallback_url)?,
            })));
        }
    }

    if post.domain.starts_with("self.") {
        return Ok(None);
    }

    let url = post.url.clone();
    let source = if is_reddit_short_url(&url) {
        PostMediaSource::CrossPost(url)
    } else {
        PostMediaSource::Media(Media {
            url: post.url.clone(),
        })
    };

    Ok(Some(source))
}

fn strip_query_params(s: &str) -> anyhow::Result<String> {
    let mut x = url::Url::parse(s)?;
    x.query_pairs_mut().clear().finish();

    Ok(x.as_str().to_string())
}

fn is_json(url: &reqwest::Url) -> bool {
    url.as_str().ends_with(".json")
}

fn make_url_json(s: &str) -> anyhow::Result<reqwest::Url> {
    let mut s = s.to_string();
    if !is_reddit_short_url(&s) {
        s.push_str("/.json");
    }
    let url = reqwest::Url::parse(&s)?;
    Ok(url)
}

/// Check if a url is a short-link to a post
fn is_reddit_short_url(text: &str) -> bool {
    lazy_static::lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(
            r##"^https://v\.redd\.it/[a-z0-9]+$"##
        ).unwrap();
    }
    RE.is_match(text)
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
    const CROSS_POST: &str = include_str!("../sample_responses/cross_post.json");

    fn check_parse_and_subreddit(json: &str, subreddit: &str) {
        let resp = serde_json::from_str::<ApiResponse>(json).unwrap();
        let post = get_post_from_response(&resp).unwrap();
        assert_eq!(post.subreddit, subreddit);
    }

    fn check_parse_and_media_source(json: &str, media: Option<PostMediaSource>) {
        let resp = serde_json::from_str::<ApiResponse>(json).unwrap();
        let post = get_post_from_response(&resp).unwrap();

        assert_eq!(scan_for_media(post).unwrap(), media);
    }

    fn check_parse_and_media(json: &str, media: Option<&str>) {
        let wrapped_media = media.map(|s| PostMediaSource::Media(Media { url: s.to_string() }));
        check_parse_and_media_source(json, wrapped_media)
    }

    fn check_parse_and_crosspost(json: &str, xpost: &str) {
        let wrapped_media = PostMediaSource::CrossPost(xpost.to_string());
        check_parse_and_media_source(json, Some(wrapped_media))
    }

    fn check_url_jsonify(input: &str, expected: &str) {
        assert_eq!(make_url_json(input).unwrap().as_str(), expected)
    }

    #[test]
    fn make_url_from_reddit_url() {
        check_url_jsonify(
            "https://www.reddit.com/video/yub5uok42jq81",
            "https://www.reddit.com/video/yub5uok42jq81/.json",
        )
    }

    #[test]
    fn make_url_from_short_reddit_url() {
        check_url_jsonify(
            "https://v.redd.it/yub5uok42jq81",
            "https://v.redd.it/yub5uok42jq81",
        )
    }

    #[test]
    fn check_not_short_reddit() {
        assert!(!is_reddit_short_url(
            "https://www.reddit.com/video/yub5uok42jq81"
        ),);
    }
    #[test]
    fn check_is_short_reddit() {
        assert!(is_reddit_short_url("https://v.redd.it/yub5uok42jq81"));
    }
    #[test]
    fn check_isnt_short_reddit_with_video() {
        assert!(!is_reddit_short_url(
            "https://v.redd.it/u23a45f7pcd81/DASH_720.mp4?"
        ));
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
    fn scan_media_cross_post() {
        check_parse_and_crosspost(CROSS_POST, "https://v.redd.it/dkczbt15n2r71");
    }

    #[test]
    fn scan_media_vreddit_preview() {
        check_parse_and_media(
            VREDDIT_PREVIEW,
            Some("https://v.redd.it/u23a45f7pcd81/DASH_720.mp4?"),
        );
    }
}
