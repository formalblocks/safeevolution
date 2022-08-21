import React, { useState, useEffect, useRef } from 'react';
import { InputText } from 'primereact/inputtext';
import { Button } from 'primereact/button';
import { Toast } from 'primereact/toast';
import { Dropdown } from 'primereact/dropdown';
import API from '../service/API';
// import Web3 from "./Web3";
import Backdrop from '@material-ui/core/Backdrop';
import CircularProgress from '@material-ui/core/CircularProgress';
import { makeStyles } from '@material-ui/core/styles';
import Web3 from "web3";
import { confirmEtherTransaction } from "../service/util";


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

    const [filesImplementation, setFilesImplementation] = useState([]);
    const [specificationId, setSpecificationId] = useState('');
    const toast = useRef();
    const [loading, setLoading] = useState(false);
    const [dropdownValue, setDropdownValue] = useState(null);
    const classes=useStyles();

    async function upload_multiple_files_implementation(event) {
        let files_implementation = await upload_multiple_files(event);
        setFilesImplementation(files_implementation);
    }

    function clearVariables () {
        setFilesImplementation([]);
        setSpecificationId('');
        setDropdownValue(null);
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

      function show_toast(severity, summary) {
        toast.current.show({ severity: severity , summary: summary, detail: '', life: 3000 });
      } 


      async function submit_form() {

        if (check_data()) {
            return;
        } 

        const chainId = await window.ethereum.request({ method: 'eth_chainId', });

        // console.log('Connected Address ->', (await window.ethereum.request({method: 'eth_requestAccounts'}))[0]);
        let web3 = new Web3(window.ethereum);

        // deploy contract

        let contracts = {
            implementation_files: filesImplementation, 
            specification_id: specificationId,
            file_to_be_verified: dropdownValue.name,
         }

        try {
            setLoading(true); 
            let contract = await API.post(`upgradecontract/${window.ethereum.selectedAddress}/${chainId}`, contracts);
            show_toast('info', 'Your was verified');
            setLoading(false); 

            let contract_converted = contract.data[0].bin;
            const transactionParametersContract = { from: window.ethereum.selectedAddress, data: contract_converted, };
            const txHashContract = await window.ethereum.request({ method: 'eth_sendTransaction', params: [transactionParametersContract], });
            
            setLoading(true); 
            await confirmEtherTransaction(txHashContract);
            let trx_contract_receipt = await web3.eth.getTransactionReceipt(txHashContract);
            show_toast('info', 'Your contract was deployed');
            setLoading(false); 

            // update registry

            let registy_contract = new web3.eth.Contract(JSON.parse(contract.data[1].abi), contract.data[1].address);
            let spec_id_bytes32 = web3.utils.asciiToHex(specificationId);

            // console.log('address -> ', trx_contract_receipt.contractAddress, 'spec_id ->',  spec_id_bytes32)
            
        setLoading(true); 
        let response_new_mapping = await registy_contract.methods.new_mapping(trx_contract_receipt.contractAddress.trim(), spec_id_bytes32)  
                                                .send({from: window.web3.currentProvider.selectedAddress, gasPrice: '2000000000000' });
            show_toast('info', 'Your contract was updated');
            setLoading(false); 
            // update proxy proxy_abi

            let proxy_contract = new web3.eth.Contract(JSON.parse(contract.data[2].abi), contract.data[2].address);

            setLoading(true); 
            let response_upgrade = await proxy_contract.methods.upgrade(trx_contract_receipt.contractAddress.trim())  
            .send({from: window.web3.currentProvider.selectedAddress, gasPrice: '2000000000000' });
            show_toast('info', 'Your contract was updated');
            setLoading(false);
            
            clearVariables();

            // let response_new_proxy = await proxy_contract.methods.get_selected().call();
            // console.log('response_new_proxy -> ', response_new_proxy);

        } catch(error) {
            show_toast('error', 'There was an unexpected error');
            setLoading(false);             
        }
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
        if (!dropdownValue) {
            show_toast('error', 'You should select a contract to be verified');
            return true;
        }
        if (filesImplementation.length === 0 ){
            show_toast('error', 'You should upload at least one implementation file');
            return true;
        }
        return false;
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
                    <h5>Upgrade a Contract</h5>
                    <div className="p-fluid formgrid grid">
                        <div className="field col-2">
                            <label htmlFor="specificationId">Specification Id</label>
                            <InputText id="specificationId" type="text" onChange={(e) => setSpecificationId(e.target.value)}/>
                        </div>
                   
                        <div className="field col-5">
                            <label htmlFor="implementationFile">Implementation File</label><br/>
                            <Button> {filesImplementation.length > 0 ? `${filesImplementation.length} files selected` :"+ Choose"}
                            <InputText  className={classes.file}type="file" id="implementationFile" multiple onChange={(e) => upload_multiple_files_implementation(e)} />
                            </Button>
                        </div>
                        <div className="field col-12">
                            <label htmlFor="specificationId">Select the Implementation File</label><br />
                            <Dropdown value={dropdownValue} onChange={(e) => setDropdownValue(e.value)} options={filesImplementation} optionLabel="name" placeholder="Select" />
                        </div>
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
