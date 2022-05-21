use crate::argparse;
use crate::reddit;
use crate::reddit::ApiResponse;
use crate::reddit::PostMediaSource;
use anyhow::Context;

const CROSS_POST_RETRIES: usize = 10;

pub fn check_saved_responses(opts: &argparse::Test) -> anyhow::Result<()> {
    {
        use std::fs;
        for entry in fs::read_dir(&opts.file)? {
            let entry = entry?;
            log::debug!("begin {:?}", entry.path());
            let f = fs::File::open(&entry.path())?;
            let resp: anyhow::Result<reddit::ApiResponse> =
                serde_json::from_reader(f).context("deserialize api response");
            match resp {
                Ok(r) => {
                    let post = reddit::get_post_from_response(&r);
                    log::debug!("{:?}: {:#?}", entry.path(), post);
                    if let Ok(p) = post {
                        let media = reddit::scan_for_media(p);
                        log::info!("Media({:?}): {:?}", entry.path(), media);
                    }
                }
                Err(e) => {
                    log::warn!("{:#?}", e);
                }
            }
        }
    }
    Ok(())
}

pub async fn fetch_url(opts: &argparse::ExtractMediaUrl) -> anyhow::Result<()> {
    let media = fetch_url_through_cross_posts(&opts.url).await?;
    match media {
        Some(m) => println!("{}", m),
        None => log::warn!("could not find media"),
    }
    Ok(())
}

pub async fn fetch_url_through_cross_posts(url: &str) -> anyhow::Result<Option<String>> {
    let mut xpost_retries = 0;
    let mut url = url.to_string();
    while xpost_retries < CROSS_POST_RETRIES {
        match fetch_and_extract_source(&url).await? {
            Some(PostMediaSource::Media(m)) => return Ok(Some(m.url)),
            Some(PostMediaSource::CrossPost(u)) => url = u,
            None => return Ok(None),
        }
        xpost_retries += 1;
    }
    Err(anyhow::anyhow!(
        "could not get media after {} cross posts",
        xpost_retries
    ))
}

pub async fn save_url(opts: &argparse::FetchTestCase) -> anyhow::Result<()> {
    let client = reddit::RedditClient;
    let resp = client.get_url_as::<serde_json::Value>(&opts.url).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn fetch_and_extract_source(url: &str) -> anyhow::Result<Option<PostMediaSource>> {
    let resp = fetch_url_str(url).await?;
    extract_media_from_respsonse(&resp)
}

fn extract_media_from_respsonse(resp: &ApiResponse) -> anyhow::Result<Option<PostMediaSource>> {
    let post = reddit::get_post_from_response(&resp)?;
    log::debug!("{:#?}", post);
    reddit::scan_for_media(post)
}

pub async fn fetch_url_str(url: &str) -> anyhow::Result<ApiResponse> {
    let client = reddit::RedditClient;
    let resp = client.get_info(url).await?;
    Ok(resp)
}
