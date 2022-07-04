
use crate::parser::{Implementation,VariableDeclaration};
use std::str::FromStr;
use web3::{
    ethabi::ethereum_types::{U256, H32, H160, H256},
    types::{Address, TransactionRequest},
    transports::Http,
    contract::{Contract, Options},
    Web3
};

use hex;
use async_std::task;
use std::{collections::HashMap, hash::Hash, time};
use std::env;
use std::fs::File;
use std::convert::TryInto;
use std::str;
use std::{path::Path, process::{Command, Stdio}};
use std::io::{self, prelude::*, SeekFrom, Read};
use std::fs;
use ethabi::{Token, Param, Function, ParamType};
use web3::types::Bytes;
// use crate::deployer::{get_connection};
use serde::{Serialize, Deserialize};
use std::future::Future;

#[derive(Debug,Clone, Serialize,Deserialize)]
pub struct AssignedVariable {
    pub variable_declaration: VariableDeclaration, 
    pub variable_value: String, 
 }


pub fn get_value(assigned_variable: &AssignedVariable) -> Result<Token, String> {
    
    match assigned_variable.variable_declaration.typ.as_str() {
        "address" =>  { 
            let result = web3::types::H160::from_str(&assigned_variable.variable_value);
            if let Err(_) = &result {
                return Err( format!("Cannot convert value {} for type {}", assigned_variable.variable_value, assigned_variable.variable_declaration.typ));
            }
            return Ok(Token::Address(ethereum_types::H160::from(string_to_static_str(assigned_variable.variable_value.clone()))));
        },
        "bool" =>  {
            let result = assigned_variable.variable_value.parse::<bool>();
            if let Err(_) = &result {
                return Err( format!("Cannot convert value {} for type {}", assigned_variable.variable_value, assigned_variable.variable_declaration.typ));
            }
            return Ok(Token::Bool(result.unwrap()))
        },
        "string" => return Ok(Token::String(assigned_variable.variable_value.to_owned())),
        "bytes8" | "bytes32" | "bytes64" | "bytes1024" => return Ok(Token::FixedBytes(assigned_variable.variable_value.as_bytes().to_vec())),
        "uint256" => {
            let result = assigned_variable.variable_value.parse::<u64>();
            if let Err(_) = &result {
                return Err( format!("Cannot convert value {} for type {}", assigned_variable.variable_value, assigned_variable.variable_declaration.typ));
            }
            return Ok(Token::Uint(ethereum_types::U256::from(result.unwrap())));
        },
        _ => Err("There is no match for this value".to_owned()),
    }
}

pub fn write_file(text: &String, file_name: &String) {

    let mut file = match File::create(Path::new("contracts/input").join(&file_name).as_path()) {
        Err(why) => panic!("Couldn't create the file {}", why),
        Ok(file) => file,
    };

    match file.write_all(text.as_bytes()) {
        Err(why) => panic!("Couldn't create the file {} ", why),
        Ok(_) => println!("Successfully created the file"),
    }
}

pub fn get_number_argument_constructor(imp: &Implementation) ->  Result <Vec<VariableDeclaration>, ()>{
    let mut contructor_variables: Vec<VariableDeclaration> = Vec::new();
    for i in 0..imp.functions.len() {
        if &imp.functions[i].signature.kind == "constructor" {
            contructor_variables = imp.functions[i].signature.ins.clone();
        }
    }
    Ok(contructor_variables)
}


pub fn delete_dir_contents() {
    let contents = fs::read_dir("contracts/input");
    
    if let Ok(dir) = contents {
        for entry in dir {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(path).expect("Failed to remove a dir");
                } else {
                    fs::remove_file(path).expect("Failed to remove a file");
                }
            };
        }
    };
}

pub fn copy_dir_contents(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_contents(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

// pub async fn deploy_contract_blockchain<'a>(from:&'a H160, parameters_and_values: &Vec<AssignedVariable>, path_bin:&'a Path, path_abi:&'a Path) ->  Result<H160, String> {

//     let proxy_bin = get_compiled_files(&path_bin).unwrap();

//     let code =  hex::decode(&proxy_bin).map_err( |_| {"Error parsing bin".to_owned()} ).unwrap();

//     let json = get_compiled_files(&path_abi).unwrap();
//     let abi = ethabi::Contract::load(json.as_bytes()).unwrap();

//     let params = config_contructor_parameters(&parameters_and_values)?;

//     let data = match (abi.constructor(), params.is_empty()) {
//         (None, false) => return Err("The constructor is not defined in the ABI.".to_owned()),
//         (None, true) => code,
//         (Some(constructor), _) => constructor.encode_input(code, &params).unwrap(),
//     };

//     let contract_address = send_contract_blockchain(&from, data).await;

//     Ok(contract_address.unwrap())
// }

// pub async fn send_contract_blockchain(from:&H160, encoded_data: Vec<u8>) -> Result<H160, String>{
    
//     let web3:Web3<Http> = get_connection().unwrap();

//     let tx_request = TransactionRequest {
//         from: *from,
//         to: None,
//         gas: Some(3_000_000.into()),
//         gas_price: None,
//         value: None,
//         data: Some(Bytes(encoded_data)),
//         nonce: None,
//         condition: None,
//         transaction_type: None,
//         access_list: None,
//     };

//     let receipt = web3.send_transaction_with_confirmation(tx_request, time::Duration::from_secs(7), 0).await.unwrap();

//     match receipt.status {
//         Some(status) if status == 0.into() => return Err("The contract could not be deployed".to_owned()),
//         _ => match receipt.contract_address {
//             Some(address) => Ok(address),
//             None => return Err("The contract could not be deployed".to_owned()),
//         },
//     }
// }

pub fn config_contructor_parameters(assigned_variables: &Vec<AssignedVariable>)  -> Result <Vec<Token>, String> {

    let mut list_token: Vec<Token> = Vec::new();
    for value in assigned_variables {
        list_token.push(get_value(&value)?);
    }
    Ok(list_token)
}

pub fn get_compiled_files(path_contract:&Path)  -> Result <String, String> {
    let mut file_contract = File::open(&path_contract).unwrap();
    let mut data_contract = String::new();
    file_contract.read_to_string(&mut data_contract).unwrap();
    Ok(data_contract)
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}


pub fn generate_ast_contract(file_name: &str) -> Result <(), String> {
    let path = env::current_dir().unwrap();
    let command = format!("docker run --rm -v {}/contracts/input:/sources ethereum/solc:0.5.17 -o sources --ast-compact-json  /sources/{} --overwrite",
    path.to_str().unwrap(), &file_name);

    let com = Command::new("cmd").args(&["/C", &command]).stdin(Stdio::piped())
    .stdout(Stdio::piped()).spawn().expect("echo command failed to start");

    let mut answer = String::new();
    match com.stdout.unwrap().read_to_string(&mut answer) {
        Err(why) => panic!("Couldn't generate ast tree: {}", why),
        Ok(_) => print!("Tree generated with sucess:\n{}", answer),
    }
   Ok(())
}

pub fn verify_contract(merged_contract_file: String) -> Result <String, String> {
    let path = env::current_dir().unwrap();
    let command = format!("docker run --rm -v {}/contracts/input:/contracts solc-verify:0.7 /contracts/{}",
    path.to_str().unwrap(), merged_contract_file);

    let com = Command::new("cmd").args(&["/C", &command]).stdin(Stdio::piped())
    .stdout(Stdio::piped()).spawn().expect("echo command failed to start");

    let mut answer = String::new();
    match com.stdout.unwrap().read_to_string(&mut answer) {
        Err(why) => return Err(format!("Couldn't verify the contract {} :", why)),
        Ok(_) => Ok(answer)
    }
}


pub fn generate_compiled_contract(file_name: &str) -> Result <(), String> {
    let path = env::current_dir().unwrap();
    let command = format!("docker run --rm -v {}/contracts/input:/sources ethereum/solc:0.5.17 -o sources --bin --abi  /sources/{} --overwrite",
    path.to_str().unwrap(), &file_name);
    
    let com = Command::new("cmd").args(&["/C", &command]).stdin(Stdio::piped())
    .stdout(Stdio::piped()).spawn().expect("echo command failed to start");

    let mut answer = String::new();
    match com.stdout.unwrap().read_to_string(&mut answer) {
        Err(why) => panic!("Couldn't compile the contract {} : {}", why, file_name),
        Ok(_) => print!("Contract compiled with sucess:\n{}", answer),
    }
   Ok(())
}


pub fn parse_merge_files(list_verification:Vec<String>) -> Result<(), String> {
    for file_result in list_verification {
        if file_result.contains("Errors were found by the verifier.") {
            return Err("The contract could not be deployed. Errors were found by the verifier.".to_owned());
        }
    }
    Ok(())
}

pub fn search_merge_files_dir(path_contract:&Path)  ->Result<Vec<String>, String>{
    let mut list_verification = Vec::new();
    for entry in fs::read_dir(path_contract).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        if file_name.starts_with("merged_contract_") {
           let result = verify_contract(file_name);
           list_verification.push(result.unwrap()); 
        }
    }
    Ok(list_verification)
}