use anyhow::Context;
use argparse::CliOpts;

mod argparse;
mod media;
mod reddit;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_backtrace::install();
    let args = argparse::get_args();
    setup_logger(args.verbose);
    log::trace!("Args: {:?}", args);

    run(&args).await.map_err(|e| {
        log::error!("{}", e);
        e.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));
        anyhow::anyhow!("unrecoverable bananagrabber failure")
    })
}

async fn run(args: &CliOpts) -> anyhow::Result<()> {
    match &args.subcmd {
        argparse::SubCommand::ExtractMediaUrl(opts) => fetch_url(opts).await,
        argparse::SubCommand::Test(opts) => scratch(opts),
    }
}

fn scratch(opts: &argparse::Test) -> anyhow::Result<()> {
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

async fn fetch_url(opts: &argparse::ExtractMediaUrl) -> anyhow::Result<()> {
    let client = reddit::RedditClient;
    let resp = client.get_info(&opts.url).await?;
    let post = reddit::get_post_from_response(&resp)?;
    log::debug!("{:#?}", post);
    let media = reddit::scan_for_media(post)?;
    match media {
        Some(m) => println!("{}", m.url),
        None => log::warn!("could not find media"),
    }
    Ok(())
}

pub fn setup_logger(level: u8) {
    let mut builder = pretty_env_logger::formatted_timed_builder();

    let noisy_modules = &[
        "hyper",
        "mio",
        "tokio_core",
        "tokio_reactor",
        "tokio_threadpool",
        "fuse::request",
        "rusoto_core",
        "want",
        "tantivy",
    ];

    let log_level = match level {
        //0 => log::Level::Error,
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    if level > 1 && level < 4 {
        for module in noisy_modules {
            builder.filter_module(module, log::LevelFilter::Info);
        }
    }

    builder.filter_level(log_level);
    builder.format_timestamp_millis();
    //builder.format(|buf, record| writeln!(buf, "{}", record.args()));
    builder.init();
}
