use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use futures::stream;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    event_name: String,
}

async fn collect_metrics(
    data: web::Json<Data>,
    tx: web::Data<tokio::sync::mpsc::UnboundedSender<Data>>,
) -> impl Responder {
    if tx.send(data.into_inner()).is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().body("OK")
}

async fn write_metrics(data: &Vec<DataPoint>) {
    dotenv().ok();
    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
    let bucket = std::env::var("INFLUXDB_BUCKET").unwrap();

    let client = Client::new(host, org, token);

    let mut data_vec = vec![];
    for data_point in data.iter() {
        data_vec.push(data_point.clone());
    }

    client
        .write(&bucket, stream::iter(data_vec))
        .await
        .expect("writefailed");
    println!("influx write");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Data>();
    let tx = web::Data::new(tx);

    let thread_http_server = HttpServer::new(move || {
        let tx_ref = tx.clone();
        App::new().app_data(tx_ref.clone()).route(
            "/logger_v1/event/metrics",
            web::post().to(move |data: web::Json<Data>| collect_metrics(data, tx_ref.clone())),
        )
    })
    .bind("0.0.0.0:8081")?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.spawn(async move {
        let mut metrix_v: Vec<DataPoint> = Vec::new();
        while let Some(data) = rx.recv().await {
            let data_point = DataPoint::builder("<event_name>")
                .tag("<event_value>", data.event_name)
                //If there is no unique_value during large-capacity insertion, the same event is treated as one.
                .tag("<unique_value>", Uuid::new_v4().to_string())
                .field("value", 1)
                .build()
                .unwrap();
            metrix_v.push(data_point);
            if metrix_v.len() > 1000 {
                write_metrics(&metrix_v).await;
                metrix_v = Vec::new();
            }
        }
    });
    thread_http_server.run().await?;
    Ok(())
}
