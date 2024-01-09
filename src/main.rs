use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    Client,
    Error,
    types::AttributeValue,
};
use std::collections::HashMap;

#[derive(Debug)]
enum ListItemsResult {
    Array(Vec<HashMap<String, AttributeValue>>),
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World"); format!("Hello {}!", &name)
}

async fn solar_systems(req: HttpRequest) -> impl Responder {
    let partition_key_value = req.match_info().get("id").unwrap_or_default();
    dbg!(partition_key_value);

    let result = match fetch_solar_system_by_id(partition_key_value).await {
        Ok(result) => result,
        Err(error) => {
            return format!("Error: {:?}", error);
        },
    };

    match result {
        ListItemsResult::Array(items) => {
            let mut response = String::new();
            for item in items {
                response.push_str(&format!("{:?}\n", item));
            }
            response
        },
    }
}

async fn fetch_solar_system_by_id(partition_key_value: &str) -> Result<ListItemsResult, Error> {
    let shared_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&shared_config);

    let result = client
        .query()
        .table_name("DynamoDan")
        .key_condition_expression("#pk = :pk")
        .expression_attribute_names("#pk", "id")
        .expression_attribute_values(":pk", AttributeValue::S(partition_key_value.to_string()))
        .send()
        .await?;

    if let Some(items) = result.items {
        return Ok(ListItemsResult::Array(items));
    }

    Ok(ListItemsResult::Array(vec![]))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
HttpServer::new(|| {
    App::new()
        .route("/", web::get().to(greet))
        .route("/solar_systems/{id}", web::get().to(solar_systems))
        .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8999")?
    .run()
    .await
}

