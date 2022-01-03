use hyper::{
    body::{self, Buf},
    client::HttpConnector,
    Body, Client, Method, Request,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use lol_quotes::ddragon::{ChampionData, ChampionExport};
use tokio::fs;
use tracing::{error, info};

type HyperClient = Client<HttpsConnector<HttpConnector>>;

const VERSIONS_ENDPOINT: &str = "https://ddragon.leagueoflegends.com/api/versions.json";

async fn get_versions(client: &HyperClient) -> Result<Vec<String>, lol_quotes::Error> {
    let req = Request::builder()
        .method(Method::GET)
        .uri(VERSIONS_ENDPOINT)
        .body(Body::empty())?;

    let res = client.request(req).await?;

    let body = body::aggregate(res).await?;

    Ok(simd_json::from_reader(body.reader())?)
}

async fn get_champions(
    client: &HyperClient,
    version: &str,
) -> Result<ChampionData, lol_quotes::Error> {
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json",
            version
        ))
        .body(Body::empty())?;

    let res = client.request(req).await?;

    let body = body::aggregate(res).await?;

    Ok(simd_json::from_reader(body.reader())?)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting champion update");

    let connector = HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .enable_http2()
        .build();
    let client: Client<_> = Client::builder().build(connector);

    let versions = match get_versions(&client).await {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to get versions: {:?}", e);
            return;
        }
    };

    let latest_version = match versions.get(0) {
        Some(version) => version,
        None => {
            error!("No versions found");
            return;
        }
    };

    info!("Latest game version is {}", latest_version);

    let champions = match get_champions(&client, latest_version).await {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to get champions: {:?}", e);
            return;
        }
    };

    info!("Found {} champions, exporting", champions.data.len());

    let export_champions: Vec<ChampionExport> = champions
        .data
        .into_iter()
        .map(|(_, champion)| ChampionExport {
            name: champion.name,
            id: champion.id,
            icon: format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/champion/{}",
                latest_version, champion.image.full
            ),
        })
        .collect();

    info!("Writing results to champions.json");

    let json = match simd_json::to_vec_pretty(&export_champions) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize champion export: {:?}", e);
            return;
        }
    };

    match fs::write("champions.json", json).await {
        Ok(_) => info!("Data written to champions.json"),
        Err(e) => error!("Failed to write to champions.json: {}", e),
    };
}
