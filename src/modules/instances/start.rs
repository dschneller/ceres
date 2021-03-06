use clap::{App, Arg, ArgMatches, SubCommand};

use config::{CeresConfig as Config, Provider};
use run_config::RunConfig;
use modules::*;
use output::OutputType;
use output::instances::{JsonOutputStateChanges, OutputStateChanges, TableOutputStatusChanges};
use provider::{StateChange, StartInstances};
use utils::cli::read_instance_ids;

pub const NAME: &str = "start";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("start instances")
            .arg(
                Arg::with_name("instance_ids")
                    .multiple(true)
                    .required(true)
                    .help("Instance Ids to start; or '-' to read json with instance ids from stdin"),
            )
            .arg(
                Arg::with_name("dry")
                    .long("dry")
                    .short("d")
                    .help("Makes a dry run without actually starting the instances"),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .default_value("human")
                    .possible_values(&["human", "json"])
                    .help("Selects output format"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    info!("Starting instances.");
    let changes = start_instances(args, run_config, config)?;

    info!("Outputting instance state changes.");
    output_changes(args, run_config, config, &changes)?;

    Ok(())
}

fn start_instances(
    args: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
) -> Result<Vec<StateChange>> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let Provider::Aws(provider) = profile.provider
        .as_ref()
        .ok_or(Error::from_kind(ErrorKind::ConfigMissingInProfile("provider".to_string())))?;

    let dry = args.is_present("dry");

    if dry {
        warn!("Running in dry mode -- no changes will be executed.");
    }

    let instance_ids: Vec<&str> = args.values_of("instance_ids").unwrap_or_else(|| Default::default()).collect();
    let instance_ids: Vec<_> = read_instance_ids(&instance_ids)
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;

    provider
        .start_instances(dry, &instance_ids)
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}

fn output_changes(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    state_changes: &[StateChange],
) -> Result<()> {
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let output = TableOutputStatusChanges {};

            output
                .output(&mut stdout, state_changes)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Json => {
            let output = JsonOutputStateChanges;

            output
                .output(&mut stdout, state_changes)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Plain => {
            unimplemented!("'Plain' output is not supported for this module");
        }
    }
}
