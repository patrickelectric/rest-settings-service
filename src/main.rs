use std::fs::File;
use std::io::prelude::*;

use toml;

mod settings;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Receita {
    quantidade: u8,
    sal: u8,
    oregano: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Batata {
    tipo_de_batata: String,
    frita: bool,
    receita: Receita,
}

fn main() -> std::io::Result<()> {
    let file_name = "foo.json";

    // Open if the file exist, otherwise create it
    let mut file = File::open(file_name).unwrap_or_else(|_| {
        File::create(file_name).unwrap_or_else(|error| {
            panic!("{:#?}", error);
        })
    });

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("> {}", contents);

    let json_value = serde_json::to_value(Batata::default()).unwrap();

    let mut setting = settings::Content::default();
    setting.settings = Some(toml::Value::try_from(json_value).unwrap());
    println!("{}", serde_json::to_string_pretty(&setting).unwrap());
    println!("{}", toml::to_string(&setting).unwrap());
    Ok(())
}
