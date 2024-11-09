use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug)]
struct Link {
    title: String,
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct Subcategory {
    title: String,
    links: Vec<Link>,
}

#[derive(Deserialize, Debug)]
pub struct Category {
    title: String,
    subcategories: Vec<Subcategory>,
}

pub fn read_awesome_json() -> Result<Vec<Category>, Box<dyn std::error::Error>> {
    let mut file = File::open("awesome.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let categories: Vec<Category> = serde_json::from_str(&contents)?;
    Ok(categories)
}

impl Category {
    pub fn get_subcategories(&self) -> &Vec<Subcategory> {
        &self.subcategories
    }
}

fn main() {
    match read_awesome_json() {
        Ok(categories) => println!("{:?}", categories),
        Err(e) => eprintln!("Error reading JSON: {}", e),
    }
}
