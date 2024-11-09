use crate::models::Data;
use crate::search;

pub fn fuzzy_search(data: &Data, _query: &str) -> Vec<String> {
    let mut results = Vec::new();
    for category in &data.categories {
        for subcategory in &category.subcategories {
            let matches = search::fuzzy_search(data, &subcategory.title);
            if matches.is_empty() {
                continue;
            }
            results.push(subcategory.title.clone());
        }
    }
    results
}
