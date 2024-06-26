use serde::{de::IntoDeserializer, ser::SerializeStruct};
use serde_json::Value;

use crate::{mixed_rational::MixedRational, Measure, MeasureType, Unit};
use std::fmt;

fn none<T>(s: &Option<T>) -> bool {
    s.is_none()
}

fn default_option<T>() -> Option<T> {
    None
}
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RecipeItem {
    #[serde(flatten)]
    pub measure: MeasureType,
    #[serde(skip_serializing_if = "none", default = "default_option")]
    pub measure_b: Option<MeasureType>,
    #[serde(serialize_with = "crate::proper_string_serialize")]
    pub name: String,
    #[serde(skip_serializing_if = "none", default = "default_option")]
    pub note: Option<String>,
    pub plural: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DirectionSection {
    pub name: String,
    pub sections: Vec<String>,
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
    pub directions: Vec<DirectionSection>,
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
        let note = if let Some(n) = self.note.clone() {
            format!(" ({})", n)
        } else {
            "".into()
        };
        let unit = if self.plural {
            format!("{}s", self.measure.unit)
        } else {
            self.measure.unit.names[0].to_string()
        };
        if let Some(m) = self.measure_b {
            write!(
                f,
                "{}-{} {} {}{}",
                self.measure.count, m.count, unit, self.name, note
            )
        } else {
            write!(f, "{} {} {}{}", self.measure.count, unit, self.name, note)
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
        for section in self.data.directions.iter() {
            string.push_str(&format!("{} {}\n", " ".repeat(c - 2), section.name));
            for (i, direction) in section.sections.iter().enumerate() {
                string.push_str(&format!("{: >n$}) {}\n", i, direction, n = c * 2));
            }
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
            measure_b: self.measure_b.map(|b| b * rhs),
            note: self.note,
            plural: self.plural,
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
fn remove_duplicate_chars(s: &str, chars: &[char]) -> String {
    let mut s = s.to_string();
    chars.iter().for_each(|c| {
        s = s
            .split(*c)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(&c.to_string())
    });
    s
}
impl From<(String, serde_json::Value)> for ParsedRecipe {
    fn from((url, value): (String, serde_json::Value)) -> Self {
        let ingredients_txt = if let Some(ingredients) = value["recipeIngredient"].as_array() {
            ingredients.iter().map(|v| v.to_string()).collect()
        } else {
            value["recipeIngredient"]
                .to_string()
                .split('\n')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        };
        // Init ingredients list
        let mut ingredients: Vec<RecipeItem> = Vec::new();
        for mut ingredient_str in ingredients_txt {
            // Remove duplicate spaces and parenthesis
            ingredient_str = remove_duplicate_chars(ingredient_str.trim(), &[' ', '(', ')']);
            // Find where parenthesis are
            let parens = (ingredient_str.find('('), ingredient_str.find(')'));
            // Assemble a note if there are parenthesis
            let note: String = if let (Some(left), Some(right)) = parens {
                if ingredient_str.as_bytes()[left + 1] == b',' {
                    "".into()
                } else {
                    let new = ingredient_str[left + 1..right].into();
                    ingredient_str = ingredient_str[..left].into();
                    new
                }
            } else {
                "".into()
            };
            // Remove unnecessary characters
            ingredient_str = ingredient_str.replace(['"', ',', ';', '(', ')'], "");
            // Break if we have nothing >.>
            if ingredient_str == "null" {
                continue;
            }
            /*
                If the first few characters are not part of a rational, we dont need extra processing
            */
            let (first_rational_char, first_alpha) = (
                ingredient_str.find(|c: char| MixedRational::valid_chars().contains(c)),
                ingredient_str.find(|c: char| !MixedRational::valid_chars().contains(c)),
            );
            if let Some(rational_char) = first_rational_char {
                if let Some(alpha) = first_alpha {
                    if alpha < rational_char {
                        println!("Processed: {}", ingredient_str);
                        ingredients.push(RecipeItem {
                            measure: MeasureType {
                                count: 0.into(),
                                unit: Measure {
                                    unit: Unit::Other,
                                    names: &[""],
                                    fluid: false,
                                },
                            },
                            measure_b: None,
                            name: ingredient_str,
                            note: None,
                            plural: false,
                        });
                        println!();
                        continue;
                    }
                }
            }
            // Just grab the first option if the recipe lists different alternatives
            let ingredient = if let Some(or) = ingredient_str.find(" or ") {
                ingredient_str.split_at(or).0.into()
            } else {
                ingredient_str
            };
            // Find the first non-rational character
            let non_fract = ingredient.find(|c: char| c.is_alphabetic()).unwrap();
            let (fract_str, unit_str) = ingredient.split_at(non_fract);
            let (count, count_b) = MixedRational::from_string(fract_str.to_string());
            let (name_str, unit_str): (String, String) = if let Some(space) = unit_str.find(' ') {
                (
                    unit_str[space + 1..].into(),
                    // Ensure the unit name is letters only for lookup
                    unit_str[..space]
                        .replace(|c: char| !c.is_ascii_alphabetic(), "")
                        .to_lowercase(),
                )
            } else {
                ("".into(), unit_str.into())
            };
            // The actual struct for the unit, providing useful methods
            let mut struct_unit = Measure::new(unit_str.clone());
            let ingredient = if struct_unit.unit as u8 == Unit::Other as u8 {
                struct_unit.names = &[""];
                // Concatenate unit and name as there is no actual unit here
                let mut unit = unit_str;
                unit.push(' ');
                unit.push_str(&name_str);
                RecipeItem {
                    measure: MeasureType {
                        count,
                        unit: struct_unit,
                    },
                    // If the recipe specifies a range, this is the upper limit
                    // ex: 2-3 oz of cream cheese
                    measure_b: count_b.map(|m| MeasureType {
                        count: m,
                        unit: struct_unit,
                    }),
                    name: unit,
                    note: if note.is_empty() { None } else { Some(note) },
                    plural: false,
                }
            } else {
                RecipeItem {
                    measure: MeasureType {
                        count,
                        unit: struct_unit,
                    },
                    // If the recipe specifies a range, this is the upper limit
                    // ex: 2-3 oz of cream cheese
                    measure_b: count_b.map(|m| MeasureType {
                        count: m,
                        unit: struct_unit,
                    }),
                    name: name_str,
                    note: if note.is_empty() { None } else { Some(note) },
                    plural: unit_str.ends_with("'s") || unit_str.ends_with('s'),
                }
            };
            ingredients.push(ingredient);
            //let range = ingredient.find('-');
        }
        let mut nutrition_info = Vec::new();
        // "servingSize",
        for field in [
            "calories",
            "carbohydrateContent",
            "proteinContent",
            "fatContent",
            "saturatedFatContent",
            "transFatContent",
            "cholesterolContent",
            "sodiumContent",
            "fiberContent",
            "sugarContent",
            "unsaturatedFatContent",
        ]
        .iter()
        {
            let val = value["nutrition"][field].to_string().replace('"', "");
            if let Some(space) = val.find(' ') {
                let (whole, unit) = val.split_at(space);
                if let Ok(v) = whole.replace(|c: char| !c.is_numeric(), "").parse::<i32>() {
                    nutrition_info.push(RecipeItem {
                        measure: MeasureType {
                            count: MixedRational::whole(v),
                            unit: Measure::new(unit.replace(' ', "")),
                        },
                        measure_b: None,
                        name: String::from(*field),
                        note: None,
                        plural: false,
                    });
                }
            }
        }
        let mut new = Self::default();
        new.data.ingredients = ingredients;
        new.data.nutrition_info = NutritionInfo {
            servings_size: MixedRational::default(),
            servings_unit: "".into(),
            nutrients: nutrition_info,
        };
        //HowToSection
        let mut section = DirectionSection::default();
        let mut directions = Vec::new();
        if let Some(d_array) = value["recipeInstructions"].as_array() {
            for item in d_array {
                match item["@type"].as_str() {
                    Some("HowToSection") => {
                        if let Some(s_array) = item["itemListElement"].as_array() {
                            if !section.sections.is_empty() {
                                directions.push(section);
                            }
                            section = DirectionSection::default();
                            section.name = item["name"].to_string().replace('"', "");
                            for d in s_array {
                                section
                                    .sections
                                    .push(d["text"].to_string().replace('"', ""))
                            }
                        }
                    }
                    Some("HowToStep") => {
                        let mut s = item["text"].to_string().replace('"', "");
                        let mut new_section = DirectionSection::default();
                        let mut i = 0;
                        if s.contains(':') {
                            while s.len() > 2 && i < 256 {
                                if let Some(mut colon) = s.find(':') {
                                    if let Some(end) = s[colon + 1..].find(". ") {
                                        if end < colon {
                                            new_section
                                                .sections
                                                .push(s[..end].to_string().replace('"', ""));
                                            s = s[end + 2..].into();
                                            continue;
                                        }
                                        if let Some(next_colon) = s[colon + 1..].find(':') {
                                            if end > next_colon {
                                                colon = colon + 1 + next_colon;
                                            }
                                        }
                                    }
                                    if !new_section.sections.is_empty() {
                                        directions.push(new_section);
                                    }
                                    new_section = DirectionSection::default();
                                    new_section.name = s[..colon].to_string().replace('"', "");
                                    s = s[colon + 1..].into();
                                } else if let Some(end) = s.find(". ") {
                                    new_section
                                        .sections
                                        .push(s[..end].to_string().replace('"', ""));
                                    s = s[end + 2..].into();
                                }
                                i += 1;
                            }
                            new_section.sections.push(s);
                            directions.push(new_section);
                        } else if let Some(semi) = s.find(';') {
                            if s.split_at(semi).0.replace(|c: char| c != ' ', "").len()
                                < s.split_at(semi).1.replace(|c: char| c != ' ', "").len()
                            {
                                while s.len() > 2 && i < 256 {
                                    if let Some(semicolon) = s.find(';') {
                                        if !new_section.sections.is_empty() {
                                            directions.push(new_section);
                                        }
                                        new_section = DirectionSection::default();
                                        new_section.name =
                                            s[..semicolon].to_string().replace('"', "");

                                        s = s[semicolon + 1..].into();
                                    } else if let Some(end) = s.find(". ") {
                                        new_section
                                            .sections
                                            .push(s[..end].to_string().replace('"', ""));
                                        s = s[end + 2..].into();
                                    }
                                    i += 1;
                                }

                                if new_section.sections.is_empty() {
                                    section.sections.push(s);
                                } else {
                                    directions.push(new_section);
                                }
                            } else {
                                directions.push(new_section);
                            }
                        } else {
                            section.sections.push(s);
                        }
                    }
                    _ => {}
                }
            }
        }
        if !section.sections.is_empty() {
            directions.push(section);
        }
        /*
        for direction in directions {
            if direction.len() > 90 {
                if direction.contains(". ") {
                    for d in direction.split(". ") {
                        if !d.is_empty() {
                            new.data.directions.push(d.into());
                        }
                    }
                } else {
                    new.data.directions.push(direction);
                }
            } else {
                new.data.directions.push(direction);
            }
        } */
        new.data.directions = directions;

        new.text.author_name = if let Some(arr) = value["author"].as_array() {
            arr[0]["name"].to_string()
        } else {
            value["author"]["name"].to_string()
        };

        new.text.description = value["description"].to_string();
        new.text.title = value["name"].to_string();
        new.text.origin = url;
        if let Some(ryield) = value["recipeYield"].as_array() {
            let s = ryield[0].to_string();
            new.data.servings = MixedRational::from_string(s).0;
        } else {
            let s = value["recipeYield"].to_string();
            if !s.is_empty() {
                new.data.servings = MixedRational::from_string(s).0;
            }
        }

        new.keywords = value["keywords"]
            .as_array()
            .map_or(Vec::new(), |m| m.iter().map(|v| v.to_string()).collect());

        new.text.prep_time = if let Some(time) = [
            value["totalTime"].to_string(),
            value["performTime"].to_string(),
            value["cookTime"].to_string(),
            value["prepTime"].to_string(),
        ]
        .iter()
        .find(|s| !s.contains("null"))
        {
            let mut t = time.replace('"', "");
            if t.contains("PT") {
                let stime = t.replace(|c: char| !c.is_numeric(), "");
                if let Ok(minutes) = stime.parse::<usize>() {
                    let (h, m) = (minutes / 60, minutes % 60);
                    t = format!(
                        "{} {}",
                        match h {
                            0 => "".into(),
                            1 => format!("{} hour", h),
                            _ => format!("{} hours", h),
                        },
                        match m {
                            0 => "".into(),
                            1 => format!("{} minute", m),
                            _ => format!("{} minutes", m),
                        },
                    );
                }
            }
            t
        } else {
            "".into()
        };

        std::fs::write(
            format!("recipes/wip/{}.json", new.text.title.replace('"', "")),
            serde_json::to_string(&new).unwrap(),
        )
        .unwrap();

        println!("{}", new);
        new
    }
}
