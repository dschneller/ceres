use clap::{App, Arg, ArgMatches, SubCommand};
use service_world::consul::{Consul, Catalog};

use config::CeresConfig as Config;
use run_config::RunConfig;
use modules::*;
use modules::consul::NodeField;
use output::OutputType;
use output::consul::{JsonOutputCatalogResult, OutputCatalogResult, PlainOutputCatalogResult, TableOutputCatalogResult};

pub const NAME: &str = "list";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Query Consul")
            .arg(
                Arg::with_name("services")
                    .long("services")
                    .short("s")
                    .takes_value(true)
                    .multiple(true)
                    .value_delimiter(",")
                    .help("Filters services for specific service names"),
            )
            .arg(
                Arg::with_name("tags")
                    .long("tags")
                    .short("t")
                    .takes_value(true)
                    .multiple(true)
                    .value_delimiter(",")
                    .help("Filters services for specific tags"),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .default_value("human")
                    .possible_values(&["human", "json", "plain"])
                    .help("Selects output format"),
            )
            .arg(
                Arg::with_name("output-options")
                    .long("output-options")
                    .takes_value(true)
                    .default_value("Name,Address,MetaData:ec2_instance_id,ServicePort,ServiceTags,ServiceName,Healthy")
                    .help("Selects the nodes description fields for human and plain output"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    let services = args.values_of_lossy("services");
    let tags = args.values_of_lossy("tags");
    let url = profile.consul
        .as_ref()
        .ok_or(Error::from_kind(ErrorKind::ConfigMissingInProfile("consul".to_string())))?
        .urls
        .first()
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;

    info!("Quering for services = {}, tags = {}",
        services.as_ref().map(|x| x.join(",")).unwrap_or_else(|| "()".to_owned()),
        tags.as_ref().map(|x| x.join(",")).unwrap_or_else(|| "()".to_owned())
    );
    let catalog = query_consul(url.to_string(), services, tags)
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;

    info!("Outputting catalog");
    output_instances(args, run_config, config, &catalog)?;

    Ok(())
}

fn query_consul(url: String, services: Option<Vec<String>>, tags: Option<Vec<String>>) -> Result<Catalog> {
    let consul = Consul::new(url);
    let catalog = consul.catalog_by(services, tags)
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;

    Ok(catalog)
}

fn output_instances(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    catalog: &Catalog,
) -> Result<()> {
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let opts = args.value_of("output-options").unwrap(); // Safe unwrap
            let output = human_output(opts)?;

            output
                .output(&mut stdout, catalog)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Json => {
            let output = JsonOutputCatalogResult;

            output
                .output(&mut stdout, catalog)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Plain => {
            let opts = args.value_of("output-options").unwrap(); // Safe unwrap
            let output = plain_output(opts)?;

            output
                .output(&mut stdout, catalog)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },    }
}

fn human_output(output_opts: &str) -> Result<TableOutputCatalogResult> {
    let output = if output_opts.contains("all") {
        Default::default()
    } else {
        TableOutputCatalogResult { fields: output_fields(output_opts)? }
    };
    trace!("output = {:?}", output.fields);

    Ok(output)
}

fn plain_output(output_opts: &str) -> Result<PlainOutputCatalogResult> {
    let output = if output_opts.contains("all") {
        Default::default()
    } else {
        PlainOutputCatalogResult { fields: output_fields(output_opts)? }
    };
    trace!("output = {:?}", output.fields);

    Ok(output)
}

fn output_fields(field_str: &str) -> Result<Vec<NodeField>> {
    let fields: ::std::result::Result<Vec<_>, _> = field_str
        .split(',')
        .map(|s| s.parse::<NodeField>())
        .collect();
    let fields =
        fields.map_err(|e| Error::with_chain(e, ErrorKind::ModuleFailed(NAME.to_owned())))?;

    Ok(fields)
}
