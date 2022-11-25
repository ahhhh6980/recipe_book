#![allow(unused_imports)]
#![allow(unused_variables)]
use ego_tree::{iter::Edge, NodeRef};
use rand::{distributions, prelude::Distribution, Rng};
use recipe_book::{
    mixed_rational::MixedRational, recipe::ParsedRecipe, recursive_json_grab, units::*,
};
use serde_json::{Result, Value};
use std::{error::Error, fs, path::PathBuf, str::FromStr};

#[allow(clippy::never_loop)]
fn main() -> core::result::Result<(), Box<dyn Error>> {
    let recipe_data = recursive_json_grab("recipes".into(), Vec::new(), 0, "     |")?;
    println!("{}", recipe_data[0]);
    println!("size: {}", recipe_data[0].memory_size());
    println!("{}", recipe_data[0].get_recipe_for_servings(64.into()));

    println!(
        "{}",
        MixedRational::whole(16) * (MixedRational::whole(1) / MixedRational::whole(16))
    );

    let measure = MeasureType::new(
        "tsp".into(),
        MixedRational {
            value: 4,
            num: 2,
            den: 3,
        },
    );

    use Unit::*;
    let new_m = measure.simplify(|m| !m.fluid);
    println!(
        "{} {} = {} {} ",
        measure.count, measure.unit, new_m.count, new_m.unit
    );
    println!(
        "{} {} = {} {} ",
        measure.count,
        measure.unit,
        measure
            .unit
            .convert(measure.count, Measure::from_enum(Tablespoon))
            .unwrap(),
        new_m.unit
    );

    let char_list = ['w', 'i', 'l', 'l'];
    let char_list2 = "william";
    let char_list3 = String::from("william");

    println!("{}", std::mem::size_of_val(&char_list2));
    println!("{}", std::mem::size_of_val(&char_list3));
    println!(
        "{}",
        std::mem::size_of_val(&String::from("william").bytes())
    );

    for a in STANDARD_COOKING_MEASUREMENT_UNITS {
        for b in STANDARD_COOKING_MEASUREMENT_UNITS {
            if a.unit as u8 != b.unit as u8 {
                if let Some(c) = Measure::conversion_table(a, b) {
                    let v = MixedRational::whole(1) / c;
                    println!("1 {} = {}{} ({:.5})", a, v, b, v.to_float())
                }
            }
        }
    }

    let v = 1.2345;

    /*
    let urls = vec![
        "https://www.spendwithpennies.com/homemade-carrot-bread//",
        "https://amandascookin.com/million-dollar-chicken-casserole/",
        "https://www.simplyrecipes.com/instant-pot-turkey-breast-and-gravy-recipe-5207290",
        "https://www.epicurious.com/recipes/food/views/hash-brown-casserole",
        "https://www.averiecooks.com/garlic-butter-chicken/",
        "https://themodernproper.com/60-best-chicken-breast-recipes",
        "https://cafedelites.com/quick-easy-creamy-herb-chicken/",
        "https://www.aheadofthyme.com/25-quick-chicken-recipes/",
        "https://www.foodiecrush.com/30-easy-healthy-chicken-dinners-ideas/",
        "https://littlesunnykitchen.com/marry-me-chicken/",
        "https://www.eatwell101.com/garlic-butter-chicken-bites-asparagus-recipe",
    ];
    let a = Rational::new(2, 4, 7);
    let b = Rational::new(-8, 3, 8);
    println!("{} / {} = {}", a, b, a / b);
    println!("{} * {} = {}", a, b, a * b);
    let c = Rational::fract(41152, 102809);
    println!(
        "{} approx {} (1digit) {} (2digits) {} (3digits) {} (4digits) {} (5digits) ",
        c,
        c.approx_ratio(1),
        c.approx_ratio(2),
        c.approx_ratio(3),
        c.approx_ratio(4),
        c.approx_ratio(5)
    );
    let c = Rational::new(2, 4, 7);

    let a = Measure::new("cup");
    let b = Measure::new("tsp");
    println!(
        "{}, {}, {} {} = {} {}",
        a,
        b,
        c,
        a,
        a.convert(c, b).unwrap(),
        b
    ); */
    /*

    */
    Ok(())
}

// https://amandascookin.com/million-dollar-chicken-casserole/
