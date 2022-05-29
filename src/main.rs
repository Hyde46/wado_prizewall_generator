use image::RgbaImage;
use image::{imageops, DynamicImage, Rgba};
use image::io::Reader as ImageReader;
use imageproc::drawing::{draw_text_mut, text_size};
use reqwest;
use rusttype::{Font, Scale};
use scryfall::card::Card;
use std::env;
use std::path::Path;

use std::error::Error;
use std::io;
use std::process;

use serde::Deserialize;

static CARDS_PATH: &str = "imgs/cards";
static BG_PATH: &str = "imgs/bg";

/// Layout calculations
const WIDTH: u32 = 3508;
//const WIDTH: u32 = 350;
const HEIGHT: u32 = 4961;
//const HEIGHT: u32 = 496;

const IMG_ROWS: u32 = 3;
const IMGS_PER_COLUMN: u32 = 4;

const MARGIN_TOP: u32 = (HEIGHT as f64 * 0.08) as u32;
const MARGIN_LEFT: u32 = (WIDTH as f64 * 0.07) as u32;

const CARD_PADDING_RIGHT: u32 = (WIDTH as f64 * 0.08) as u32;

const CARD_WIDTH: u32 = (WIDTH - (2 * MARGIN_LEFT) - ((IMGS_PER_COLUMN - 1) * CARD_PADDING_RIGHT))
    / IMGS_PER_COLUMN as u32;
const CARD_HEIGHT: u32 = ((889_f64 / 635_f64) * CARD_WIDTH as f64) as u32;

const CARD_PADDING_BOTTOM: u32 =
    (HEIGHT - (MARGIN_TOP * 2) - (IMG_ROWS * CARD_HEIGHT)) / IMG_ROWS as u32;

static SCRYFALL_BASE_URI: &str = "https://c1.scryfall.com";

static WADO_POINT_SUFFIX: &str = "WP";

const FONT_HEIGHT: f32 = CARD_HEIGHT as f32 * 0.1;
const FONT_SCALE: rusttype::Scale = Scale {
    x: FONT_HEIGHT * 1.2,
    y: FONT_HEIGHT,
};

#[derive(Debug, Deserialize)]
struct CSVCard {
    pub name: String,
    pub condition: String,
    pub set: String,
    pub language: String,
    pub eur: String,
    pub rwp: String,
    pub wp: u32,
    pub display: Option<String>,
}

fn place_card_images(mut base: DynamicImage, imgs: &[DynamicImage]) -> DynamicImage {
    let mut img_row_count = 0;
    let mut img_col_count = 0;
    for img in imgs {
        // In case the downloaded images have different sizes. This can happen on scryfall for
        // older images _sometimes_
        let resized = imageops::resize(img, CARD_WIDTH, CARD_HEIGHT, imageops::FilterType::Nearest);
        // Calculate position to place image
        let x_pos =
            MARGIN_LEFT + (img_col_count * CARD_WIDTH) + (img_col_count * CARD_PADDING_RIGHT);
        let y_pos =
            MARGIN_TOP + (img_row_count * CARD_HEIGHT) + (img_row_count * CARD_PADDING_BOTTOM);
        // Place image by overlaying with base
        imageops::overlay(&mut base, &resized, x_pos.into(), y_pos.into());
        // Update row and column for next image to be placed
        img_col_count += 1;
        if img_col_count == (IMGS_PER_COLUMN) {
            img_col_count = 0;
            img_row_count += 1;
        }
    }
    base
}

fn request_card_uris(card_names: &Vec<CSVCard>) -> Vec<String> {
    let mut card_uris: Vec<String> = Vec::new();

    for card in card_names {
        match Card::named_fuzzy(&card.name) {
            Ok(card) => {
                card_uris.push(format!(
                    "{}{}",
                    SCRYFALL_BASE_URI,
                    card.image_uris["normal"].path()
                ));
            }
            Err(e) => panic!("{}", format!("Could not find card, {:?}", e)),
        }
    }
    card_uris
}

fn load_image_from_uri(card_names: Vec<String>) -> Vec<DynamicImage> {
    let mut images: Vec<DynamicImage> = Vec::new();
    for card_uri in card_names {
        let img_bytes = reqwest::blocking::get(card_uri).unwrap().bytes().unwrap();
        let image = image::load_from_memory(&img_bytes).unwrap();
        images.push(image);
    }
    images
}

fn draw_text_to_image(image: &mut RgbaImage, cards: &[CSVCard]) -> RgbaImage {
    let font = Vec::from(include_bytes!("Jupitex Sans Serif.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    let mut row = 0;
    let mut col = 0;
    let font_color = [230u8, 230u8, 230u8, 255u8];
    for card in cards {
        let title = card.display.as_ref().unwrap_or(&card.name);
        let set = format!("{} ({}) {}",&card.set, &card.language, &card.condition);
        let price = format!("{} {}",card.wp, WADO_POINT_SUFFIX);

        let x_start = MARGIN_LEFT + col * (CARD_WIDTH + CARD_PADDING_RIGHT);  
        let x_center = x_start + (CARD_WIDTH as f32 / 2.) as u32;

        let (w, h) = text_size(FONT_SCALE, &font, title);
        let font_padding = (h as f64 / 2.) as i32;
        let y_start = MARGIN_TOP + row * (CARD_HEIGHT + CARD_PADDING_BOTTOM) + CARD_HEIGHT + h as u32;
        //Title
        let x = x_center - (w as f32 / 2.).min(x_start as f32) as u32;
        let y = y_start;
        draw_text_mut(image, Rgba(font_color), x as i32, y as i32, FONT_SCALE, &font, title);
        //Set
        let (w, _h) = text_size(FONT_SCALE, &font, &set);
        let x = x_center - (w as f32 / 2.).min(x_center as f32) as u32;
        let y = y_start;
        draw_text_mut(image, Rgba(font_color), x as i32, y as i32 + h + font_padding, FONT_SCALE, &font, &set);
        //Price
        let (w, _h) = text_size(FONT_SCALE, &font, &price);
        let x = x_center - (w as f32 / 2.) as u32;
        let y = y_start;
        draw_text_mut(image, Rgba(font_color), x as i32, y as i32 + 2 * ( font_padding + h), FONT_SCALE, &font, &price);
        col += 1;
        if col == IMGS_PER_COLUMN {
            col = 0;
            row +=1;
        }
    }
    image.to_owned()
}

fn main() {
    // Read out filename argument, defualts to "card_list.csv"
    fn default_filename() -> Option<String> {
        Some("card_list.csv".to_string())
    }
    let filename = std::env::args().nth(1).or_else(default_filename).unwrap();

    println!("Reading from {}", filename);

    let mut csv_cards: Vec<CSVCard> = Vec::new();
    //Read out cards from csv
    println!("Reading cards from file");
    let mut rdr = csv::Reader::from_path(filename).unwrap();
    //    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let record: CSVCard = result.unwrap();
        csv_cards.push(record);
    }
    println!("Found {} cards", &csv_cards.len());
    // Get card images
    println!("Requesting images from Scryfall ...");
    let card_imgs = load_image_from_uri(request_card_uris(&csv_cards));

    let cards_per_page = IMGS_PER_COLUMN * IMG_ROWS;
    let num_pages = (card_imgs.len() as f64 / cards_per_page as f64).ceil() as u32;
    println!("Creating {} prizewall pages", num_pages);

    for i in 0..num_pages {
        // Generate the image with all magic cards
        println!("Generating base image");
        let mut base = DynamicImage::new_rgba16(WIDTH, HEIGHT);
        // TODO: Place background image
        let bg = ImageReader::open("img/blue.png").unwrap().decode().unwrap();
        imageops::overlay(&mut base, &bg, 0, 0);

        // Place cards on base
        let lower: usize = (i * cards_per_page) as usize;
        let upper: usize =
            (i * cards_per_page + cards_per_page).min((card_imgs.len()) as u32) as usize;
        println!("Using cards from {} to {}", lower, upper);
        println!("Placing cards on page");
        let card_img_slice = &card_imgs[lower..upper];
        let base_with_cards = place_card_images(base, card_img_slice);

        // Write text under each card
        let mut image: RgbaImage = base_with_cards.to_rgba8();
        let csv_card_slice = &csv_cards[lower..upper];
        let final_image = draw_text_to_image(&mut image, csv_card_slice);
        final_image.save(format!("output/prizewall_p{}.png", i))
            .unwrap();

    }
}
