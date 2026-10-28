use std::{fs::File, path::Path};

use clap::{Args, Parser, Subcommand};
use kicad_data_model::model::{At, Effects, FontOptions, KiCadSchema, OneOrMulti, Property, Size, Text, YesOrNo};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "rewrite")]
#[command(author = "Hunter Mellema")]
#[command(version = "1.0")]
#[command(about = "Loads, updates, re-writes kicad file", long_about = None)]
struct Cli {
    /// Path to the raw input KiCad source file (.kicad_sch)
    #[arg(short, long, value_name = "FILE")]
    input: String,

    /// Destination path
    #[arg(short, long, value_name = "FILE")]
    output: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Modify text properties within the schematic
    Text(TextArgs),
    Remove(RemoveArgs),
    Add(AddArgs),
}
impl Commands {
    fn run(self, parsed: &mut KiCadSchema) {
        match self {
            Commands::Text(text_args) => text_args.run(parsed),
            Commands::Remove(remove_args) => remove_args.run(parsed),
            Commands::Add(add_args) => add_args.run(parsed),
        }
    }
}

/// Add a text element to the schematic
#[derive(Args, Debug)]
struct TextArgs {
    /// Text to add
    #[arg(value_name = "TEXT")]
    text: String,

    /// Font thickness (weight)
    #[arg(short, long, value_name = "WEIGHT")]
    weight: Option<f32>,

    /// Text size
    #[arg(long, value_name = "SIZE", default_value = "1.27 1.27", value_parser = parse_size_tuple)]
    text_size: (f32, f32),

    /// Alignment (justify) params
    #[arg(long="align", value_name = "ALIGN", num_args(1..))]
    text_justify: Option<Vec<String>>,
}
impl TextArgs {
    fn run(self, parsed: &mut KiCadSchema) {
        eprintln!("Adding Text...");
        let update = Text {
            content: self.text,
            // Todo: Add from and new methods to hide
            at: At::WithRotation(12.7, 13.97, 0.0), // I dunno... just picked something
            effects: Effects {
                font: FontOptions {
                    size: Size(self.text_size.0, self.text_size.1),
                    thickness: self.weight,
                    flags: None,
                },
                justify: self.text_justify.map(OneOrMulti::Multi),
                hide: None
            },
            uuid: Uuid::new_v4()
        };
        parsed.add_text(update);
    }
}

fn parse_size_tuple(s: &str) -> Result<(f32, f32), String> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(format!("Expected exactly two parts, got {}", parts.len()));
    }
    let w = parts[0].parse::<f32>().map_err(|e| format!("Invalid size width: {}", e))?;
    let h = parts[1].parse::<f32>().map_err(|e| format!("Invalid size height: {}", e))?;
    Ok((w, h))
}

/// Remove a property from a symbol
#[derive(Args, Debug)]
struct RemoveArgs {
    /// Symbol
    #[arg(short, long, value_name = "SYMBOL")]
    symbol: String,
    /// Symbol
    #[arg(short, long, value_name = "PROP")]
    property: String,
}
impl RemoveArgs {
    fn run(self, parsed: &mut KiCadSchema) {
        parsed.remove_property(&self.symbol, &self.property);
    }
}

/// Add a property to a symbol
#[derive(Args, Debug)]
struct AddArgs {
    /// Symbol
    #[arg(short, long, value_name = "SYMBOL")]
    symbol: String,
    /// Name
    #[arg(short, long, value_name = "NAME")]
    name: String,
    /// Type
    #[arg(short, long="type", value_name = "TYPE")]
    ty: String,
}
impl AddArgs {
    fn run(self, parsed: &mut KiCadSchema) {
        let property = Property {
            keys: (self.ty, self.name),
            at: At::WithRotation(1.4, 1.4, 0.0),
            hide: None,
            show_name: YesOrNo::Yes,
            do_not_autoplace: YesOrNo::Yes,
            // TODO: allow modification
            effects: Effects {
                font: FontOptions {
                    size: Size(1.5, 1.5),
                    thickness: None,
                    flags: None,
                },
                justify: None,
                hide: None,
            },
        };
        parsed.add_property(&self.symbol, property);
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    eprintln!("Reading Source: {} ...", cli.input);
    let mut parsed = KiCadSchema::load(Path::new(&cli.input))?;

    if let Some(command) = cli.command {
        command.run(&mut parsed);
    }

    eprintln!("Writing to: {} ...", cli.output);
    let mut file_content = File::create(&cli.output)?;
    parsed.write(&mut file_content)?;

    eprintln!("Done!");
    Ok(())
}
