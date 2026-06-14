use std::error::{self};

use twitch_emote_parser_rs::TwitchEmoteParser;

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = TwitchEmoteParser::new("token", "client-id").await?;

    let emote = parser.populate_string("LUL whats up Kappa");

    println!("{:#?}", emote);

    Ok(())
}
