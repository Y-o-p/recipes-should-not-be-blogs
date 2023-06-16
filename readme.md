# recipes-should-not-be-blogs

Gets rid of those pesky blogs filling up recipes nowadays.

## How it Works

1. Takes a URL as input 
2. Web scrapes all plain text with [ToolsYEP](https://toolsyep.com/en/webpage-to-plain-text/)
3. Parses the plain text with ChatGPT
4. Gathers nutrtional information about ingredients with NutritionIX
5. Formats everything into a markdown document.

## Example

```bash
$ cargo run -- https://www.allrecipes.com/recipe/20144/banana-banana-bread/
```

### recipe.md

| INGREDIENT | CALORIES | PROTEIN | CARBS | FAT |
| - | - | - | - | - |
| 2 cups all purpose flour | 910 | 25.83 | 190.78 | 2.45 |
| 1 teaspoon baking soda | 0 | 0 | 0 | 0 |
| 0.25 teaspoon salt | 0 | 0 | 0 | 0 |
| 0.75 cup brown sugar | 520.13 | 0.16 | 134.26 | 0 |
| 0.5 cup butter | 813.8 | 0.96 | 0.07 | 92.06 |
| 2 large eggs | 143 | 12.56 | 0.72 | 9.51 |
| 2.33 cups bananas | 388.82 | 4.76 | 99.78 | 1.44 |

| NUTRIENT | AMOUNT |
| - | - |
| CALORIES | 2775.75 |
| PROTEIN | 44.269997 |
| CARBS | 425.61 |
| FAT | 105.46 |
1. Preheat the oven to 350 degrees F (175 degrees C). Lightly grease a 9x5-inch loaf pan.
2. Combine flour, baking soda, and salt in a large bowl.
3. Beat brown sugar and butter with an electric mixer in a separate large bowl until smooth. Stir in eggs and mashed bananas until well blended.
4. Stir banana mixture into flour mixture until just combined.
5. Pour batter into the prepared loaf pan.
6. Bake in the preheated oven until a toothpick inserted into the center comes out clean, about 60 minutes.
7. Let bread cool in pan for 10 minutes, then turn out onto a wire rack to cool completely.

## How to Build

You'll need:
* [OpenAI API account](https://openai.com/blog/openai-api)
* [NutritionIX API account](https://www.nutritionix.com/business/api)
* [Rust](https://www.rust-lang.org/tools/install)

Steps:
1. Clone the project
2. Create a `.env` file in the root folder
3. Define the following keys
```
OPENAI_KEY="12345"
NUTRITIONIX_APP_ID="12345"
NUTRITIONIX_APP_KEY="12345"
NUTRITIONIX_APP_USERID="name" 
```
4. Try a recipe
```bash
$ cargo run -- URL
```