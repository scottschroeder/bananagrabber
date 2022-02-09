use anyhow::Context;
use crate::argparse;
use crate::reddit;

pub fn check_saved_responses(opts: &argparse::Test) -> anyhow::Result<()> {
    log::info!("scratch");
    {
        use std::fs;
        for entry in fs::read_dir(&opts.file)? {
            let entry = entry?;
            log::info!("begin {:?}", entry.path());
            let f = fs::File::open(&entry.path())?;
            let resp: anyhow::Result<reddit::ApiResponse> =
                serde_json::from_reader(f).context("deserialize api response");
            match resp {
                Ok(r) => {
                    let post = reddit::get_post_from_response(&r);
                    log::info!("{:#?}", post);
                    if let Ok(p) = post {
                        let media = reddit::scan_for_media(p);
                        log::info!("Media: {:#?}", media);
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
    let media = fetch_url_str(&opts.url).await?;
    match media {
        Some(m) => println!("{}", m),
        None => log::warn!("could not find media"),
    }
    Ok(())
}

pub async fn fetch_url_str(url: &str) -> anyhow::Result<Option<String>> {
    let client = reddit::RedditClient;
    let resp = client.get_info(url).await?;
    let post = reddit::get_post_from_response(&resp)?;
    log::debug!("{:#?}", post);
    let media = reddit::scan_for_media(post)?;
    match media {
        Some(m) => Ok(Some(m.url)),
        None => Ok(None),
    }
}
