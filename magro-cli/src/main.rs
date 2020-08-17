//! Command to manage git repositories.

use structopt::StructOpt;

use self::cli_opt::Opt;

pub(crate) mod cli_opt;
pub(crate) mod clone;
pub(crate) mod collection;
pub(crate) mod list;
pub(crate) mod refresh;

fn main() -> anyhow::Result<()> {
    init_logger();

    magro::context::create_default_config_file_if_missing()?;
    let ctx = magro::Context::new(None)?;
    let opt = Opt::from_args();
    opt.run(&ctx)?;

    Ok(())
}

/// Initialize logger.
fn init_logger() {
    /// Default log filter for debug build.
    #[cfg(debug_assertions)]
    const DEFAULT_LOG_FILTER: &str = "magro=debug";
    /// Default log filter for release build.
    #[cfg(not(debug_assertions))]
    const DEFAULT_LOG_FILTER: &str = "magro=warn";

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_FILTER))
        .init();
}
