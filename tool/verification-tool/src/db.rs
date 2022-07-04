
use dotenv::dotenv;
use std::env;
use postgres::{Client, NoTls};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};


#[derive(Debug,Clone)]
#[derive(Serialize,Deserialize)]
pub struct Logs {
    pub id: i32,
    pub registry_address: String, 
    pub author_address: String, 
    pub specification_id: String,
    pub specification: String,
    pub specification_file_name: String,
    pub proxy_address: String, 
    pub proxy: String,
    pub chain_id: String,
    pub created_at :DateTime<Utc>,
}

pub fn get_database_connection() -> Result<Client, String> {
   
    dotenv().ok();
    let data_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let result = Client::connect(&data_url, NoTls);

    if let Err(_) =  &result {
        return Err("Error for connecting to the database".to_owned());
    }
    let client = result.unwrap();
    Ok(client)
}

pub fn insert_on_table(log: &Logs ) -> Result<(), String> {

    let mut client = get_database_connection()?;
    let cur_time = Utc::now();

    let result = client.execute(
        "INSERT INTO public.logs (registry_address, author_address, specification, specification_id, proxy_address, 
            specification_file_name, proxy, chain_id, created_at)  VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        &[&log.registry_address, &log.author_address, &log.specification, &log.specification_id, 
        &log.proxy_address, &log.specification_file_name, &log.proxy, &log.chain_id, &cur_time] );

        println!("{:?}", result);

    if let Err(_) =  &result {
        return Err("Error for saving the contract data on the database".to_owned());
    }

    Ok(())
}


pub fn select_on_table(specification_id:&String, author_address:&String, chain_id:&String,) -> Result<Vec<Logs>, String> {

    let mut client =  get_database_connection()?;
    let mut list : Vec<Logs> =  Vec::new();

    for row in client.query("SELECT id, registry_address, author_address, specification_id, specification, proxy_address, 
    specification_file_name, proxy, chain_id, created_at FROM logs where specification_id ilike ($1) and author_address ilike ($2) and chain_id ilike ($3)", 
    &[&specification_id, &author_address, &chain_id]).unwrap() {
        
        let log = Logs {
            id: row.get(0),
            registry_address: row.get(1), 
            author_address: row.get(2), 
            specification_id: row.get(3),
            specification: row.get(4),
            proxy_address: row.get(5),
            specification_file_name: row.get(6),
            proxy: row.get(7),
            chain_id: row.get(8),
            created_at: row.get(9),      
        };
        list.push(log);
    }
    Ok(list)
}

pub fn select_specifications(author_address: &String) -> Result<Vec<Logs>, String> {

    let mut client =  get_database_connection()?;
    
    let mut list : Vec<Logs> =  Vec::new();

    for row in client.query("SELECT id, registry_address, author_address, specification_id, specification, proxy_address, 
    specification_file_name, proxy, chain_id, created_at FROM logs where author_address ilike ($1)", 
    &[&author_address]).unwrap() {
        
        let log = Logs {
            id: row.get(0),
            registry_address: row.get(1), 
            author_address: row.get(2), 
            specification_id: row.get(3),
            specification: row.get(4),
            proxy_address: row.get(5),
            specification_file_name: row.get(6),
            proxy: row.get(7),
            chain_id: row.get(8),
            created_at: row.get(9),    
        };
        list.push(log);
    }
    Ok(list)
}