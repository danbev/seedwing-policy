use crate::{
    cli::{Context, InputType},
    util::{self, load_values},
};
use seedwing_policy_engine::{lang::Severity, runtime::Response};
use serde_view::View;
use std::{path::PathBuf, process::ExitCode};

#[derive(clap::Args, Debug)]
#[command(
    about = "Evaluate a pattern against an input",
    args_conflicts_with_subcommands = true
)]
pub struct Eval {
    #[arg(short='t', value_name = "TYPE", value_enum, default_value_t=InputType::Json)]
    typ: InputType,
    #[arg(short, long)]
    input: Option<PathBuf>,
    #[arg(short = 'n', long = "name")]
    name: Option<String>,
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    verbose: bool,
    #[arg(
        short = 's',
        long = "select",
        help = "comma-delimited list of 'name,bindings,input,output,severity,reason,rationale'",
        default_value_t = String::from("name,severity,reason,rationale")
    )]
    select: String,
}

impl Eval {
    pub async fn run(&self, context: Context) -> anyhow::Result<ExitCode> {
        let world = context.world().await?.1;

        let inputs: Vec<PathBuf> = if let Some(input) = &self.input {
            vec![input.clone()]
        } else {
            context.inputs.clone()
        };

        let names = if let Some(name) = &self.name {
            vec![name.clone()]
        } else {
            if context.required_policies.is_empty() {
                eprintln!("No policies specified on command line (-n) or in config file");
                return Ok(ExitCode::FAILURE);
            }
            context.required_policies.clone()
        };

        // Load from config

        let values = load_values(self.typ, inputs).await?;
        for value in values {
            for name in names.iter() {
                let eval = util::eval::Eval::new(&world, name, value.clone());

                let result = eval.run().await?;
                let mut response = Response::new(&result);
                if self.verbose {
                    response = response.collapse(Severity::Error);
                }

                let response = response.as_view().with_fields(self.select.split(","));

                println!("{}", serde_json::to_string_pretty(&response).unwrap());
                if result.severity() >= Severity::Error {
                    return Ok(ExitCode::from(2));
                }
            }
        }

        Ok(ExitCode::SUCCESS)
    }
}
