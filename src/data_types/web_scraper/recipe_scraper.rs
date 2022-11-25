pub struct Scraper {
    pub json: Vec<serde_json::Value>,
    pub un_parsed: Vec<(String, scraper::Html)>,
}

impl Scraper {
    pub fn get_json(
        &mut self,
        urls: Vec<String>,
        char_count_limit: usize,
    ) -> std::result::Result<Vec<usize>, Box<dyn std::error::Error>> {
        let mut found = Vec::new();
        for (i, url) in urls.iter().enumerate() {
            // Get the web page
            let response = reqwest::blocking::get(url).unwrap().text().unwrap();
            let document = scraper::Html::parse_document(&response);
            let tree = &document.tree;
            /*
               Look at each node in the tree until we find text containing JSON
               Specifically, JSON containing our RECIPE! :)

               This should work for any recipe that google can display in blocks
                   when looking for recipes :D
            */
            for node in tree.nodes() {
                let value = node.value();
                if value.is_text() {
                    let mut txt = value.as_text().unwrap().to_string();
                    let lower = txt.to_lowercase();

                    // Some edge cases, WIP
                    if (lower.contains(r#""@type""#)
                        && (lower.contains(r#""recipe""#) || lower.contains(r#"["recipe""#)))
                        && lower.len() < char_count_limit
                    {
                        // Ensure the string starts and ends with curly braces :)
                        let curly = txt.find('{');
                        txt = txt.split_at(curly.unwrap()).1.into();
                        while !txt.ends_with('}') {
                            txt.remove(txt.len() - 1);
                        }
                        // Remove escape characters
                        txt = txt.replace(r#"\/"#, " ");
                        // Parse to JSON and push to structs list of JSON
                        self.json.push(serde_json::from_str(&txt)?);
                        found.push(i);
                        break;
                    }
                }
            }
            // We didn't find a recipe in JSON :(
            self.un_parsed.push((url.clone(), document))
        }
        Ok(found)
    }
}
