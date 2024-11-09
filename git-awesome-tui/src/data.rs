use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug, Clone)]
pub struct Link {
    pub title: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Subcategory {
    pub title: String,
    pub links: Vec<Link>,
}

#[derive(Deserialize, Debug)]
pub struct Category {
    pub title: String,
    pub subcategories: Option<Vec<Subcategory>>, // Allows categories without subcategories
    pub direct_links: Option<Vec<Link>>,         // Direct links when no subcategories exist
}

pub fn read_awesome_json() -> Result<Vec<Category>, Box<dyn std::error::Error>> {
    let mut file = File::open("awesome.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let categories: Vec<Category> = serde_json::from_str(&contents)?;
    Ok(categories)
}

impl Category {
    pub fn get_all_links(&self) -> Vec<Link> {
        let mut all_links = Vec::new();

        // Collect links from subcategories, if present
        if let Some(subcategories) = &self.subcategories {
            for subcategory in subcategories {
                all_links.extend(subcategory.links.clone());
            }
        }

        // Collect links from direct_links, if present
        if let Some(direct_links) = &self.direct_links {
            all_links.extend(direct_links.clone());
        }

        all_links
    }
}

fn main() {
    match read_awesome_json() {
        Ok(categories) => {
            for category in categories {
                println!("Category: {}", category.title);

                // Print links from subcategories
                if let Some(subcategories) = &category.subcategories {
                    for subcategory in subcategories {
                        println!("  Subcategory: {}", subcategory.title);
                        for link in &subcategory.links {
                            println!("    Link: {} - {}", link.title, link.url);
                        }
                    }
                }

                // Print direct links if available
                if let Some(direct_links) = &category.direct_links {
                    for link in direct_links {
                        println!("  Direct Link: {} - {}", link.title, link.url);
                    }
                }
            }
        }
        Err(e) => eprintln!("Error reading JSON: {}", e),
    }
}
