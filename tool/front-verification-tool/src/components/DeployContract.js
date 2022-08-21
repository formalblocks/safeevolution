import React, { useState, useEffect, useRef } from 'react';
import { InputText } from 'primereact/inputtext';
import { Button } from 'primereact/button';
import { Toast } from 'primereact/toast';
import { Dropdown } from 'primereact/dropdown';
import API from '../service/API';
import Backdrop from '@material-ui/core/Backdrop';
import CircularProgress from '@material-ui/core/CircularProgress';
import { makeStyles } from '@material-ui/core/styles';
import { confirmEtherTransaction } from "../service/util";
import Web3 from "web3";


const useStyles = makeStyles((theme) => ({
    backdrop: {
      zIndex: theme.zIndex.drawer + 1,
      color: '#fff',
    },
    file: {
        opacity: 0,
        position: 'absolute',
        zIndex:  2,
    }
  }));

const DeployContract = () => {

    const classes=useStyles();
    const [filesSpecification, setFilesSpecification] = useState([]);
    const [filesImplementation, setFilesImplementation] = useState([]);
    const [specificationId, setSpecificationId] = useState('');
    const toast = useRef();
    const [loading, setLoading] = useState(false);
    const [dropdownValue, setDropdownValue] = useState(null);
    const [constructorArguments, setConstructorArguments] = useState([]);

    const web3 = new Web3(window.ethereum);


    function clearVariables () {
        setFilesSpecification([]);
        setFilesImplementation([]);
        setSpecificationId('');
        setDropdownValue(null);
        setConstructorArguments([]);
    }

    useEffect(() => {
        async function fetchParameters() {
            try {

                let contracts = {
                    implementation_files: filesImplementation, 
                    file_to_be_verified: dropdownValue ? dropdownValue.name : null,
                }
                if (dropdownValue) {
                    setLoading(true); 
                    let result = await API.post(`getconstructorarguments`, contracts);
                    setConstructorArguments(result.data);
                    setLoading(false);
                    show_toast('info', 'The constructor parameters were parsed'); 
                }
            } catch (error) {
                setLoading(false);
                if(error.response && error.response.status == 400) {
                    show_toast('error', error.response.data);
                } else {
                    show_toast('error', 'It was not possible parse the constructor arguments');
                }
            }    
        }
        fetchParameters();
    }, [dropdownValue]);

    async function upload_multiple_files_specification(event) {
        let files_specification = await upload_multiple_files(event);
        setFilesSpecification(files_specification);
    }

    async function upload_multiple_files_implementation(event) {
        let files_implementation = await upload_multiple_files(event);
        setFilesImplementation(files_implementation);
    }

    function getParameters(obj) {
        let types = [];
        let values = [];
        for (let i = 0; i < obj.length; i++) {
          types.push(obj[i].variable_declaration.typ);
          values.push(obj[i].variable_value);
        }
        return web3.eth.abi.encodeParameters(types, values).slice(2);
    }  

    async function submit_form() {

        if (check_data()) {
            return;
        }

        // console.log('Connected Address ->', (await window.ethereum.request({method: 'eth_requestAccounts'}))[0]);

        try {
    
        let contracts = {
            specification_file: filesSpecification[0], 
            implementation_files: filesImplementation, 
            specification_id: specificationId,
            constructor_arguments: constructorArguments,
            file_to_be_verified: dropdownValue ? dropdownValue.name : null,
         }
        
        setLoading(true); 
        let contract = await API.post(`getcontract`, contracts);
        show_toast('info', 'Your contract was verified');
        setLoading(false); 


        const transactionParametersRegistry = { from: window.ethereum.selectedAddress, data: contract.data[2].bin, };
        const txHashRegistry = await window.ethereum.request({ method: 'eth_sendTransaction', params: [transactionParametersRegistry], });
        
        setLoading(true); 
        await confirmEtherTransaction(txHashRegistry);
        let trx_registry_receipt = await web3.eth.getTransactionReceipt(txHashRegistry);
        show_toast('info', 'Your contract was deployed');
        setLoading(false); 
   
        
        let contract_converted = contract.data[0].bin + getParameters(constructorArguments);

        const transactionParametersContract = { from: window.ethereum.selectedAddress, data: contract_converted, };
        const txHashContract = await window.ethereum.request({ method: 'eth_sendTransaction', params: [transactionParametersContract], });
        
        setLoading(true); 
        await confirmEtherTransaction(txHashContract);
        let trx_contract_receipt = await web3.eth.getTransactionReceipt(txHashContract);
        show_toast('info', 'Your contract was deployed');
        setLoading(false); 

        // let teste = new web3.eth.Contract(JSON.parse(contract.data[0].abi), trx_contract_receipt.contractAddress);
        // let teste2 = await teste.methods.get_selected().call();
        // console.log('selected Contract -> ',teste2)

       let registy_contract = new web3.eth.Contract(JSON.parse(contract.data[2].abi), trx_registry_receipt.contractAddress);

       let spec_id_bytes32 = web3.utils.asciiToHex(specificationId);
        
       setLoading(true);   
       let response_new_mapping = await registy_contract.methods.new_mapping(trx_contract_receipt.contractAddress.trim(), spec_id_bytes32)  
                                            .send({from: window.web3.currentProvider.selectedAddress, gasPrice: '2000000000000' });
        show_toast('info', 'Your contract was updated');
        setLoading(false); 


        let obj_proxy = constructorArguments.map((obj) => obj);
        
        obj_proxy.unshift({ variable_declaration: { visibility: "", typ : "address", name : "contract_address", storage_location : "",},  variable_value: trx_contract_receipt.contractAddress});
        obj_proxy.unshift({ variable_declaration: { visibility: "", typ : "bytes32", name : "spec_id", storage_location : "",},  variable_value: spec_id_bytes32});
        obj_proxy.unshift({ variable_declaration: { visibility: "", typ : "address", name : "registry_address", storage_location : "",},  variable_value: trx_registry_receipt.contractAddress});
        
        let proxy_converted = contract.data[1].bin + getParameters(obj_proxy);

        const transactionParametersProxy = { from: window.ethereum.selectedAddress, data: proxy_converted, };
        const txHashProxy = await window.ethereum.request({ method: 'eth_sendTransaction', params: [transactionParametersProxy], });
        
        setLoading(true);   
        await confirmEtherTransaction(txHashProxy); 
        show_toast('info', 'Your contract was updated');
        setLoading(false); 

        let trx_contract_proxy = await web3.eth.getTransactionReceipt(txHashProxy);

        // let proxy_contract = new web3.eth.Contract(JSON.parse(contract.data[1].abi), trx_contract_proxy.contractAddress);
        // let response_new_proxy = await proxy_contract.methods.get_selected().call();
        // console.log('response_new_proxy -> ', response_new_proxy);

        // save log

        const chainId = await window.ethereum.request({ method: 'eth_chainId', });

        let logs = {
            id : 0,
            registry_address: trx_registry_receipt.contractAddress, 
            author_address: window.ethereum.selectedAddress, 
            specification_id: specificationId,
            specification: filesSpecification[0].content,
            specification_file_name: filesSpecification[0].name,
            proxy_address: trx_contract_proxy.contractAddress, 
            proxy: contract.data[1].file,
            chain_id: chainId,
            created_at: "2022-06-29T12:58:26.233Z"
        }

        setLoading(true);   
        let log = await API.post(`savelog`, logs);
        show_toast('info', 'Deploy logs were saved');
        setLoading(false);
        
        clearVariables();

        } catch(error) {
            show_toast('error', 'There was an unexpected error');
            setLoading(false);             
        }
    }


    function configContructorParameter (value, index) {
        let args = [...constructorArguments];
        args[index].variable_value = value;
        setConstructorArguments(args);
    }

    function show_toast(severity, summary) {
        toast.current.show({ severity: severity , summary: summary, detail: '', life: 6000 });
    }
    
    function check_data() {

        if(window.ethereum == undefined){
            show_toast('error', 'Your wallet is not connected');
            return;
        }

        if (specificationId.length === 0 ){
            show_toast('error', 'You should type a specification id');
            return true;
        }
        if (filesSpecification.length === 0){
            show_toast('error', 'You should upload at least one specification file');
            return true;
        }
        if (filesSpecification.length > 1){
            show_toast('error', 'You should upload only one specification file');
            return true;
        }
        if (filesImplementation.length === 0 ){
            show_toast('error', 'You should upload at least one implementation file');
            return true;
        }
        if (!dropdownValue) {
            show_toast('error', 'You should select a contract to be verified');
            return true;
        }
        for (let i = 0; i < constructorArguments.length; i++){
            if (!constructorArguments[i].variable_value){
                show_toast('error', 'You should enter the constructor argument'); 
                return true;
            }
        }

       let i = 0;
    
        try {
          for (i; i < constructorArguments.length; i++) {
            web3.eth.abi.encodeParameters([constructorArguments[i].variable_declaration.typ], [constructorArguments[i].variable_value] ).slice(2);
    
            if (constructorArguments[i].variable_declaration.typ === "bool" && (constructorArguments[i].variable_value !== "false" 
                                    && constructorArguments[i].variable_value !== "true" )) {
              throw new Error();
            }
          }
    
        } catch(error) {
            show_toast('error', `Invalid Parameter value for ${constructorArguments[i].variable_declaration.name}`); 
            return true;
        }
     
        return false;
      }

    async function upload_multiple_files(event) {
        const files = [...event.target.files].map(file => {
          const reader = new FileReader();
          return new Promise(resolve => {
            reader.onload = () => resolve(reader.result);
            reader.readAsText(file);
          });
       
        });
        const res = await Promise.all(files);
        const files_with_name = res.map( (file, i) => {
            return {
                content: file,
                name: event.target.files[i].name,
                verify: false,
            }
        });
        return files_with_name;
      }


    return (
        <div className="grid">
              <div className="col-1"></div>
            <div className="col-10">
            {loading    ?   <Backdrop className={classes.backdrop} open>
                            <CircularProgress color="inherit" />
                        </Backdrop> : loading}
                <Toast ref={toast} />
                <div className="card">
                    <h5>Deploy New Contract</h5>
                    <div className="p-fluid formgrid grid">
                        <div className="field col-2">
                            <label htmlFor="specificationId">Specification Id</label>
                            <InputText  id="specificationId" type="text" onChange={(e) => setSpecificationId(e.target.value)}/>
                        </div>
                        <div className="field col-5">
                            <label htmlFor="specificationFile">Specification File</label><br/>
                            <Button> {filesSpecification.length > 0 ? `${filesSpecification.length} files selected` :"+ Choose"}
                            <InputText  className={classes.file} type="file" id="specificationFile" multiple onChange={(e) => upload_multiple_files_specification(e)} />
                            </Button>
                       </div>
                        <div className="field col-5">
                            <label htmlFor="implementationFile">Implementation File</label><br/>
                            <Button> {filesImplementation.length > 0 ? `${filesImplementation.length} files selected` :"+ Choose"}
                            <InputText  className={classes.file}type="file" id="implementationFile" multiple onChange={(e) => upload_multiple_files_implementation(e)} />
                            </Button>
                        </div>
                        <div className="field col-12">
                            <label htmlFor="specificationId">Select the Implementation File</label>
                            <Dropdown value={dropdownValue} onChange={(e) => setDropdownValue(e.value)} options={filesImplementation} optionLabel="name" placeholder="Select" />
                        </div>
                        
                        {
                            constructorArguments.map( (argument, i) => {
                                return <div className="field col-3" key={i}>
                                        <label key={i + "label"} htmlFor={i}> {argument.variable_declaration.typ + " " + argument.variable_declaration.name} </label>
                                        <InputText key={i + "input"} id={i} type="text" onChange={(e) => configContructorParameter(e.target.value, i)}/>
                                      </div>
                            })
                    }
                        <div className="field col-12">
                            <Button label="Submit" onClick={(e) => submit_form()} className="mr-2 mb-2"></Button>
                        </div>
                    </div>
                    </div>
                </div>
                <div className="col-1"></div>
        </div>
    )
}

const comparisonFn = function (prevProps, nextProps) {
    return prevProps.location.pathname === nextProps.location.pathname;
};

export default React.memo(DeployContract, comparisonFn);
