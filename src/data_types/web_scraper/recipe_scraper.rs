use crate::{get_time_string, ParsedRecipe};
use reqwest::header::Iter;
use serde::de::IntoDeserializer;

#[derive(Default)]
pub struct Scraper {
    pub json: Vec<serde_json::Value>,
    pub un_parsed: Vec<(String, scraper::Html)>,
    pub parsed: Vec<ParsedRecipe>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct UrlHistoryItem {
    pub time: String,
    pub url: String,
    pub name: String,
    pub processed: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct UrlHistory {
    pub grabbed_json_history: Vec<UrlHistoryItem>,
    pub skipped: Vec<UrlHistoryItem>,
    //pub non_json_urls: Vec<UrlHistory>,
}

impl UrlHistory {
    pub fn get_grabbed(&self) -> Vec<String> {
        self.grabbed_json_history
            .clone()
            .into_iter()
            .map(|h| h.url)
            .collect()
    }
    pub fn from_file<P>(path: P) -> Result<Self, serde_json::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let backup = serde_json::json!({
            "grabbed_json_history": [],
            "skipped": []
        });
        Self::deserialize_json_string(
            if let Ok(f) = &std::fs::read_to_string(path) {
                if let Ok(s) = serde_json::from_str(f) {
                    s
                } else {
                    backup
                }
            } else {
                backup
            }
            .into_deserializer(),
        )
    }
    fn deserialize_json_string<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s: serde_json::Value = serde::de::Deserialize::deserialize(deserializer)?;
        serde_json::from_value(s).map_err(serde::de::Error::custom)
    }
}

impl Scraper {
    pub fn get_json(
        &mut self,
        urls: Vec<String>,
        char_count_limit: usize,
    ) -> std::result::Result<Vec<usize>, Box<dyn std::error::Error>> {
        let mut url_hist_obj = UrlHistory::from_file("temp/scrape_history.json")?;
        let url_history = url_hist_obj.get_grabbed();

        let mut found = Vec::new();
        for (i, url) in urls.iter().enumerate() {
            if url_history.contains(url) {
                continue;
            }
            // Get the web page
            println!("{}", url);
            let response = reqwest::blocking::get(url).unwrap().text().unwrap();
            let document = scraper::Html::parse_document(&response);
            let tree = &document.tree;
            /*
               Look at each node in the tree until we find text containing JSON
               Specifically, JSON containing our RECIPE! :)

               This should work for any recipe that google can display in blocks
                   when looking for recipes :D
            */
            let mut foundf = false;
            for node in tree.nodes() {
                let value = node.value();
                if value.is_text() {
                    let mut txt = value.as_text().unwrap().to_string();
                    let lower = txt.to_lowercase();

                    // Some edge cases, WIP
                    if (lower.contains(r#""@type""#)
                        && (lower.contains(r#""recipe""#) || lower.contains(r#"["recipe""#)))
                        && (txt.contains("Instructions"))
                        && lower.len() < char_count_limit
                    {
                        // Ensure the string starts and ends with curly braces :)
                        let curly = txt.find('{');
                        txt = txt.split_at(curly.unwrap()).1.into();
                        while !txt.ends_with('}') {
                            txt.remove(txt.len() - 1);
                        }
                        // Remove escape characters
                        txt = txt
                            .replace(r#"\/"#, "/")
                            .replace(r#"\u00a0"#, " ")
                            .replace("&#39;", "'")
                            .replace("&#039;", "'");
                        let json: serde_json::Value = serde_json::from_str(&txt)?;
                        let graph = &json["@graph"];
                        if let Some(arr) = graph.as_array() {
                            for item in arr {
                                if format!("{}", item["@type"])
                                    .to_lowercase()
                                    .contains("recipe")
                                {
                                    println!("{} : {}", i, item["name"]);
                                    std::fs::write(
                                        format!(
                                            "temp/{}.json",
                                            &item["name"]
                                                .to_string()
                                                .replace(' ', "_")
                                                .replace('"', "")
                                        ),
                                        &txt,
                                    )?;
                                    self.json.push(item.clone());
                                    found.push(i);
                                    foundf = true;
                                    url_hist_obj.grabbed_json_history.push(UrlHistoryItem {
                                        time: get_time_string(),
                                        url: url.clone(),
                                        name: item["name"].to_string().replace('"', ""),
                                        processed: false,
                                    });
                                    self.parsed
                                        .push(ParsedRecipe::from((url.clone(), item.clone())));
                                }
                            }
                        } else {
                            println!("{} : {}", i, json["name"]);
                            std::fs::write(
                                format!(
                                    "temp/{}.json",
                                    &json["name"].to_string().replace(' ', "_").replace('"', "")
                                ),
                                &txt,
                            )?;
                            self.json.push(json.clone());
                            foundf = true;
                            found.push(i);
                            url_hist_obj.grabbed_json_history.push(UrlHistoryItem {
                                time: get_time_string(),
                                url: url.clone(),
                                name: json["name"].to_string().replace('"', ""),
                                processed: false,
                            });
                            self.parsed
                                .push(ParsedRecipe::from((url.clone(), json.clone())));
                        }
                        // Parse to JSON and push to structs list of JSON

                        break;
                    }
                }
            }
            if !foundf {
                // We didn't find a recipe in JSON :(
                self.un_parsed.push((url.clone(), document));
                url_hist_obj.skipped.push(UrlHistoryItem {
                    time: get_time_string(),
                    url: url.clone(),
                    name: url.clone(),
                    processed: false,
                });
            }
        }
        std::fs::write(
            "temp/scrape_history.json",
            serde_json::to_string(&url_hist_obj)?,
        )?;
        Ok(found)
    }
}
