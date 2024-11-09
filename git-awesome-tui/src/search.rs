use crate::data::{Category, Link, Subcategory};
use fuzzy_search::search;

pub fn fuzzy_search(query: &str, categories: &[Category]) -> Vec<usize> {
    let mut results = Vec::new();

    // Search through category titles
    results.extend(
        categories
            .iter()
            .enumerate()
            .filter_map(|(index, category)| {
                if search(&category.title, query).is_some() {
                    Some(index)
                } else {
                    None
                }
            }),
    );

    // Search through subcategory titles
    results.extend(categories.iter().enumerate().flat_map(|(index, category)| {
        category
            .subcategories
            .iter()
            .enumerate()
            .filter_map(|(sub_index, subcategory)| {
                if search(&subcategory.title, query).is_some() {
                    Some((index, sub_index)) // Returning category and subcategory index
                } else {
                    None
                }
            })
    }));

    // Search through link titles
    results.extend(categories.iter().enumerate().flat_map(|(index, category)| {
        category
            .direct_links
            .iter()
            .enumerate()
            .filter_map(|(link_index, link)| {
                if search(&link.title, query).is_some() {
                    Some((index, link_index)) // Returning category and link index
                } else {
                    None
                }
            })
    }));

    results
}
