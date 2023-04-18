use std::default;

use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use chrono::{DateTime, TimeZone, Utc};
use dotenvy::dotenv;
use fake::faker::{
    address::en::{
        CountryCode, CountryName, Geohash, SecondaryAddress, SecondaryAddressType, StateAbbr,
        StateName, StreetName, StreetSuffix,
    },
    automotive::en::LicencePlate,
    chrono::en::{DateTimeAfter, DateTimeBetween},
    color::en::{Color, HslColor, RgbColor, RgbaColor},
    company::en::{
        Bs, BsAdj, BsNoun, BsVerb, Buzzword, BuzzwordMiddle, BuzzwordTail, CatchPhase, CompanyName,
        CompanySuffix, Profession,
    },
    filesystem::en::{DirPath, FileExtension, FileName, FilePath},
    finance::raw::Bic,
    http::en::ValidStatusCode,
    lorem::raw::Words,
};
use fake::faker::{
    color::en::HexColor,
    http::en::RfcStatusCode,
    internet::en::{DomainSuffix, IPv4, IPv6, MACAddress, SafeEmail, UserAgent, Username, IP},
    lorem::en::*,
};
use fake::locales::EN;
use fake::locales::*;
use fake::Dummy;
use fake::{
    faker::{
        address::en::{BuildingNumber, Latitude, Longitude, PostCode, ZipCode},
        barcode::en::{Isbn, Isbn10, Isbn13},
        chrono::en::{Date, DateTime, DateTimeBefore, Duration, Time},
        name::raw::*,
        number::en::Digit,
        phone_number::en::{CellNumber, PhoneNumber},
    },
    uuid::{UUIDv1, UUIDv3, UUIDv4, UUIDv5},
};
use fake::{
    faker::{
        address::en::{CityName, CityPrefix, CitySuffix},
        boolean::en::Boolean,
        internet::en::{FreeEmail, FreeEmailProvider, Password},
    },
    Fake, Faker,
};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{
    pool,
    postgres::{self, PgConnectOptions, PgPool},
};

#[derive(Debug, Serialize)]
struct CreateDataResponse {
    res: String,
}

#[derive(Deserialize)]
struct CreateDataRequest {
    database: String,
    tables: Vec<tables>,
}

#[derive(Deserialize)]
struct tables {
    tablename: String,
    datasize: u128,
    fields: Vec<Field>,
}

#[derive(Deserialize)]
struct Field {
    fieldname: String,
    data_type: String,
    config: Config,
}

#[derive(Deserialize)]
struct Config {
    min_length: Option<i32>,
    max_length: Option<i32>,
    ratio: Option<u8>,
    date: Option<String>,
    date_end: Option<String>, // Everything else that config may accept
}

// Creating Table
async fn create_table(tablename: &String, fields: &Vec<Field>) {
    // Connecting to the Database
    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database("datasynth");

    let pool = PgPool::connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    let mut create_query = format!("CREATE TABLE {} (", tablename);
    let mut column_definitions = vec![];

    for field in fields {
        let mut column_definition = format!("");
        match field.data_type.as_ref() {
            "Name" => {
                let max_length = field.config.max_length.unwrap_or(255);
                column_definition = format!("{} {}({})", field.fieldname, "VARCHAR", max_length);
            } //INT
            "Int" => {
                column_definition = format!("{} {}", field.fieldname, "INT");
            } //DATETIME
            "DateTime" => {
                column_definition = format!("{} {}", field.fieldname, "TIMESTAMP");
            }
            _ => println!(
                "Didn't find -> {} <- in any of the expected values.",
                field.data_type
            ),
        }
        column_definitions.push(column_definition);
    }
    create_query.push_str(&column_definitions.join(", "));
    create_query.push_str(");");

    sqlx::query(&create_query)
        .execute(&pool)
        .await
        .expect("Failed to create table");
}

//Creating and Inserting fake data into the table
async fn create_and_insert_data(tablename: &String, datasize: &u128, fields: &Vec<Field>) -> impl Responder {
    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database("datasynth");

    let pool = PgPool::connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    // create the fake data using `fake = "2.5.0"` crate
    let mut values = vec![];
    for _i in 0..*datasize {
        let mut row_values = vec![];
        for field in fields {
            let fake_value = match field.data_type.as_str() {
                //VARCHAR
                "Name" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Name(EN).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                //INT
                "Int" => {
                    let max_length = field.config.max_length.unwrap_or(25);
                    let min_length = field.config.min_length.unwrap_or(5);
                    let mut rng = rand::thread_rng();
                    let num = rng.gen_range(min_length..=max_length);
                    num.to_string()
                }
                "DateTime" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = DateTime().fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                //support for other data types
                _ => panic!("Unsupported data type: -> {} <- \n", field.data_type),
            };
            row_values.push(fake_value);
        }
        let row_value_string = format!("'{}'", row_values.join("', '"));
        values.push(row_value_string);
    }

    let insert_query = format!(
        "INSERT INTO {} ({}) VALUES ({});",
        tablename,
        fields
            .iter()
            .map(|f| f.fieldname.clone())
            .collect::<Vec<String>>()
            .join(", "),
        values.join("), (")
    );

    sqlx::query(&insert_query)
        .execute(&pool)
        .await
        .expect("Failed to insert data");

    HttpResponse::Created().json(CreateDataResponse {
        res: "Data created and Added successfully".to_string(),
    })
}

async fn handle_create_req(req: web::Json<CreateDataRequest>) -> impl Responder {
    // Getting the request JSON
    let CreateDataRequest { database, tables } = req.into_inner();

    // Creating tables in database
    for i in 0..tables.len() {
        create_table(&tables[i].tablename, &tables[i].fields).await;
    }

    // creating fake data and inserting into the tables
    for i in 0..tables.len() {
        create_and_insert_data(&tables[i].tablename, &tables[i].datasize, &tables[i].fields).await;
    }



    HttpResponse::Created().json(CreateDataResponse {
        res: "Data created and Added successfully".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new().service(web::resource("/createdata").route(web::post().to(handle_create_req)))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
