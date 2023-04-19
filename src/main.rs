// // ALL THE REQUIRED IMPORTS // //
use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use chrono::{TimeZone, Utc};
use fake::faker::{
    address::en::{
        CountryCode, CountryName, Geohash, SecondaryAddress, SecondaryAddressType, StateAbbr,
        StateName, StreetName, StreetSuffix,
    },
    chrono::en::{DateTimeAfter, DateTimeBetween},
    color::en::{Color, HslColor, RgbColor, RgbaColor},
    company::en::{
        Bs, BsAdj, BsNoun, BsVerb, Buzzword, BuzzwordMiddle, BuzzwordTail, CatchPhase, CompanyName,
        CompanySuffix, Profession,
    },
    filesystem::en::{DirPath, FileExtension, FileName, FilePath},
    finance::raw::Bic,
    http::en::ValidStatusCode,
};
use fake::faker::{
    color::en::HexColor,
    http::en::RfcStatusCode,
    internet::en::{DomainSuffix, IPv4, IPv6, MACAddress, SafeEmail, UserAgent, Username, IP},
    lorem::en::*,
};
use fake::locales::EN;
use fake::{
    faker::{
        address::en::{BuildingNumber, Latitude, Longitude, PostCode, ZipCode},
        barcode::en::{Isbn, Isbn10, Isbn13},
        chrono::en::{Date, DateTime, DateTimeBefore, Time},
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
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPool};
use uuid::Uuid;

// // ALL THE REQUIRED STRUCT DEFINATIONS // //
#[derive(Debug, Serialize)]
struct CreateDataResponse {
    response: String,
}

#[derive(Deserialize)]
struct CreateDataRequest {
    database: String,
    tables: Vec<Table>,
}

#[derive(Deserialize)]
struct Table {
    tablename: String,
    datasize: u128,
    add_sql: String,
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
    date_end: Option<String>,
    // Add Everything else that config may accept
}

#[derive(Debug, Serialize)]
struct CreateDataResponse1 {
    res: String,
    request_id: String,
}

#[derive(Deserialize)]
struct CreateRelation {
    database: String,
    primary_table: String,
    secondary_table: String,
}

#[derive(Deserialize)]
struct DeleteDataRequest {
    database: String,
    request_id: String,
}

#[derive(Debug, sqlx::FromRow)]
struct Relation {
    id: String,
    primary_table: String,
    secondary_table: String,
}

// // ALL THE NECESSORY HANDLERS AND FUNCTIONS // //

//Handleing the Post request on create tables and data
async fn handle_create_tables_and_data_req(req: web::Json<CreateDataRequest>) -> impl Responder {
    // Getting the request JSON
    let CreateDataRequest { database, tables } = req.into_inner();

    // Creating tables in database
    for i in 0..tables.len() {
        create_table(
            &tables[i].tablename,
            &tables[i].add_sql,
            &tables[i].fields,
            &database,
        )
        .await;
    }

    // Create relations table
    create_relations_table(&database).await;
    
    // creating fake data and inserting into the tables
    for i in 0..tables.len() {
        create_and_insert_data(
            &tables[i].tablename,
            &tables[i].datasize,
            &tables[i].fields,
            &database,
        )
        .await;
    }

    let response: String = format!("Data created and added successfully");
    HttpResponse::Created().json(CreateDataResponse { response: response })
}

//CREATES RELATIONS TABLE
async fn create_relations_table(database: &String) {
    // Connecting to the Database
    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database(&database); // connect to the default postgres database

    let pool = PgPool::connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    let create_relations_table_query = "CREATE TABLE relations (unique_id varchar, primary_table varchar, secondary_table varchar);".to_string();
    sqlx::query(&create_relations_table_query)
        .execute(&pool)
        .await
        .expect("Failed to create relations table");
}

//CREATES RELATIONS BETWEEN EXISTING TABLES
async fn handle_add_relations_in_tables_req(req: web::Json<CreateRelation>) -> impl Responder {
    // Getting the request JSON
    let CreateRelation {
        database,
        primary_table,
        secondary_table,
    } = req.into_inner();

    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database(&database);
    let pool = PgPool::connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    let request_id = Uuid::new_v4();
    let requestid = request_id.to_string();
    let add_column_query = format!(
        "ALTER TABLE {} ADD {} INTEGER",
        secondary_table,
        primary_table.to_lowercase()
    );

    sqlx::query(&add_column_query)
        .execute(&pool)
        .await
        .expect("Failed to create new column");

    let primary_key_query = format!(
        "SELECT column_name
        FROM information_schema.key_column_usage
        WHERE table_name = '{}'
        AND constraint_name LIKE '%_pkey'",
        primary_table
    );

    let primary_key_name: String = sqlx::query_scalar(&primary_key_query)
        .fetch_one(&pool)
        .await
        .expect("Failed to get primary key name");

    let products_query = format!("SELECT {} FROM {}", primary_key_name, primary_table);

    let mut products: Vec<i32> = sqlx::query_scalar(&products_query)
        .fetch_all(&pool)
        .await
        .expect("Failed to get products");

    let column_query = format!(
        "SELECT column_name FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position LIMIT 1",
        secondary_table
    );
    let column_name: String = sqlx::query_scalar(&column_query)
        .fetch_one(&pool)
        .await
        .expect("Failed to get column name");

    let orders_query = format!("SELECT {} FROM {}", column_name, secondary_table);
    let orders: Vec<i32> = sqlx::query_scalar(&orders_query)
        .fetch_all(&pool)
        .await
        .expect("Failed to get orders");

    let mut rng = rand::thread_rng();
    let temp_vec = products.clone();
    for order_id in orders {
        let product_index = rng.gen_range(0..products.len());
        let product_id = products[product_index];

        let update_query = format!(
            "UPDATE {} SET {}_id = $1 WHERE {} = $2",
            secondary_table,
            primary_table.to_lowercase(),
            column_name
        );

        sqlx::query(&update_query)
            .bind(product_id)
            .bind(order_id)
            .execute(&pool)
            .await
            .expect("Failed to update order with foreign key");

        products.remove(product_index);
        if products.is_empty() {
            products = temp_vec.clone();
        }
    }

    let relation = Relation {
        id: requestid,
        primary_table,
        secondary_table,
    };

    //insert both table names and unique id in database
    let insert_query =
        "INSERT INTO relations (unique_id, primary_table, secondary_table) VALUES ($1, $2, $3)";
    sqlx::query(insert_query)
        .bind(relation.id)
        .bind(relation.primary_table)
        .bind(relation.secondary_table)
        .execute(&pool)
        .await
        .expect("Failed to insert relation into database");

    HttpResponse::Created().json(CreateDataResponse1 {
        request_id: request_id.to_string(),
        res: "Relational data added successfully".to_string(),
    })
}

//DELETES RELATIONS BETWEEN TABLES
async fn handle_delete_relations_in_tables_req(
    req: web::Json<DeleteDataRequest>,
) -> impl Responder {
    // Getting the request JSON
    let DeleteDataRequest {
        database,
        request_id,
    } = req.into_inner();

    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database(&database);

    let pool = PgPool::connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    // -- TODO - HARDCODED VALUES -- TO BE CHANGED LATER ON
    let get_relation_query =
        "SELECT primary_table, secondary_table FROM relations WHERE unique_id = $1";
    let relation = sqlx::query_as::<_, (String, String)>(get_relation_query)
        .bind(&request_id)
        .fetch_one(&pool)
        .await;

    match relation {
        Ok(relation) => {
            let delete_query = format!(
                "ALTER TABLE {} DROP COLUMN {}_id",
                relation.1,
                relation.0.to_lowercase()
            );

            sqlx::query(&delete_query)
                .execute(&pool)
                .await
                .expect("Failed to delete relation");

            // -- TODO - HARDCODED VALUES -- TO BE CHANGED LATER ON
            let delete_uuid_query = "DELETE FROM relations WHERE unique_id = $1";
            sqlx::query(delete_uuid_query)
                .bind(&request_id)
                .execute(&pool)
                .await
                .expect("Failed to delete UUID from relations table");

            HttpResponse::Ok().json(CreateDataResponse1 {
                request_id: request_id,
                res: "Relation deleted successfully".to_string(),
            })
        }
        Err(_) => HttpResponse::NotFound().json(CreateDataResponse1 {
            request_id: request_id,
            res: "Relation not found".to_string(),
        }),
    }
}

//Creating Table
async fn create_table(
    tablename: &String,
    add_sql: &String,
    fields: &Vec<Field>,
    database: &String,
) {
    // Connecting to the Database
    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database("postgres"); // connect to the default postgres database

    let pool = PgPool::connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    // Check if database exists
    let database_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT datname FROM pg_catalog.pg_database WHERE datname = $1)",
    )
    .bind(database)
    .fetch_one(&pool)
    .await
    .expect("Failed to check if database exists");

    if !database_exists {
        // Create database
        sqlx::query(&format!("CREATE DATABASE {}", database))
            .execute(&pool)
            .await
            .expect("Failed to create database");
    }

    // Connect to the newly created or existing database
    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database(&database);

    let pool = PgPool::connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
                SELECT *
                FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = $1
            )",
    )
    .bind(tablename)
    .fetch_one(&pool)
    .await
    .expect("Failed to check if table exists");

    if !table_exists {
        let mut create_query = format!("CREATE TABLE {} (", tablename);
        let mut column_definitions = vec![];

        let add_sql_str = format!(" {} ,", add_sql);
        create_query.push_str(&add_sql_str); ///// MODIFY

        for field in fields {
            let mut column_definition = format!("");
            match field.data_type.as_ref() {
                "String"
                | "StringInt"
                | "Name"
                | "City"
                | "Email"
                | "Password"
                | "Word"
                | "FirstName"
                | "LastName"
                | "Title"
                | "Suffix"
                | "NameWithTitle"
                | "FreeEmailProvider"
                | "DomainSuffix"
                | "FreeEmail"
                | "SafeEmail"
                | "Username"
                | "IPv4"
                | "IPv6"
                | "IP"
                | "MACAddress"
                | "UserAgent"
                | "RfcStatusCode"
                | "ValidStatusCode"
                | "HexColor"
                | "RgbColor"
                | "RgbaColor"
                | "HslColor"
                | "Color"
                | "CompanySuffix"
                | "CompanyName"
                | "Buzzword"
                | "BuzzwordMiddle"
                | "BuzzwordTail"
                | "CatchPhase"
                | "Verb"
                | "Adj"
                | "Noun"
                | "Bs"
                | "Profession"
                | "Industry"
                | "Geohash"
                | "CityPrefix"
                | "CitySuffix"
                | "CityName"
                | "CountryName"
                | "CountryCode"
                | "StreetSuffix"
                | "StreetName"
                | "FilePath"
                | "FileName"
                | "FileExtension"
                | "DirPath"
                | "StateName"
                | "StateAbbr"
                | "SecondaryAddressType"
                | "SecondaryAddress"
                | "PostCode"
                | "BuildingNumber"
                | "LicencePlate"
                | "Isbn"
                | "Isbn13"
                | "Isbn10"
                | "PhoneNumber"
                | "CellNumber"
                | "Bic"
                | "UUIDv1"
                | "UUIDv3"
                | "UUIDv4"
                | "UUIDv5"
                | "Product" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    column_definition =
                        format!("{} {}({})", field.fieldname, "VARCHAR", max_length);
                } //DECIMAL(#, #)
                "Latitude" => {
                    column_definition = format!("{} {}", field.fieldname, "DECIMAL(8,6)");
                }
                "Longitude" => {
                    column_definition = format!("{} {}", field.fieldname, "DECIMAL(9,6)");
                } //BOOLEAN
                "Bool" => {
                    column_definition = format!("{} {}", field.fieldname, "BOOLEAN");
                } //TEXT
                "Sentence" | "Sentences" | "Words" | "Paragraph" | "Paragraphs" => {
                    column_definition = format!("{} {}", field.fieldname, "TEXT");
                } //INT
                "Int" | "Digit" | "ZipCode" => {
                    column_definition = format!("{} {}", field.fieldname, "INT");
                } //FLOAT
                "Float" => {
                    column_definition = format!("{} {}", field.fieldname, "FLOAT");
                } //BIGINT
                "Bigint" => {
                    column_definition = format!("{} {}", field.fieldname, "BIGINT");
                }
                //TIME
                "Time" => {
                    column_definition = format!("{} {}", field.fieldname, "Time");
                } //DATE
                "Date" => {
                    column_definition = format!("{} {}", field.fieldname, "DATE");
                } //DATETIME //TIMESTAMP
                "DateTime" | "DateTimeBefore" | "DateTimeAfter" | "DateTimeBetween" => {
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

        print!("create_query -> {}", create_query);

        sqlx::query(&create_query)
            .execute(&pool)
            .await
            .expect("Failed to create table");
    } else {
        print!("table -> {} already exists", tablename);
    }
}

//Creating and Inserting fake data into the table
async fn create_and_insert_data(
    tablename: &String,
    datasize: &u128,
    fields: &Vec<Field>,
    database: &String,
) -> impl Responder {
    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database(&database);

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
                "String" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Faker.fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                "StringInt" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let min_length = field.config.min_length.unwrap_or(255);
                    let mut rng = rand::thread_rng();
                    let num = rng.gen_range(min_length..=max_length);
                    num.to_string()
                }
                "Name" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Name(EN).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "City" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CityName().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Email" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = FreeEmail().fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Password" => {
                    let mut max_length = field.config.max_length.unwrap_or(25);
                    let mut min_length = field.config.min_length.unwrap_or(5);
                    if min_length >= max_length {
                        std::mem::swap(&mut min_length, &mut max_length);
                    }
                    let fake_string: String =
                        Password(min_length as usize..max_length as usize).fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Word" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Word().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "FirstName" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = FirstName(EN).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "LastName" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = LastName(EN).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Title" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Title(EN).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Suffix" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Suffix(EN).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "NameWithTitle" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = NameWithTitle(EN).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "FreeEmailProvider" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = FreeEmailProvider().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "DomainSuffix" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = DomainSuffix().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "FreeEmail" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = FreeEmail().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "SafeEmail" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = SafeEmail().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Username" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Username().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "IPv4" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = IPv4().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "IPv6" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = IPv6().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "IP" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = IP().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "MACAddress" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = MACAddress().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "UserAgent" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = UserAgent().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "RfcStatusCode" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = RfcStatusCode().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "ValidStatusCode" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = ValidStatusCode().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "HexColor" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = HexColor().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "RgbColor" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = RgbColor().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "RgbaColor" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = RgbaColor().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "HslColor" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = HslColor().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Color" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Color().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CompanySuffix" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CompanySuffix().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CompanyName" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CompanyName().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Buzzword" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Buzzword().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "BuzzwordMiddle" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = BuzzwordMiddle().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "BuzzwordTail" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = BuzzwordTail().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CatchPhase" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CatchPhase().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Verb" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = BsVerb().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Adj" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = BsAdj().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Noun" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = BsNoun().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Bs" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Bs().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Profession" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Profession().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Industry" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CatchPhase().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Geohash" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Geohash(8).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CityPrefix" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CityPrefix().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CitySuffix" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CitySuffix().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CityName" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CityName().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CountryName" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CountryName().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CountryCode" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CountryCode().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "StreetSuffix" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = StreetSuffix().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "StreetName" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = StreetName().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "FilePath" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = FilePath().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "FileName" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = FileName().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "FileExtension" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = FileExtension().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "DirPath" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = DirPath().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "StateName" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = StateName().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "StateAbbr" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = StateAbbr().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "SecondaryAddressType" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = SecondaryAddressType().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "SecondaryAddress" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = SecondaryAddress().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "PostCode" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = PostCode().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "BuildingNumber" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = BuildingNumber().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "LicencePlate" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    // let fake_string: String = LicencePlate().fake();
                    // let re = Regex::new(r"[A-Z]{2}[0-9]{2}[A-Z]{2}").unwrap();
                    // let fake_string: String = rand::thread_rng().sample_iter(&re).take(1).next().unwrap().to_string();
                    let fake_string: String = format!("MH26RB5501"); //Hardcoded String as a placeholder
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Isbn" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Isbn().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Isbn13" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Isbn13().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Isbn10" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Isbn10().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "PhoneNumber" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = PhoneNumber().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "CellNumber" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = CellNumber().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Bic" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Bic(EN).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "UUIDv1" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = UUIDv1.fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "UUIDv3" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = UUIDv3.fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "UUIDv4" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = UUIDv4.fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "UUIDv5" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = UUIDv5.fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                //VARCHAR END
                // Decimal(8,6) - Latitude
                "Latitude" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Latitude().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                // Decimal(9,6) - Latitude
                "Longitude" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Longitude().fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                //BOOLEAN
                "Bool" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let ratio: u8 = field.config.ratio.unwrap_or(50) as u8;
                    let fake_bool: bool = Boolean(ratio as u8).fake();
                    let fake_string: String = fake_bool.to_string();
                    fake_string.chars().take(max_length as usize).collect()
                }
                //Bool END
                //TEXT
                "Sentence" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let min_length = field.config.min_length.unwrap_or_default();
                    let fake_string: String =
                        Sentence(min_length as usize..max_length as usize).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Sentences" => {
                    let max_length = field.config.max_length.unwrap_or(25);
                    let min_length = field.config.min_length.unwrap_or(5);
                    let fake_string: String =
                        Sentence(min_length as usize..max_length as usize).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string
                        .chars()
                        .take((max_length * 10) as usize)
                        .collect()
                }
                "Words" => {
                    let max_length = field.config.max_length.unwrap_or(25);
                    let mut fake_string = String::new();
                    for i in 0..max_length {
                        fake_string.push_str(Word().fake());

                        if i != max_length - 1 {
                            fake_string.push_str(", ");
                        }
                    }
                    let fake_string = fake_string.replace("'", "''");
                    fake_string
                        .chars()
                        .take((max_length * 10) as usize)
                        .collect()
                }
                "Paragraph" => {
                    let max_length = field.config.max_length.unwrap_or(25);
                    let min_length = field.config.min_length.unwrap_or(5);
                    let fake_string: String =
                        Paragraph(min_length as usize..max_length as usize).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                "Paragraphs" => {
                    let max_length = field.config.max_length.unwrap_or(55);
                    let min_length = field.config.min_length.unwrap_or(5);
                    let fake_string: String =
                        Paragraph(min_length as usize..max_length as usize).fake();
                    let fake_string = fake_string.replace("'", "''");
                    fake_string
                        .chars()
                        .take((max_length * 10) as usize)
                        .collect()
                }
                //INT
                "Int" => {
                    let max_length = field.config.max_length.unwrap_or(25);
                    let min_length = field.config.min_length.unwrap_or(5);
                    let mut rng = rand::thread_rng();
                    let num = rng.gen_range(min_length..=max_length);
                    num.to_string()
                }
                "Digit" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Digit().fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                "ZipCode" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = ZipCode().fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                //FLOAT
                "Float" => {
                    let max_length = field.config.max_length.unwrap_or(250);
                    let min_length = field.config.min_length.unwrap_or(5);
                    let mut rng = rand::thread_rng();
                    let num = rng.gen_range(min_length..=max_length);
                    num.to_string()
                }
                //BIGINT
                "Bigint" => {
                    let max_length = field.config.max_length.unwrap_or(250000);
                    let min_length = field.config.min_length.unwrap_or(5);
                    let mut rng = rand::thread_rng();
                    let num = rng.gen_range(min_length..=max_length);
                    num.to_string()
                }
                //TIME
                "Time" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Time().fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                //Date
                "Date" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = Date().fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                "DateTime" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let fake_string: String = DateTime().fake();
                    fake_string.chars().take(max_length as usize).collect()
                }
                "DateTimeBefore" => {
                    let default_str: String = format!("2023-04-14 05:05:01");
                    let get_dt: &str = field.config.date.as_ref().unwrap_or(&default_str);
                    let dt_before = Utc
                        .datetime_from_str(get_dt, "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                        .with_timezone(&Utc);
                    let fake_string: String = DateTimeBefore(dt_before).fake();
                    fake_string.chars().take(255 as usize).collect()
                }
                "DateTimeAfter" => {
                    let default_str: String = format!("2023-04-14 05:05:01");
                    let get_dt: &str = field.config.date.as_ref().unwrap_or(&default_str);
                    let dt_after = Utc
                        .datetime_from_str(get_dt, "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                        .with_timezone(&Utc);
                    let fake_string: String = DateTimeAfter(dt_after).fake();
                    fake_string.chars().take(255 as usize).collect()
                }
                "DateTimeBetween" => {
                    let default_start: String = format!("2001-05-05 05:05:01");
                    let default_end: String = format!("2023-04-14 05:05:01");
                    let get_start_dt: &str = field.config.date.as_ref().unwrap_or(&default_start);
                    let get_end_dt: &str = field.config.date_end.as_ref().unwrap_or(&default_end);
                    let dt_start = Utc
                        .datetime_from_str(get_start_dt, "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                        .with_timezone(&Utc);
                    let dt_end = Utc
                        .datetime_from_str(get_end_dt, "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                        .with_timezone(&Utc);
                    let fake_string: String = DateTimeBetween(dt_start, dt_end).fake();
                    fake_string.chars().take(255 as usize).collect()
                }
                "Product" => {
                    let max_length = field.config.max_length.unwrap_or(255);
                    let adj: String = BsAdj().fake();
                    let noun: String = BsNoun().fake();
                    let fake_string: String = format!("{} {}", adj, noun);
                    let fake_string = fake_string.replace("'", "''");
                    fake_string.chars().take(max_length as usize).collect()
                }
                // add support for other data types if needed
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
        response: "Data created and Added successfully".to_string(),
    })
}

//ACTIX WEB HANDLE THE REST FEATURES
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(
                web::resource("/create_tables_and_data")
                    .route(web::post().to(handle_create_tables_and_data_req)),
            )
            .service(
                web::resource("/add_relations_in_tables")
                    .route(web::post().to(handle_add_relations_in_tables_req)),
            )
            .service(
                web::resource("/delete_relations_in_tables")
                    .route(web::delete().to(handle_delete_relations_in_tables_req)),
            )
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
