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
};
use serde_json::Value;
use regex::Regex;

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
    nf_sugars: f32,
    nf_protein: f32,
    nf_potassium: f32,
    nf_p: f32,
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
    value: f32
}

pub struct Recipe {
    pub website: Website,
    pub ingredients: Vec<Food>,
    pub directions: Vec<String>,
    pub chatgpt_response: String,
}

impl Recipe {
    pub fn new(website: Website) -> Recipe {
        Recipe {
            website: website,
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
        let message = format!("What is the list of ingredients and the directions of this recipe?:\n{}", self.website.plaintext);
        let chatgpt_response = client.send_message(message).await?;
        self.chatgpt_response = chatgpt_response.message()
            .content
            .clone();

        // Parse the directions
        let find_directions = Regex::new(r#"[1-9]\..*"#).unwrap();
        for direction in find_directions.captures_iter(self.chatgpt_response.as_str()) {
            self.directions.push(String::from(direction.get(0).unwrap().as_str()));
        }

        // Parse the ingredients
        let find_ingredients = Regex::new(r#"-.*"#).unwrap();
        let mut ingredients_as_str = String::new();
        for ingredient in find_ingredients.captures_iter(self.chatgpt_response.as_str()) {
            ingredients_as_str.push_str(&format!("{}\n", ingredient.get(0).unwrap().as_str())[..]);
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
        let data: NutritionResponse = serde_json::from_str(response_as_str.as_str())?;
        self.ingredients = data.foods;

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