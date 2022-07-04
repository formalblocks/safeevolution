#[macro_use] extern crate rocket;

use crate::{db::{select_specifications, select_on_table, insert_on_table, Logs}};
use crate::{util::{write_file,get_number_argument_constructor,AssignedVariable, get_compiled_files, delete_dir_contents, copy_dir_contents}};
// use crate::{deployer::{get_connection,get_compiled_files, get_assigned_variable, pad_to_bytes32}};
use crate::{old_main_j::{verify_deploy_contract, upgrade_contract, get_specification, get_implementation}};
mod db;
mod util;
mod parser;
mod deployer;
mod checker;
mod proxy;
mod old_main_j;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::http::Header;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{http::Method, http::Status, Request, Response};
use rocket::response::status::BadRequest;
use std::{path::Path};
use web3::{types::{Address}};
use std::str::FromStr;
use std::fs;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Guard, Responder};

#[get("/listupgrades/<wallet_address>")]
fn list_upgrades(wallet_address: String) -> Result<Json<Vec<Logs>>, BadRequest<String>> {
    let logs = select_specifications(&wallet_address).unwrap();
    Ok(Json(logs))
}

#[post("/getconstructorarguments", format = "json", data = "<payload>")]
fn get_constructor_arguments(payload: Json<ContructorArguments>) -> Result<Json<Vec<AssignedVariable>>, BadRequest<String>> {
    
    // for file in &payload.implementation_files {
    //     write_file(&file.content, &file.name);
    // }
    // let impl_url = format!("contracts/input/{}", &payload.file_to_be_verified);
    // let imp = get_implementation(Path::new(&impl_url));
    
    // if let Err(error) = &imp {
    //    return Err(BadRequest(Some(error.to_string())));
    // }

    // let constructor_arguments = get_number_argument_constructor(&imp.unwrap()).unwrap();

    let mut parameters_and_values: Vec<AssignedVariable> = Vec::new();

    // for i in 0..constructor_arguments.len() {
    //     let variable_value = AssignedVariable {
    //         variable_declaration: constructor_arguments[i].clone(),
    //         variable_value: "".to_owned(),
    //     };
    //     parameters_and_values.push(variable_value);
    // }
    Ok(Json(parameters_and_values))
}


#[post("/getcontract", format = "json", data = "<payload>")]
async fn get_contract(payload: Json<DeployContract>) -> Result<Json<Vec<ContractCompiled>>, BadRequest<String>> {

    write_file(&payload.specification_file.content, &payload.specification_file.name);

    for file in &payload.implementation_files {
        write_file(&file.content, &file.name);
    }

    let spec_url = format!("contracts/input/{}", &payload.specification_file.name);
    let spec = get_specification(Path::new(&spec_url)).unwrap();

    let impl_url = format!("contracts/input/{}", &payload.file_to_be_verified);
    let imp = get_implementation(Path::new(&impl_url)).unwrap();

    let result = verify_deploy_contract(Path::new(&impl_url), Path::new(&spec_url), &imp, &spec, 
                                &payload.specification_id, &mut payload.constructor_arguments.clone()).await;

    if let Err(error) = &result {
        return Err(BadRequest(Some(error.to_string())));
    }
    
    let registry_abi = get_compiled_files(Path::new("contracts/input/Registry.abi"));
    let registry_bin = get_compiled_files(Path::new("contracts/input/Registry.bin"));
    let registry =  ContractCompiled { category: "registry".to_string(), abi: registry_abi.unwrap(), 
                        bin: registry_bin.unwrap(), file: "".to_string(), address: "".to_string() };


    let path_contract_abi = Path::new("contracts/input").join(&format!("{}.abi", &imp.contract_name));
    let path_contract_bin = Path::new("contracts/input").join(&format!("{}.bin", &imp.contract_name));

    let contents_contract_abi = get_compiled_files(&path_contract_abi);
    let contents_contract_bin = get_compiled_files(&path_contract_bin);

    let contract =  ContractCompiled { category: "contract".to_string(), abi: contents_contract_abi.unwrap(), 
    bin: contents_contract_bin.unwrap(), file: "".to_string(), address: "".to_string() };

    let path_proxy_abi = Path::new("contracts/input/Proxy.abi");
    let path_proxy_bin = Path::new("contracts/input/Proxy.bin");

    let contents_proxy_abi = get_compiled_files(&path_proxy_abi);
    let contents_proxy_bin = get_compiled_files(&path_proxy_bin);
    let proxy_file = get_compiled_files(Path::new("contracts/input/implementedproxy.sol")).unwrap();

    let proxy = ContractCompiled { category: "proxy".to_string(), abi: contents_proxy_abi.unwrap(), 
    bin: contents_proxy_bin.unwrap(), file: proxy_file, address: "".to_string()};

    let list: Vec<ContractCompiled> = vec![contract, proxy, registry];

    Ok(Json(list))
}


#[post("/upgradecontract/<author_wallet>/<chain_id>", format = "json", data = "<payload>")]
async fn upgrade_contract_file(author_wallet:String, chain_id:String, payload:Json<UpgradeContract>) -> Result<Json<Vec<ContractCompiled>>, BadRequest<String>> {
    
    for file in &payload.implementation_files {
        write_file(&file.content, &file.name);
    }

    let impl_url = format!("contracts/input/{}", &payload.file_to_be_verified);
    let imp = get_implementation(Path::new(&impl_url)).unwrap();


    let impl_url = format!("contracts/input/{}", &payload.file_to_be_verified);
    let author_wallet_address = Address::from_str(author_wallet.as_str()).unwrap();
    
    let result = upgrade_contract(Path::new(&impl_url), &imp, &payload.specification_id, &author_wallet_address, &chain_id).await;

    if let Err(error) = &result {
        return Err(BadRequest(Some(error.to_string())));
    }

    let log = select_on_table(&payload.specification_id, &format!("{:?}", &author_wallet_address), &chain_id).unwrap();
   
    let path_contract_abi = Path::new("contracts/input").join(format!("{}.abi", &imp.contract_name));
    let path_contract_bin = Path::new("contracts/input").join(format!("{}.bin", &imp.contract_name));
    let contents_contract_abi = get_compiled_files(&path_contract_abi);
    let contents_contract_bin = get_compiled_files(&path_contract_bin);
    let contract =  ContractCompiled { category: "contract".to_string(), abi: contents_contract_abi.unwrap(), 
                        bin: contents_contract_bin.unwrap(), file: "".to_string(), address: "".to_string()  };

    let registry_abi = get_compiled_files(Path::new("contracts/input/Registry.abi"));
    let registry_bin = get_compiled_files(Path::new("contracts/input/Registry.bin"));
    let registry =  ContractCompiled { category: "registry".to_string(), abi: registry_abi.unwrap(), 
                        bin: registry_bin.unwrap(), file: "".to_string(), address: log[0].registry_address.to_string() };

    let proxy_abi = get_compiled_files(Path::new("contracts/input/Proxy.abi"));
    let proxy_bin = get_compiled_files(Path::new("contracts/input/Proxy.bin"));
    let proxy =  ContractCompiled { category: "proxy".to_string(), abi: proxy_abi.unwrap(), 
                    bin: proxy_bin.unwrap(), file: "".to_string(), address: log[0].proxy_address.to_string() };

    let list: Vec<ContractCompiled> = vec![contract, registry, proxy ];  

    Ok(Json(list))
}

#[post("/savelog", format = "json", data = "<payload>")]
fn save_log(payload: Json<Logs>) -> Result<(), BadRequest<String>> {
    
    let result = insert_on_table(&payload);

    if let Err(error) = &result {
        return Err(BadRequest(Some(error.to_string())));
    }
    
    Ok(())
}


// #[post("/deploycontract/<wallet_address>", format = "json", data = "<payload>")]
// async fn deploy_contract_file(wallet_address: String, payload: Json<DeployContract>) -> Result<(), BadRequest<String>> {
    
//     write_file(&payload.specification_file.content,  &payload.specification_file.name);

//     for file in &payload.implementation_files {
//         write_file(&file.content, &file.name);
//     }

//     let spec_url = format!("contracts/input/{}", &payload.specification_file.name);
//     let spec = get_specification(Path::new(&spec_url)).unwrap();

//     let impl_url = format!("contracts/input/{}", &payload.file_to_be_verified);
//     let imp = get_implementation(Path::new(&impl_url)).unwrap();

//     let wallet_address = Address::from_str(wallet_address.as_str()).unwrap();

//     let result = verify_deploy_contract(Path::new(&impl_url), Path::new(&spec_url), &imp, &spec, &payload.specification_id,  &mut payload.constructor_arguments.clone()).await;

//     if let Err(error) = &result {
//         return Err(BadRequest(Some(error.to_string())));
//     }
//     Ok(())
// }

// #[post("/upgradecontract/<wallet_address>", format = "json", data = "<payload>")]
// async fn upgrade_contract_file(wallet_address: String, payload: Json<UpgradeContract>) -> Result<(), BadRequest<String>> {
    
//     for file in &payload.implementation_files {
//         write_file(&file.content, &file.name);
//     }

//     let impl_url = format!("contracts/input/{}", &payload.file_to_be_verified);
//     let wallet_address = Address::from_str(wallet_address.as_str()).unwrap();
//     let result = upgrade_contract(Path::new(&impl_url), &payload.specification_id, &wallet_address).await;

//     if let Err(error) = &result {
//         return Err(BadRequest(Some(error.to_string())));
//     }
//     Ok(())
// }




#[launch]
fn rocket() -> _ {
    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![Method::Get,Method::Post,Method::Patch].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    rocket::build()
    .mount("/", routes![list_upgrades])
    .mount("/", routes![get_constructor_arguments])
    .mount("/", routes![upgrade_contract_file])
    .mount("/", routes![get_contract])
    .mount("/", routes![save_log])
    .mount("/", rocket_cors::catch_all_options_routes())
    .manage(cors)
}


// #[derive(Serialize,Deserialize, Debug)]
// pub struct ContractSpecCompiled {
//     pub specification_file_32_bytes: [u8; 32],
//     pub contracts_compiled: Vec<ContractCompiledFiles>, 
// }

#[derive(Serialize,Deserialize, Debug)]
pub struct ContractCompiled {
    pub category: String,
    pub abi: String, 
    pub bin: String, 
    pub file: String,
    pub address: String, 
}

#[derive(Serialize,Deserialize, Debug)]
pub struct DeployContract {
    pub specification_file: File,
    pub file_to_be_verified: String, 
    pub implementation_files: Vec<File>, 
    pub specification_id: String,
    pub constructor_arguments: Vec<AssignedVariable>,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct UpgradeContract {
    pub implementation_files: Vec<File>, 
    pub specification_id: String,
    pub file_to_be_verified: String 
}

#[derive(Serialize,Deserialize, Debug)]
pub struct ContructorArguments {
    pub implementation_files: Vec<File>, 
    pub file_to_be_verified: String
}

#[derive(Serialize,Deserialize, Debug)]
pub struct File {
    pub name: String, 
    pub content: String,
    pub verify:bool,
}


pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self,request: &'r Request<'_>,response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}


use web3::{
    types::{H160,U256},
    contract::{Contract, Options},
};

#[cfg(test)]
mod tests {
    use super::*;

// #[actix_rt::test]
    // async fn deploy_reentrancy_contract() {
    //     let web3 = get_connection().unwrap(); 
    //     let spec_url = "tests/ReentrancySpec.sol";
    //     let spec = get_specification(Path::new(&spec_url)).unwrap();
    //     let impl_url = "tests/Reentrancy.sol";
    //     let imp = get_implementation(Path::new(&impl_url)).unwrap();
    //     let spec_id = "reentrancy".to_string();
    //     let mut accounts = web3.eth().accounts().await.unwrap();
    //     deploy_contract(Path::new(&impl_url), Path::new(&spec_url), &imp, &spec, &"reentrancy".to_string(), &accounts[0], &mut vec![]).await;
   
    //     accounts = web3.eth().accounts().await.unwrap();
    //     let log = select_on_table(&spec_id.clone(), &format!("{:?}", accounts[0])).unwrap();    
        
    //     let proxy_abi = get_compiled_files(Path::new("contracts/input/Proxy.abi")).unwrap();
        
    //     let proxy_contract = Contract::from_json(web3.eth(), Address::from_str(&log[0].proxy_address).unwrap(), 
    //     proxy_abi.as_bytes()).unwrap();

    //     proxy_contract.call("deposit", (),  accounts[0], Options::with(|opt| {
    //         opt.value = Some(10000.into()); opt.gas_price = Some(5.into()); opt.gas = Some(3_000_000.into()); })).await;
        
    //     let balance:U256 = proxy_contract.query("getBalance", (), None, Options::default(), None).await.unwrap();
    //     assert_eq!(balance, U256::from(10000));

    //     delete_dir_contents();

    //     let impl_evol = "tests/ReentrancyEvol.sol";

    //     upgrade_contract( Path::new(&impl_evol), &spec_id, &accounts[0]).await;
       
    //     accounts = web3.eth().accounts().await.unwrap();  

    //     proxy_contract.call("deposit", (),  accounts[0], Options::with(|opt| {
    //         opt.value = Some(10000.into()); opt.gas_price = Some(5.into()); opt.gas = Some(3_000_000.into());})).await;

    //     let balance:U256 = proxy_contract.query("getBalance", (), None, Options::default(), None).await.unwrap();
    //     assert_eq!(balance, U256::from(10000));

    //     proxy_contract.call("deposit", (),  accounts[0], Options::with(|opt| {
    //         opt.value = Some(20000.into()); opt.gas_price = Some(5.into()); opt.gas = Some(3_000_000.into()); })).await;
        
    //     let balance:U256 = proxy_contract.query("getBalance", (), None, Options::default(), None).await.unwrap();
    //     assert_eq!(balance, U256::from(30000));
    //     delete_dir_contents();
    // }
    
    // #[actix_rt::test]
    // async fn deploy_simple_contract() {
    //     let web3 = get_connection().unwrap(); 
    //     let spec_url = "tests/SimpleSpec.sol";
    //     let spec = get_specification(Path::new(&spec_url)).unwrap();
    //     let impl_url = "tests/Simple.sol";
    //     let imp = get_implementation(Path::new(&impl_url)).unwrap();
    //     let spec_id = "simple".to_string();
    //     let mut accounts = web3.eth().accounts().await.unwrap();

    //     let values = vec!["5".to_string(),"6".to_string(),"false".to_string()];
    //     let constructor_arguments = get_number_argument_constructor(&imp).unwrap();
    //     let mut parameters_and_values: Vec<AssignedVariable> = Vec::new();

    //     for i in 0..constructor_arguments.len() {
    //         let variable_value = AssignedVariable {
    //             variable_declaration: constructor_arguments[i].clone(),
    //             variable_value: values[i].clone(),
    //         };
    //         parameters_and_values.push(variable_value);
    //     }
      
    //     deploy_contract(Path::new(&impl_url), Path::new(&spec_url), &imp, &spec, &"simple".to_string(), &accounts[0], &mut parameters_and_values).await;
       
    //     accounts = web3.eth().accounts().await.unwrap();  
    //     let log = select_on_table(&spec_id.clone(), &format!("{:?}", accounts[0])).unwrap();    
        
    //     let proxy_abi = get_compiled_files(Path::new("contracts/input/Proxy.abi")).unwrap();
        
    //     let proxy_contract = Contract::from_json(web3.eth(), Address::from_str(&log[0].proxy_address).unwrap(), 
    //     proxy_abi.as_bytes()).unwrap();
        
    //     let result = proxy_contract.query("get_selected", (), None, Options::default(), None);
    //     let selected: U256 = result.await.unwrap();
    //     assert_eq!(selected, web3::types::U256::from(6));

    //     delete_dir_contents();
        
    //     let impl_evol = "tests/SimpleEvol.sol";
        
    //     upgrade_contract(Path::new(&impl_evol), &spec_id, &accounts[0]).await;

    //     let result_updated = proxy_contract.query("get_selected", (), None, Options::default(), None);
    //     let selected_updated: U256 = result_updated.await.unwrap();
    //     assert_eq!(selected_updated, web3::types::U256::from(5));
       
    //     delete_dir_contents();
    // }

    // #[actix_rt::test]   
    // async fn deploy_food_token() {
    //     copy_dir_contents("tests/0xMonorepo/deploy", "contracts/input");
        
    //     let web3 = get_connection().unwrap(); 
    //     let spec_url = "tests/0xMonorepo/ERC20Specification.sol";
    //     let spec = get_specification(Path::new(&spec_url)).unwrap();
    //     let impl_url = "tests/0xMonorepo/deploy/ERC20TokenHeritage2.sol";
    //     let imp = get_implementation(Path::new(&impl_url)).unwrap();
    //     let spec_id = "erc20".to_string();
    //     let mut accounts = web3.eth().accounts().await.unwrap();
        
    //     deploy_contract(Path::new(&impl_url), Path::new(&spec_url), &imp, &spec, &spec_id, &accounts[0],  &mut vec![]).await;
    
    //     accounts = web3.eth().accounts().await.unwrap();
    //     let log = select_on_table(&spec_id.clone(), &format!("{:?}", accounts[0])).unwrap();
        
    //     let proxy_abi = get_compiled_files(Path::new("contracts/input/Proxy.abi")).unwrap();
        
    //     let proxy_contract = Contract::from_json(web3.eth(), Address::from_str(&log[0].proxy_address).unwrap(), 
    //     proxy_abi.as_bytes()).unwrap();

    //     proxy_contract.call("approve", (accounts[1], web3::types::U256::from(5)),  accounts[0], Options::with(|opt| {
    //          opt.gas_price = Some(5.into()); opt.gas = Some(3_000_000.into());})).await;

    //     let result = proxy_contract.query("allowance", (accounts[0], accounts[1]), None, Options::default(), None);
    //     let selected: U256 = result.await.unwrap();

    //     assert_eq!(selected, web3::types::U256::from(5));
                
    //     delete_dir_contents();
        
    //     copy_dir_contents("tests/0xMonorepo/evolution", "contracts/input");
    //     let impl_evol = "tests/0xMonorepo/evolution/ERC20TokenHeritage2.sol";
        
    //     upgrade_contract(Path::new(&impl_evol), &spec_id, &accounts[0]).await;

    //     let result_updated = proxy_contract.query("approve", (accounts[1], web3::types::U256::from(5)), None, Options::default(), None);
    //     let selected_updated: bool = result_updated.await.unwrap();
    //     assert_eq!(selected_updated, false);
       
    //     delete_dir_contents();
    
    // }

}
