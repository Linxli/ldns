use actix_web::{self, App, HttpResponse, HttpServer, Responder, put, web};
use serde::{Deserialize, Serialize};

// Import blocklookup to download and load the blocklist
use crate::blocklookup;

// ===== REQUEST TYPES =====

/// Request body for updating the blocklist URL
#[derive(Deserialize, Serialize)]
struct BlocklistUpdateRequest {
    url: String,
}

// ===== RESPONSE TYPES =====

/// Success response when blocklist is updated
#[derive(Serialize)]
struct BlocklistUpdateResponse {
    message: String,
    url: String,
    domains_loaded: usize,
}

/// Error response for failed requests
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// ===== API ENDPOINTS =====

/// PUT /blocklist - Update the blocklist URL and reload domains
///
/// This endpoint:
/// 1. Validates the URL format
/// 2. Downloads the blocklist from the new URL
/// 3. Parses and loads it into memory
/// 4. Returns stats about the loaded blocklist
#[put("/blocklist")]
async fn update_blocklist(msg: web::Json<BlocklistUpdateRequest>) -> impl Responder {
    let url = &msg.url;

    // Step 1: Validate URL format
    // The url crate's parse() returns Result<Url, ParseError>
    // We use is_err() to check if parsing failed
    if reqwest::Url::parse(url).is_err() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Invalid URL format: {}", url),
        });
    }

    // Step 2: Download the blocklist
    // reqwest is async, so we use .await
    // Building a client with custom user-agent (good HTTP etiquette!)
    let client = match reqwest::Client::builder()
        .user_agent("ldns-api/1.0")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to create HTTP client: {}", e),
            });
        }
    };

    // Make the HTTP request
    let response = match client.get(url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Failed to download blocklist: {}", e),
            });
        }
    };

    // Check if the response was successful (2xx status code)
    if !response.status().is_success() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Server returned error: {}", response.status()),
        });
    }

    // Get the response body as bytes
    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to read response body: {}", e),
            });
        }
    };

    // Step 3: Load into memory using our blocklookup module
    // This calls the load_file function which:
    // - Parses the raw text into a HashSet
    // - Acquires the write lock
    // - Updates the global BLOCKLIST
    // - DNS queries wait briefly during this update
    let domains_loaded = blocklookup::load_file(Some(bytes.to_vec())).await;

    // Step 4: Return success response with stats
    // Now we can tell the user exactly how many domains were loaded!
    HttpResponse::Ok().json(BlocklistUpdateResponse {
        message: "Blocklist updated successfully".to_string(),
        url: url.to_string(),
        domains_loaded,
    })
}

/// Public function to get the update_blocklist service for testing
///
/// We expose this so tests can create an App with our endpoints
/// without starting a real HTTP server
#[allow(dead_code)]
pub fn update_blocklist_endpoint() -> actix_web::Scope {
    actix_web::web::scope("").service(update_blocklist)
}

pub async fn server() {
    if let Err(e) = HttpServer::new(|| App::new().service(update_blocklist))
        .bind("0.0.0.0:8080")
        .expect("An Error occurred while setting up the API webserver.")
        .run()
        .await
    {
        eprintln!(
            "An Error occurred while setting up the API webserver: {}",
            e
        );
    }
}
