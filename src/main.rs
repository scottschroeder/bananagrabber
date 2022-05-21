use argparse::CliOpts;

mod argparse;
mod bot;
mod media;
mod media_extraction;
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
        argparse::SubCommand::ExtractMediaUrl(opts) => media_extraction::fetch_url(opts).await,
        argparse::SubCommand::FetchTestCase(opts) => media_extraction::save_url(opts).await,
        argparse::SubCommand::Test(opts) => media_extraction::check_saved_responses(opts),
        argparse::SubCommand::Bot(opts) => bot::bot_start().await,
    }
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
        "tracing::span",
        "serenity::http::client",
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
            builder.filter_module(module, log::LevelFilter::Warn);
        }
    }

    builder.filter_level(log_level);
    builder.format_timestamp_millis();
    //builder.format(|buf, record| writeln!(buf, "{}", record.args()));
    builder.init();
}
