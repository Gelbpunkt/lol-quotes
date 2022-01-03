use std::collections::HashMap;

use hyper::{body, client::HttpConnector, Body, Client, Method, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use lol_quotes::ddragon::{ChampionExport, QuoteExport};
use regex::{Captures, RegexBuilder};
use tokio::fs;
use tracing::{error, info};

type HyperClient = Client<HttpsConnector<HttpConnector>>;

async fn get_quotes(client: &HyperClient, name: &str) -> Result<Vec<String>, lol_quotes::Error> {
    let wiki_url = format!(
        "https://leagueoflegends.fandom.com/wiki/{}/LoL/Audio?action=raw",
        name.replace(" ", "_")
    );

    let req = Request::builder()
        .method(Method::GET)
        .uri(wiki_url)
        .body(Body::empty())?;

    let res = client.request(req).await?;

    let body = body::to_bytes(res).await?;
    let text = String::from_utf8(body.to_vec()).unwrap_or_default();

    Ok(parse_quotes(text, name))
}

fn parse_quotes(text: String, name: &str) -> Vec<String> {
    let small_bold_caps_regex = RegexBuilder::new(r"\{\{sbc\|([^}]+)\}\}")
        .multi_line(true)
        .build()
        .unwrap();

    let champion_inline_regex =
        RegexBuilder::new(r"\{\{ci\|(?P<champion>[^}|]+)(?:\|(?P<custom_name>[^}]+))?\}\}")
            .multi_line(true)
            .build()
            .unwrap();

    let link_regex = RegexBuilder::new(r"\[\[(?P<page>[^]|]+)(?:\|(?P<link_text>[^]]+))?]]")
        .build()
        .unwrap();

    let image_regex = RegexBuilder::new(r" ?\[\[File:[^[]+]] ?").build().unwrap();

    let ability_regex = RegexBuilder::new(
        r"\{\{ai\|(?P<ability>[^}|]+)(?:\|(?P<champion>[^}|]+))?(?:\|(?P<display_name>[^}]+))?\}\}",
    )
    .build()
    .unwrap();

    let rp_regex = RegexBuilder::new(r"\{\{RP([^}]*)\}\}").build().unwrap();

    let summoner_spell_regex = RegexBuilder::new(r"\{\{si\|([^}]+)\}\}").build().unwrap();

    let ccib_regex = RegexBuilder::new(
        r"\{\{[cC]cib?\|(?P<file>[^}|]+)\|(?P<link>[^}|]+)(?:\|(?P<display_name>[^}|]+))?\}\}",
    )
    .build()
    .unwrap();

    let as_regex = RegexBuilder::new(r"\{\{[Aa]s\|([^}|]+)\}\}")
        .build()
        .unwrap();

    let sti_regex =
        RegexBuilder::new(r"\{\{sti\|(?P<attribute>[^}|]+)(?:\|(?P<display_name>[^}|]+))?\}\}")
            .build()
            .unwrap();

    let bi_regex =
        RegexBuilder::new(r"\{\{bi\|(?P<buff>[^}|]+)(?:\|(?P<display_name>[^}|]+))?\}\}")
            .build()
            .unwrap();

    let tt_regex = RegexBuilder::new(r"\{\{tt\|(?P<text>[^}|]+)\|(?P<hover>[^}|]+)\}\}")
        .build()
        .unwrap();

    let ui_regex =
        RegexBuilder::new(r"\{\{ui\|(?P<unit>[^}|]+)(?:\|(?P<display_name>[^}|]+))?\}\}")
            .build()
            .unwrap();

    let csl_regex = RegexBuilder::new(
        r"\{\{csl\|(?P<champ>[^}|]+)(?:\|(?P<skin>[^}|]+))?(?:\|(?P<display_name>[^}|]+))?\}\}",
    )
    .build()
    .unwrap();

    let fi_regex =
        RegexBuilder::new(r"\{\{fi\|(?P<faction>[^}|]+)(?:\|(?P<display_name>[^}|]+))?\}\}")
            .build()
            .unwrap();

    let text = small_bold_caps_regex.replace_all(&text, |caps: &Captures| caps[1].to_uppercase());

    let text = champion_inline_regex.replace_all(&text, |caps: &Captures| {
        caps.name("custom_name")
            .unwrap_or_else(|| caps.name("champion").unwrap())
            .as_str()
            .to_string()
    });

    let text = link_regex.replace_all(&text, |caps: &Captures| {
        let page = caps.name("page").unwrap();
        if page.as_str().starts_with("File:") {
            return caps[0].to_string();
        }

        caps.name("link_text").unwrap_or(page).as_str().to_string()
    });

    let text = image_regex.replace_all(&text, " ");

    let text = ability_regex.replace_all(&text, |caps: &Captures| {
        caps.name("display_name")
            .unwrap_or_else(|| caps.name("ability").unwrap())
            .as_str()
            .to_string()
    });

    let text = rp_regex.replace_all(&text, "RP");

    let text = summoner_spell_regex.replace_all(&text, |caps: &Captures| caps[1].to_string());

    let text = ccib_regex.replace_all(&text, |caps: &Captures| {
        caps.name("display_name")
            .unwrap_or_else(|| caps.name("link").unwrap())
            .as_str()
            .to_string()
    });

    let text = as_regex.replace_all(&text, |caps: &Captures| caps[1].to_string());

    let text = sti_regex.replace_all(&text, |caps: &Captures| {
        caps.name("display_name")
            .unwrap_or_else(|| caps.name("attribute").unwrap())
            .as_str()
            .to_string()
    });

    let text = bi_regex.replace_all(&text, |caps: &Captures| {
        caps.name("display_name")
            .unwrap_or_else(|| caps.name("buff").unwrap())
            .as_str()
            .to_string()
    });

    let text = tt_regex.replace_all(&text, |caps: &Captures| {
        caps.name("text").unwrap().as_str().to_string()
    });

    let text = ui_regex.replace_all(&text, |caps: &Captures| {
        caps.name("display_name")
            .unwrap_or_else(|| caps.name("unit").unwrap())
            .as_str()
            .to_string()
    });

    let text = csl_regex.replace_all(&text, |caps: &Captures| {
        caps.name("display_name")
            .unwrap_or_else(|| {
                caps.name("skin")
                    .unwrap_or_else(|| caps.name("champ").unwrap())
            })
            .as_str()
            .to_string()
    });

    let text = fi_regex.replace_all(&text, |caps: &Captures| {
        caps.name("display_name")
            .unwrap_or_else(|| caps.name("faction").unwrap())
            .as_str()
            .to_string()
    });

    let mut matches = Vec::new();

    if name != "Kindred" {
        let re = RegexBuilder::new(r#"''"(.*)"''"#)
            .multi_line(true)
            .build()
            .unwrap();
        for captures in re.captures_iter(&text) {
            let capture = &captures[1];

            if !capture.contains("ogg") && capture != "GG!" {
                let text = capture.replace("'''", "**");
                matches.push(text);
            }
        }
    } else {
        let re = RegexBuilder::new(r#"(Wolf|Lamb|Kindred): ''(?:")?([^"]+)?(?:")?''"#)
            .build()
            .unwrap();
        let mut last_asterisk_count = 1;

        for line in text.lines() {
            let cleaned = line.trim();
            let mut asterisk_count_this_line = 0;

            for letter in cleaned.chars() {
                if letter == '*' {
                    asterisk_count_this_line += 1;
                } else {
                    break;
                }
            }

            for capture in re.captures_iter(&cleaned) {
                let quote = format!("{}: {}", &capture[1], &capture[2]).replace("'''", "**");

                if asterisk_count_this_line == last_asterisk_count + 1 {
                    let idx = matches.len() - 1;
                    matches[idx].push_str("\n");
                    matches[idx].push_str(&quote);
                } else {
                    matches.push(quote);
                }

                last_asterisk_count = asterisk_count_this_line;
            }
        }
    }

    matches
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting quote update");

    let connector = HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .enable_http2()
        .build();
    let client: Client<_> = Client::builder().build(connector);

    info!("Loading champion data from champions.json");

    let mut data = match fs::read_to_string("champions.json").await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to read champions.json: {}", e);
            return;
        }
    };

    let champions: Vec<ChampionExport> = match simd_json::from_str(&mut data) {
        Ok(champions) => champions,
        Err(e) => {
            error!("Failed to parse champions.json: {}", e);
            return;
        }
    };

    let champion_count = champions.len();
    let mut champion_quote_data = HashMap::with_capacity(champion_count);

    for (idx, champion) in champions.into_iter().enumerate() {
        info!(
            "Getting quotes for {} ({}/{})",
            champion.name,
            idx + 1,
            champion_count
        );

        let name_to_use = if champion.name.contains("&") {
            &champion.id
        } else {
            &champion.name
        };

        let quotes = match get_quotes(&client, &name_to_use).await {
            Ok(quotes) => quotes,
            Err(e) => {
                error!("Failed to get quotes for {}: {:?}", champion.name, e);
                return;
            }
        };

        let champion_quotes = QuoteExport {
            quotes,
            icon: champion.icon,
        };

        champion_quote_data.insert(champion.name, champion_quotes);
    }

    info!("Writing results to quotes.json");

    let json = match simd_json::to_vec_pretty(&champion_quote_data) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize quote export: {:?}", e);
            return;
        }
    };

    match fs::write("quotes.json", json).await {
        Ok(_) => info!("Data written to quotes.json"),
        Err(e) => error!("Failed to write to quotes.json: {}", e),
    };
}
