#![allow(unused_imports)]
#![allow(unused_variables)]
use crate::{mixed_rational::MixedRational, recipe::ParsedRecipe, units::*};
use serde_json::{Result, Value};
use std::{error::Error, fs, path::PathBuf};

pub fn recursive_json_grab(
    dir: PathBuf,
    recipes: Vec<ParsedRecipe>,
    depth: usize,
    padding: &str,
) -> core::result::Result<Vec<ParsedRecipe>, Box<dyn Error>> {
    println!(
        "{} Searching [{}/]:",
        padding.repeat(depth),
        &dir.as_os_str().to_string_lossy().split('/').last().unwrap()
    );
    // Move found recipes to new vec
    let mut new_recipes = recipes;
    if let Ok(entries) = fs::read_dir(&dir) {
        // Iterate over everything inside of this folder
        for entry in entries.flatten() {
            if entry.file_type()?.is_file() {
                println!(
                    "{}>Parsing {}",
                    padding.repeat(depth + 1),
                    entry.path().file_name().unwrap().to_string_lossy()
                );
                // Read as string so that serde_json can parse it
                if let Ok(data) = fs::read_to_string(entry.path()) {
                    let recipe_data: Value = serde_json::from_str(&data)?;
                    // Print name of recipe and then push to our recipes list
                    println!(
                        "{} Recipe Name: {}",
                        padding.repeat(depth + 1),
                        recipe_data["title"]
                    );
                    new_recipes.push(recipe_data.into());
                }
            } else {
                // Otherwise repeat this process over this folder
                new_recipes = recursive_json_grab(entry.path(), new_recipes, depth + 1, padding)?;
            }
        }
    }
    // Finally return our recipes :)
    Ok(new_recipes)
}
