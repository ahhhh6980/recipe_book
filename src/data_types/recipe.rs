use serde_json::Value;

use crate::{mixed_rational::MixedRational, MeasureType};
use std::fmt;

#[derive(Debug, Clone)]
pub struct RecipeItem {
    pub unit_first: bool,
    pub name: String,
    pub measure: MeasureType,
    //pub count: MixedRational,
    //pub unit: String,
}

impl std::ops::Mul<MixedRational> for RecipeItem {
    type Output = RecipeItem;
    fn mul(self, rhs: MixedRational) -> Self::Output {
        RecipeItem {
            unit_first: self.unit_first,
            name: self.name,
            measure: self.measure * rhs,
        }
    }
}

impl RecipeItem {
    pub fn memory_size(&self) -> usize {
        std::mem::size_of_val(&self.unit_first)
            + std::mem::size_of_val(&self.name)
            + self.measure.memory_size()
    }
}

impl From<Value> for RecipeItem {
    fn from(value: Value) -> Self {
        RecipeItem {
            unit_first: value["@type"] == "Nutrient",
            name: value["name"].to_string(),
            measure: MeasureType::new(value["unit"].to_string(), value["count"].clone().into()),
        }
    }
}

impl std::fmt::Display for RecipeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.unit_first {
            write!(
                f,
                "{} {} {}",
                self.measure.count, self.measure.unit, self.name
            )
        } else {
            write!(
                f,
                "{}: {} {}",
                self.name, self.measure.count, self.measure.unit
            )
        }
    }
}

#[derive(Clone)]
pub struct RecipeData {
    pub original_servings: MixedRational,
    pub servings: MixedRational,
    pub nutrition_servings_size: MixedRational,
    pub nutrition_servings_unit: String,
    pub ingredients: Vec<RecipeItem>,
    pub directions: Vec<String>,
    pub nutrition_info: Vec<RecipeItem>,
}

impl std::ops::Mul<MixedRational> for RecipeData {
    type Output = RecipeData;
    fn mul(self, rhs: MixedRational) -> Self::Output {
        RecipeData {
            original_servings: self.original_servings,
            servings: self.servings * rhs,
            nutrition_servings_size: self.nutrition_servings_size,
            ingredients: self.ingredients.iter().map(|i| i.clone() * rhs).collect(),
            nutrition_info: self.nutrition_info,
            directions: self.directions,
            nutrition_servings_unit: self.nutrition_servings_unit,
        }
    }
}
impl RecipeData {
    #[rustfmt::skip]
    pub fn memory_size(&self) -> usize {
        self.servings.memory_size()
            + self.nutrition_servings_size.memory_size()
            + std::mem::size_of_val(&self.nutrition_servings_unit)
            + self.ingredients.iter().map(|x| x.memory_size()).sum::<usize>()
            + self.nutrition_info.iter().map(|x| x.memory_size()).sum::<usize>()
            + self.directions.iter().map(std::mem::size_of_val).sum::<usize>()
    }
    pub fn scale_servings(&self, target_servings: MixedRational) -> Self {
        let scale = target_servings / self.servings;
        self.clone() * scale
    }
}

#[derive(Clone)]
pub struct RecipeText {
    pub title: String,
    pub prep_time: String,
    pub author_name: String,
    pub origin: String,
    pub description: String,
}

impl RecipeText {
    pub fn memory_size(&self) -> usize {
        std::mem::size_of_val(&self.title)
            + std::mem::size_of_val(&self.prep_time)
            + std::mem::size_of_val(&self.author_name)
            + std::mem::size_of_val(&self.origin)
            + std::mem::size_of_val(&self.description)
    }
}

#[derive(Clone)]
pub struct ParsedRecipe {
    pub data: RecipeData,
    pub text: RecipeText,
    pub keywords: Vec<String>,
}

impl ParsedRecipe {
    #[rustfmt::skip]
    pub fn memory_size(&self) -> usize {
        self.keywords.iter().map(std::mem::size_of_val).sum::<usize>()
            + self.data.memory_size()
            + self.text.memory_size()
    }
    pub fn get_recipe_for_servings(&self, target_servings: MixedRational) -> Self {
        ParsedRecipe {
            data: self.data.scale_servings(target_servings),
            text: self.text.clone(),
            keywords: self.keywords.clone(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<ParsedRecipe> for Value {
    fn into(self) -> ParsedRecipe {
        let mut ingredients: Vec<RecipeItem> = Vec::new();
        if let Some(unparsed) = self["ingredients"].as_array() {
            for value in unparsed.iter() {
                ingredients.push(value.clone().into())
            }
        }
        let mut nutrition_info: Vec<RecipeItem> = Vec::new();
        if let Some(unparsed) = self["nutrition"].as_array() {
            for value in unparsed.iter() {
                nutrition_info.push(value.clone().into())
            }
        }
        let mut directions = Vec::new();
        if let Some(unparsed) = self["directions"].as_array() {
            for value in unparsed.iter().flat_map(|v| v.as_str()) {
                directions.push(String::from(value))
            }
        }
        let mut keywords: Vec<String> = Vec::new();
        if let Some(unparsed) = self["keywords"].as_array() {
            for value in unparsed.iter() {
                keywords.push(value.to_string())
            }
        }
        ParsedRecipe {
            data: RecipeData {
                original_servings: self["servings"].clone().into(),
                servings: self["servings"].clone().into(),
                nutrition_servings_size: self["serving_size"]["count"].clone().into(),
                nutrition_servings_unit: self["serving_size"]["unit"].to_string(),
                ingredients,
                directions,
                nutrition_info,
            },
            text: RecipeText {
                title: self["title"].to_string(),
                prep_time: self["prep_time"].to_string(),
                author_name: self["author_name"].to_string(),
                origin: self["origin"].to_string(),
                description: self["description"].to_string(),
            },
            keywords,
        }
    }
}

impl std::fmt::Display for ParsedRecipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        string.push_str("\n\n\t");
        string.push_str(&self.text.title);
        string.push_str("\n\t");
        string.push_str("By: ");
        string.push_str(&self.text.author_name);
        string.push_str("\n\n");

        string.push_str("From: ");
        string.push_str(&self.text.origin);
        string.push_str("\n\n");

        string.push_str(&self.text.description);
        string.push_str("\n\n");

        string.push_str("Estimated Prep Time: ");
        string.push_str(&self.text.prep_time);
        string.push('\n');

        string.push_str("Servings: ");
        string.push_str(&self.data.servings.to_string());
        string.push_str("\n\n");

        string.push_str("Ingredients:\n");
        let c = self.data.ingredients.len().to_string().len() + 2;
        for (i, ingredient) in self.data.ingredients.iter().enumerate() {
            string.push_str(&format!("{: >n$}) {}\n", i, ingredient, n = c));
        }

        string.push('\n');
        string.push_str("Directions: \n");
        let c = self.data.directions.len().to_string().len() + 2;
        for (i, direction) in self.data.directions.iter().enumerate() {
            string.push_str(&format!("{: >n$}) {}\n", i, direction, n = c));
        }

        string.push_str("\nServing Size: ");
        string.push_str(&self.data.nutrition_servings_size.to_string());
        string.push(' ');
        string.push_str(&self.data.nutrition_servings_unit.to_string());
        string.push_str("\n\n");

        string.push_str("Nutrition Info:\n");
        for nutrient in self.data.nutrition_info.iter() {
            string.push_str(&format!("  {}\n", nutrient));
        }

        write!(f, "{}", string.replace('"', "").replace(" null", ""))
    }
}
