use std::env;
use async_std::task;
use std::{path::Path, process::{Command, Stdio}};
use std::io::Read;
use std::str::FromStr;
use web3::{
    types::{Address, H160,U256},
    contract::{Contract, Options},
};
use std::fs;
use std::thread;
use std::{io};
use hex;

use crate::{checker::{check_synctatic_conformance, generate_merge_contract}, 
            parser::{parse_implementation, parse_specification,Implementation, Specification}, 
            proxy::{config_proxy}, 
            // deployer::{deploy_contract_t, upgrade_contract_t,get_connection,get_compiled_files},
            util::{write_file, get_number_argument_constructor,delete_dir_contents,copy_dir_contents, AssignedVariable,
                generate_ast_contract, verify_contract, generate_compiled_contract, parse_merge_files, search_merge_files_dir}, 
            db::{insert_on_table, select_on_table} };


pub fn verify_deploy_contract(impl_path_input: &Path, spec_path_input: &Path, imp: &Implementation, 
    spec: &Specification, spec_id: &String, parameters_and_values: &mut Vec<AssignedVariable>) -> Result<(), String> {

    if let Err(_) = fs::copy("contracts/registry.sol", "contracts/input/registry.sol") {
        return Err("Error for generating the registry contract".to_owned());
    }
    
    generate_compiled_contract("registry.sol")?;

    //copying the implementation file    
    let impl_path_output = Path::new("contracts/input").join(&impl_path_input.file_name().unwrap().to_str().unwrap());

    //copying the specification file        
    let spec_path_output = Path::new("contracts/input").join(&spec_path_input.file_name().unwrap().to_str().unwrap());    
        
   check_synctatic_conformance(&spec, &imp, false)?;    
   
    // generating merged contract
    generate_merge_contract(&spec, &imp, true)?;

    let list_verification = search_merge_files_dir(Path::new("contracts/input"));
    parse_merge_files(list_verification.unwrap())?;

    generate_compiled_contract(&impl_path_output.file_name().unwrap().to_str().unwrap())?;
       
    config_proxy(&imp, Path::new(&impl_path_output));
    
    generate_compiled_contract("implementedproxy.sol")?;

    // let log = deploy_contract_t(&spec_id, author_account, &imp.contract_name, &spec_path_output, parameters_and_values).await?;
    
    // insert_on_table(&log)?;

    Ok(())
}


pub fn get_specification(spec_url: &Path) -> Result<Specification, String>{

    let spec_path_output = Path::new("contracts/input").join(&spec_url.file_name().unwrap().to_str().unwrap());    
    fs::copy(&spec_url.to_path_buf(), &spec_path_output);

    generate_ast_contract(spec_path_output.file_name().unwrap().to_str().unwrap());

    let inp_path = Path::new("contracts/input");
    let spec_json_path = format!("{}_json.ast", Path::new(&spec_path_output).file_name().unwrap().to_str().unwrap());
    let spec = parse_specification(&inp_path.join(&spec_json_path).as_path())?;
    Ok(spec)
}

pub fn get_implementation(impl_url: &Path) -> Result<Implementation, String>{

    let impl_path_output = Path::new("contracts/input").join(&impl_url.file_name().unwrap().to_str().unwrap());
    fs::copy(&impl_url.to_path_buf(), &impl_path_output);

    generate_ast_contract(impl_path_output.file_name().unwrap().to_str().unwrap());

    let imp_path = Path::new("contracts/input");
    let impl_json_path = format!("{}_json.ast", Path::new(&impl_path_output).file_name().unwrap().to_str().unwrap());
    
    let imp = parse_implementation(&imp_path.join(&impl_json_path).as_path())?;

    Ok(imp)
}


pub fn upgrade_contract(impl_path_input: &Path, imp: &Implementation, spec_id: &String, author_account: &Address, chain_id:&String) -> Result<(), String> {

    //copying registry
    if let Err(_) = fs::copy("contracts/registry.sol", "contracts/input/registry.sol") {
        return Err("Error for generating the registry contract".to_owned());
    };

    //copying the implementation file    
    // let impl_path_output = Path::new("contracts/input").join(&impl_path_input.file_name().unwrap().to_str().unwrap());
   
    // if let Err(_) = fs::copy(&impl_path_input, &impl_path_output) {
    //     return Err("Error for fetching the specification".to_owned());
    // };

    let log = select_on_table(spec_id, &format!("{:?}", author_account), &chain_id).unwrap();

    if log.len() == 0 {
        return Err(format!("Could not find a specification for this id {}", &spec_id));
    }

    write_file(&log[0].specification, &log[0].specification_file_name );
    write_file(&log[0].proxy, &"implementedproxy.sol".to_string() );
    
    //generating ast for the specification file
    generate_ast_contract(&log[0].specification_file_name)?;
    
    let imp_path = Path::new("contracts/input");
    
    let spec = get_specification(imp_path.join(&log[0].specification_file_name).as_path()).expect("Expected specification");
    // let imp = get_implementation(&impl_path_input).unwrap();

    check_synctatic_conformance(&spec, &imp, true)?;  

    generate_merge_contract(&spec, &imp, true)?;

    let merged_contract_file = format!("{}_{}.sol", "merged_contract", imp.contract_name);

    verify_contract(merged_contract_file)?;
 
    generate_compiled_contract(&impl_path_input.file_name().unwrap().to_str().unwrap())?;
    
    generate_compiled_contract(&"implementedproxy.sol")?;

    // upgrade_contract_t(&log[0].specification_id, Address::from_str(&log[0].author_address).unwrap(), 
    // Address::from_str(&log[0].registry_address).unwrap(), Address::from_str(&log[0].proxy_address).unwrap(), &impl_path_output).await?;

    Ok(())
}

