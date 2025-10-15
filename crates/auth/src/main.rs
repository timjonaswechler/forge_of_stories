use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::{Value, json};

const BASE_URL: &str = "https://auth.chicken105.com";

#[derive(Debug)]
pub struct Session {
    pub user_id: String,
    pub user_name: String,
    pub expires_at: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
struct LoginFlowResponse {
    id: String,
}
#[tokio::main]
async fn main() {
    let session = login("admin@chicken105.com", "Admin123!").await;
    match session {
        Ok(session) => {
            println!("{:?}", session);
            validate_session(session.token).await;
        }
        Err(err) => println!("Error: {:?}", err),
    }
    // validate_session(session).await;
}

pub async fn login(
    user_identifier: &str,
    password: &str,
) -> Result<Session, Box<dyn std::error::Error>> {
    let flow_url = format!("{}/self-service/login/api", BASE_URL);
    let login_url = format!("{}/self-service/login", BASE_URL);

    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("application/json"));

    #[cfg(debug_assertions)]
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    #[cfg(not(debug_assertions))]
    let client = Client::new();

    let flow_response = client.get(flow_url).headers(headers.clone()).send().await?;
    let flow: LoginFlowResponse = flow_response.json().await?;

    let payload = json!({
        "method": "password",
        "identifier": user_identifier,
        "password": password
    });

    let login_response = client
        .post(login_url)
        .query(&[("flow", flow.id.as_str())])
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let login_json: Value = login_response.json().await?;
    let session = Session {
        user_id: login_json["session"]["identity"]["id"]
            .as_str()
            .unwrap()
            .to_string(),
        user_name: login_json["session"]["identity"]["traits"]["name"]
            .as_str()
            .unwrap()
            .to_string(),
        expires_at: login_json["session"]["expires_at"]
            .as_str()
            .unwrap()
            .to_string(),
        token: login_json["session_token"].as_str().unwrap().to_string(),
    };
    Ok(session)
}

// TODO: Error handling for validate_session
// if session ist valid Result is
/*
 * results = Object {
 *   "active": Bool(true),
 *   ...
 */
// if session failes Result is
/*
 * results = Object {
 *   "error": Object {
 *       "code": Number(401),
 *       "message": String("The request could not be authorized"),
 *       "reason": String("No valid session credentials found in the request."),
 *       "status": String("Unauthorized"),
 *   },
 * }
 */
pub async fn validate_session(token: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/sessions/whoami", BASE_URL);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());
    headers.insert(
        "authorization",
        // "Bearer ory_st_NtuKEzWKgKCQt8PLkDpBn0vmvgb79uRa"
        format!("Bearer {}", token).parse().unwrap(),
    );

    #[cfg(debug_assertions)]
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    #[cfg(not(debug_assertions))]
    let client = Client::new();

    let response = client.get(url).headers(headers).send().await;

    let results = response.unwrap().json::<serde_json::Value>().await.unwrap();

    dbg!(results);
    Ok(())
}

// to refreash a session use the following code:
// #[tokio::main]
// pub async fn main() {
//     let url = "https://auth.chicken105.com/self-service/login/api";

//     let querystring = [("refresh", "true")];

//     let mut headers = reqwest::header::HeaderMap::new();
//     headers.insert("content-type", "application/json".parse().unwrap());
//     headers.insert(
//         "authorization",
//         "Bearer ory_st_ImVadzAaXAAS0vlHTjoMXNQVFZHEB6EE"
//             .parse()
//             .unwrap(),
//     );

//     let client = reqwest::Client::new();
//     let response = client
//         .get(url)
//         .query(&querystring)
//         .headers(headers)
//         .send()
//         .await;

//     let results = response.unwrap().json::<serde_json::Value>().await.unwrap();

//     dbg!(results);
// }
