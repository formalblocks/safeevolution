import React,  { useState, useRef, useEffect }  from 'react';
import { Toast } from 'primereact/toast';
import classNames from 'classnames';
import { Tooltip } from 'primereact/tooltip';

export const AppTopbar = (props) => {

    const [connected, setConnected] = useState(false);

    useEffect(() => {
        checkWallet();
    }, []);

    const checkWallet = () => {
        if (window.ethereum &&  window.ethereum.selectedAddress) {
            setConnected(true);
        }
    }

    const connectWallet = () => {
        if (window.ethereum) {
          window.ethereum.request({ method: "eth_requestAccounts" }).then((res) => { show_toast("info", "Your wallet is connected"); setConnected(true);  });
        } else {
            show_toast("error", "Please, connect your metamask wallet");
        }
      };

    const toast = useRef();
    function show_toast(severity, summary) {
        toast.current.show({ severity: severity , summary: summary, detail: '', life: 6000 });
    }

    return (
        <div className="layout-topbar">
            <Toast ref={toast} />
             <button type="button" className="p-link  layout-menu-button layout-topbar-button" onClick={props.onToggleMenuClick}>
                <i className="pi pi-bars"/>
            </button>
                <img style={{height: 60}} src={props.layoutColorMode === 'light' ? 'assets/layout/images/logo-dark.jpg' : 'assets/layout/images/logo-dark.jpg'} alt="logo"/>
                <span>Contract Verification Tool</span>
            <button type="button" className="p-link layout-topbar-menu-button layout-topbar-button" onClick={props.onMobileTopbarMenuClick}>
                <i className="pi pi-ellipsis-v" />
            </button>

                 <ul className={classNames("layout-topbar-menu lg:flex origin-top", {'layout-topbar-menu-mobile-active': props.mobileTopbarMenuActive })}>
                    <li>
                    <Tooltip target=".custom-target-icon" position='left' />
                        <button className="custom-target-icon p-link layout-topbar-button" data-pr-tooltip="Connect your wallet" onClick={connectWallet}>
                            <i className="custom-target-icon pi pi-wallet" style={{color: connected ? "green" : "red"}} />
                            <span>Events</span>
                        </button>
                    </li>
                    {/* <li>
                        <button className="p-link layout-topbar-button" onClick={props.onMobileSubTopbarMenuClick}>
                            <i className="pi pi-cog"/>
                            <span>Settings</span>
                        </button>
                    </li>
                    <li>
                        <button className="p-link layout-topbar-button" onClick={props.onMobileSubTopbarMenuClick}>
                            <i className="pi pi-user"/>
                            <span>Profile</span>
                        </button>
                    </li> */}
                </ul> 
        </div>
    );
}
