use serde::{de::IntoDeserializer, ser::SerializeStruct};
use serde_json::Value;

use crate::{mixed_rational::MixedRational, MeasureType};
use std::fmt;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RecipeItem {
    #[serde(flatten)]
    pub measure: MeasureType,
    #[serde(serialize_with = "crate::proper_string_serialize")]
    pub name: String,
    pub note: Option<String>,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct NutritionInfo {
    pub servings_size: MixedRational,
    #[serde(serialize_with = "crate::proper_string_serialize")]
    pub servings_unit: String,
    pub nutrients: Vec<RecipeItem>,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecipeData {
    #[serde(skip_deserializing, skip_serializing)]
    pub original_servings: Option<MixedRational>,
    pub servings: MixedRational,
    pub ingredients: Vec<RecipeItem>,
    pub directions: Vec<String>,
    pub nutrition_info: NutritionInfo,
}

#[derive(Clone, Default, Debug, serde::Deserialize)]
pub struct RecipeText {
    pub title: String,
    pub prep_time: String,
    pub author_name: String,
    pub origin: String,
    pub description: String,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct ParsedRecipe {
    pub keywords: Vec<String>,
    #[serde(flatten)]
    pub text: RecipeText,
    #[serde(flatten)]
    pub data: RecipeData,
}

impl RecipeItem {
    pub fn memory_size(&self) -> usize {
        std::mem::size_of_val(&self.name) + self.measure.memory_size()
    }
}

impl NutritionInfo {
    pub fn memory_size(&self) -> usize {
        self.nutrients
            .iter()
            .map(|x| x.memory_size())
            .sum::<usize>()
            + self.servings_size.memory_size()
            + std::mem::size_of_val(&self.servings_unit)
    }
}

impl RecipeData {
    #[rustfmt::skip]
    pub fn memory_size(&self) -> usize {
        self.servings.memory_size()
            + self.nutrition_info.memory_size()
            + self.ingredients.iter().map(|x| x.memory_size()).sum::<usize>()
            + self.directions.iter().map(std::mem::size_of_val).sum::<usize>()
    }
    pub fn scale_servings(&self, target_servings: MixedRational) -> Self {
        let scale = target_servings / self.servings;
        self.clone() * scale
    }
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
    pub fn from_path<P>(path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        P: AsRef<std::path::Path>,
    {
        let hm = std::fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&hm)?;
        Ok(ParsedRecipe::deserialize_json_string(
            json.into_deserializer(),
        )?)
    }
    fn deserialize_json_string<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s: serde_json::Value = serde::de::Deserialize::deserialize(deserializer)?;
        serde_json::from_value(s).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for RecipeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.measure.count, self.measure.unit, self.name
        )
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
        string.push_str(&self.data.nutrition_info.servings_size.to_string());
        string.push(' ');
        string.push_str(&self.data.nutrition_info.servings_unit.to_string());
        string.push_str("\n\n");

        string.push_str("Nutrition Info:\n");
        for nutrient in self.data.nutrition_info.nutrients.iter() {
            string.push_str(&format!("  {}\n", nutrient));
        }

        write!(f, "{}", string.replace('"', "").replace(" null", ""))
    }
}

impl serde::Serialize for RecipeText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut st = serializer.serialize_struct("RecipeText", 5)?;
        st.serialize_field("title", &self.title.replace('"', ""))?;
        st.serialize_field("prep_time", &self.prep_time.replace('"', ""))?;
        st.serialize_field("author_name", &self.author_name.replace('"', ""))?;
        st.serialize_field("origin", &self.origin.replace('"', ""))?;
        st.serialize_field("description", &self.description.replace('"', ""))?;
        st.end()
    }
}

impl std::ops::Mul<MixedRational> for RecipeItem {
    type Output = RecipeItem;
    fn mul(self, rhs: MixedRational) -> Self::Output {
        RecipeItem {
            name: self.name,
            measure: self.measure * rhs,
            note: self.note,
        }
    }
}

impl std::ops::Mul<MixedRational> for RecipeData {
    type Output = RecipeData;
    fn mul(self, rhs: MixedRational) -> Self::Output {
        RecipeData {
            original_servings: self.original_servings,
            servings: self.servings * rhs,
            ingredients: self.ingredients.iter().map(|i| i.clone() * rhs).collect(),
            directions: self.directions,
            nutrition_info: self.nutrition_info,
        }
    }
}
// recipeIngredient
/*
impl From<serde_json::Value> for ParsedRecipe {
    fn from(value: serde_json::Value) -> Self {
        let ingredients_txt = if let Some(ingredients) = value["ingredients"].as_array() {
            ingredients.iter().map(|v| v.to_string()).collect()
        } else {
            value["ingredients"]
                .to_string()
                .split('\n')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        };
        let mut ingredients: Vec<RecipeItem> = Vec::new();
        for ingredient_str in ingredients_txt {
            let or = ingredient_str.find(pat)
            let value_a = String::new();
            let value_b = String::new();
            let (a, b) = ingredient_str.split("or");
        }
        Self::default()
    }
}*/
