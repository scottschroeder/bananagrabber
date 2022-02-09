use clap::Parser;

pub fn get_args() -> CliOpts {
    CliOpts::parse()
}

#[derive(Parser, Debug)]
#[clap(version = clap::crate_version!(), author = "Scott S. <scottschroeder@sent.com>")]
pub struct CliOpts {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    ExtractMediaUrl(ExtractMediaUrl),
    Bot(BotSettings),
    Test(Test),
}

#[derive(Parser, Debug)]
pub struct ExtractMediaUrl {
    /// url to the reddit post
    pub url: String,
}

#[derive(Parser, Debug)]
pub struct BotSettings {
}

#[derive(Parser, Debug)]
pub struct Test {
    pub file: String,
}
