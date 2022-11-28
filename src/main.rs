#![allow(unused_imports)]
#![allow(unused_variables)]
use ego_tree::{iter::Edge, NodeRef};
use rand::{distributions, prelude::Distribution, Rng};
use recipe_book::{
    mixed_rational::MixedRational, recipe::ParsedRecipe, recursive_json_grab, units::*, Scraper,
};
use serde::de::IntoDeserializer;
use serde_json::{Result, Value};
use std::{error::Error, fs, path::PathBuf, str::FromStr};

#[allow(clippy::never_loop)]
fn main() -> core::result::Result<(), Box<dyn Error>> {
    let recipe_data = recursive_json_grab("recipes".into(), Vec::new(), 0, "     |")?;

    println!("{}", recipe_data[0]);
    println!("size: {}", recipe_data[0].memory_size());

    let urls: Vec<String> = vec![
        "https://www.spendwithpennies.com/homemade-carrot-bread//".into(),
        //"https://amandascookin.com/million-dollar-chicken-casserole/".into(),
        //"https://www.simplyrecipes.com/instant-pot-turkey-breast-and-gravy-recipe-5207290".into(),
        //"https://www.epicurious.com/recipes/food/views/hash-brown-casserole".into(),
        //"https://www.averiecooks.com/garlic-butter-chicken/".into(),
        //"https://themodernproper.com/lemon-chicken".into(),
        //"https://cafedelites.com/quick-easy-creamy-herb-chicken/".into(),
        //"https://www.aheadofthyme.com/roasted-spatchcock-chicken-butterflied-chicken/".into(),
        //"https://www.foodiecrush.com/thai-chicken-cucumber-salad/".into(),
        //"https://littlesunnykitchen.com/marry-me-chicken/".into(),
        //"https://www.eatwell101.com/garlic-butter-chicken-bites-asparagus-recipe".into(),
        //"https://www.ethanchlebowski.com/cooking-techniques-recipes/homemade-in-n-out-double-double-amp-animal-style-fries".into(),
        //"https://www.simplyrecipes.com/recipes/smothered_turkey_wings/".into(),
    ];

    let mut scraper = Scraper::default();
    scraper.get_json(urls, 100000)?;
    for recipe in scraper.json {}

    Ok(())
}

// https://amandascookin.com/million-dollar-chicken-casserole/
