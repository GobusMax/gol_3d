use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Read the rule from a file
    #[arg(short, long)]
    pub file: Option<String>,

    /// The size of the initial cube
    #[arg(short = 's', long)]
    pub init_size : Option<usize>,

    /// The density of the intial cube
    #[arg(short = 'd', long)]
    pub init_density : Option<f64>,

    /// Pass in the rule directly
    pub rule: Option<String>,
}
