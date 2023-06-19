use reqwest::{
    Url,
    Client,
    header::{
        HeaderMap,
        HeaderValue,
    },
};
use chatgpt::{
    client::ChatGPT,
    types::CompletionResponse,
};
use serde::{
    Serialize,
    Deserialize,
};
use std::{
    error::Error,
    collections::HashMap,
    env,
    fs,
};
use serde_json::Value;
use regex::Regex;
use chrono::{
    DateTime,
    Local,
    prelude,
};

use crate::webscraper::Website;
use crate::markdown::Markdown;

#[derive(Serialize, Deserialize)]
pub struct Food {
    food_name: String,
    brand_name: Value,
    serving_qty: f32,
    serving_unit: String,
    serving_weight_grams: f32,
    nf_calories: f32,
    nf_total_fat: f32,
    nf_saturated_fat: f32,
    nf_cholesterol: f32,
    nf_sodium: f32,
    nf_total_carbohydrate: f32,
    nf_dietary_fiber: f32,
    nf_sugars: Value,
    nf_protein: f32,
    nf_potassium: Value,
    nf_p: Value,
    full_nutrients: Vec<Nutrients>,
    nix_brand_name: Value,
    nix_brand_id: Value,
    nix_item_name: Value,
    nix_item_id: Value,
    upc: Value,
    consumed_at: Value,
    metadata: Value,
    source: u32,
    ndb_no: u32,
    tags: Value,
    alt_measures: Value,
    lat: Value,
    lng: Value,
    meal_type: u32,
    photo: Value,
    sub_recipe: Value,
    class_code: Value,
    brick_code: Value,
    tag_id: Value,
}

#[derive(Serialize, Deserialize)]
struct NutritionResponse {
    foods: Vec<Food>,
}

#[derive(Serialize, Deserialize)]
pub struct Nutrients {
    attr_id: u32,
    value: Value
}

pub struct Recipe {
    pub website: Website,
    pub title: String,
    pub date: DateTime<Local>,
    pub ingredients: Vec<Food>,
    pub directions: Vec<String>,
    pub chatgpt_response: String,
}

impl Recipe {
    pub fn new(website: Website) -> Recipe {
        Recipe {
            website: website,
            title: String::new(),
            date: prelude::Local::now(),
            ingredients: Vec::new(),
            directions: Vec::new(),
            chatgpt_response: String::new(),
        }
    }

    pub async fn from_get(website: Website) -> Result<Recipe, Box<dyn Error>> {
        let mut recipe = Recipe::new(website);
        recipe.get().await?;
        Ok(recipe)
    }

    pub async fn get(&mut self) -> Result<(), Box<dyn Error>> {
        if self.website.plaintext.len() == 0 {
            return Err(Box::<dyn Error>::from("Website is empty... was it scraped?"));
        }

        // Prompt ChatGPT to get the directions and ingredients
        let key = env::var("OPENAI_KEY")?;
        let client = ChatGPT::new(key)?;
        let prompt = fs::read_to_string("prompt").expect("prompt expected");
        let message = format!("{}\n{}", prompt, self.website.plaintext);
        let chatgpt_response = client.send_message(message).await?;
        self.chatgpt_response = chatgpt_response.message()
            .content
            .clone();

        println!("CHATGPT: {}", self.chatgpt_response.as_str());

        // Parse the title
        let find_title = Regex::new(r#"Title:.*"#).unwrap();
        for title in find_title.captures_iter(self.chatgpt_response.as_str()) {
            self.title = String::from(title.get(0).unwrap().as_str()).replace("Title: ", "");
        }
        println!("{}", self.title);


        // Parse the directions
        let find_directions = Regex::new(r#"[1-9]\..*"#).unwrap();
        for direction in find_directions.captures_iter(self.chatgpt_response.as_str()) {
            self.directions.push(String::from(direction.get(0).unwrap().as_str()));
        }

        // Parse the ingredients
        let find_ingredients = Regex::new(r#"\*.*"#).unwrap();
        let mut ingredients_as_str = String::new();
        for ingredient in find_ingredients.captures_iter(self.chatgpt_response.as_str()) {
            println!("{}", ingredient.get(0).unwrap().as_str());
            ingredients_as_str.push_str(&format!("{}, and\n", ingredient.get(0).unwrap().as_str())[..]);
        }

        // Send the ingredients to nutritionIX
        let url = Url::parse("https://trackapi.nutritionix.com/v2/natural/nutrients")?;
        let nutrition_app_id = env::var("NUTRITIONIX_APP_ID")?;
        let nutrition_app_key = env::var("NUTRITIONIX_APP_KEY")?;
        
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("x-app-id", HeaderValue::from_str(nutrition_app_id.as_str())?);
        headers.insert("x-app-key", HeaderValue::from_str(nutrition_app_key.as_str())?);
        
        let mut body = HashMap::new();
        body.insert("query", ingredients_as_str);
        
        let client = Client::new();
        let request_builder = client.post(url)
            .json(&body)
            .headers(headers);

        let response = request_builder.send().await?;
        let response_as_str: String = response.text().await?;

        // Deserialize the JSON into Rust structs
        let data: NutritionResponse = serde_json::from_str(response_as_str.as_str()).expect(response_as_str.as_str());
        self.ingredients = data.foods;

        // Get the date
        self.date = chrono::offset::Local::now();

        Ok(())
    }
}

impl Markdown for Recipe {
    fn as_markdown(&self) -> String {
        let mut markdown: String = String::new();
        
        // Get table of ingredients
        let mut table: String = String::new();
        table.push_str("| INGREDIENT | CALORIES | PROTEIN | CARBS | FAT |\n");
        table.push_str("| - | - | - | - | - |\n");
        let mut total_calories: f32 = 0.0;
        let mut total_protein: f32 = 0.0;
        let mut total_carbs: f32 = 0.0;
        let mut total_fat: f32 = 0.0;
        for food in &self.ingredients {
            table.push_str(&format!("| {} {} {} | {} | {} | {} | {} |\n", food.serving_qty, food.serving_unit, food.food_name, food.nf_calories, food.nf_protein, food.nf_total_carbohydrate, food.nf_total_fat)[..]);
            total_calories += food.nf_calories; 
            total_protein += food.nf_protein; 
            total_carbs += food.nf_total_carbohydrate; 
            total_fat += food.nf_total_fat;
        }
        markdown.push_str(&table.as_str()[..]);

        // Get table of total nutrients
        let mut tot_table: String = String::new();
        tot_table.push_str("\n| NUTRIENT | AMOUNT |\n");
        tot_table.push_str("| - | - |\n");
        tot_table.push_str(&format!("| CALORIES | {} |\n", total_calories)[..]);
        tot_table.push_str(&format!("| PROTEIN | {} |\n", total_protein)[..]);
        tot_table.push_str(&format!("| CARBS | {} |\n", total_carbs)[..]);
        tot_table.push_str(&format!("| FAT | {} |\n", total_fat)[..]);
        markdown.push_str(&tot_table.as_str()[..]);

        // Get directions
        for direction in &self.directions {
            markdown.push_str(&format!("{}\n", direction).as_str()[..]);
        }

        markdown
    }
}