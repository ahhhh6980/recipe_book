#![allow(unused_imports)]
#![allow(unused_variables)]
use crate::{mixed_rational::MixedRational, recipe::ParsedRecipe, units::*};
use serde_json::{Result, Value};
use std::{error::Error, fs, path::PathBuf, time::SystemTime};
use time::OffsetDateTime;

pub fn get_time_string() -> String {
    let test: OffsetDateTime = SystemTime::now().into();
    test.to_string()
}

pub fn proper_string_serialize<S>(s: &str, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&(s.replace('"', "")))
}

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
                if let Ok(recipe) = ParsedRecipe::from_path(entry.path()) {
                    // Print name of recipe and then push to our recipes list
                    println!(
                        "{} Recipe Name: {}",
                        padding.repeat(depth + 1),
                        recipe.text.title
                    );
                    new_recipes.push(recipe);
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
