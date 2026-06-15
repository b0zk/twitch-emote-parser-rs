use std::error::{self};

use twitch_emote_parser_rs::TwitchEmoteParser;

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = TwitchEmoteParser::new("token", "client_id").await?;

    parser.add_channel("broadcaster_id").await?;

    let emote = parser.populate_string("LUL whats yusuf7Ngang1 up Kappa");

    println!("{:#?}", emote);

    Ok(())
}
