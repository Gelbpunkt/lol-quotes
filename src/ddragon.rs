use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ChampionData {
    pub r#type: String,
    pub format: String,
    pub version: String,
    pub data: HashMap<String, Champion>,
}

#[derive(Debug, Deserialize)]
pub struct Champion {
    pub version: String,
    pub id: String,
    pub key: String,
    pub name: String,
    pub title: String,
    pub blurb: String,
    pub info: ChampionInfo,
    pub image: ChampionImage,
    pub tags: Vec<String>,
    pub partype: String,
    pub stats: ChampionStats,
}

#[derive(Debug, Deserialize)]
pub struct ChampionInfo {
    pub attack: u8,
    pub defense: u8,
    pub magic: u8,
    pub difficulty: u8,
}

#[derive(Debug, Deserialize)]
pub struct ChampionImage {
    pub full: String,
    pub sprite: String,
    pub group: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
pub struct ChampionStats {
    pub hp: f32,
    pub hpperlevel: f32,
    pub mp: f32,
    pub mpperlevel: f32,
    pub movespeed: f32,
    pub armor: f32,
    pub armorperlevel: f32,
    pub spellblock: f32,
    pub spellblockperlevel: f32,
    pub attackrange: u16,
    pub hpregen: f32,
    pub hpregenperlevel: f32,
    pub mpregen: f32,
    pub mpregenperlevel: f32,
    pub crit: u16,
    pub critperlevel: u8,
    pub attackdamage: f32,
    pub attackdamageperlevel: f32,
    pub attackspeedperlevel: f32,
    pub attackspeed: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChampionExport {
    pub name: String,
    pub id: String,
    pub icon: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuoteExport {
    pub quotes: Vec<String>,
    pub icon: String,
}
