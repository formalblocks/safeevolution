// SPDX-License-Identifier: MIT
pragma solidity >= 0.5.0;
// pragma solidity ^0.4.8;


import "./IERC20.sol";

/// @notice  invariant  _supply  ==  __verifier_sum_uint(_balances)
contract DSToken is IERC20 {
    bool public _stopped;

    function assert(bool x) internal {
        if (!x) revert();
    }

    modifier stoppable {
        if (_stopped) revert();
        _;
    }

    function stop() public {
        _stopped = true;
    }

    function start() public {
        _stopped = false;
    }

    /// @notice  postcondition ( ( _balances[msg.sender] ==  __verifier_old_uint (_balances[msg.sender] ) - value  && msg.sender  != to ) ||   ( _balances[msg.sender] ==  __verifier_old_uint ( _balances[msg.sender]) && msg.sender  == to ) &&  success )   || !success
    /// @notice  postcondition ( ( _balances[to] ==  __verifier_old_uint ( _balances[to] ) + value  && msg.sender  != to ) ||   ( _balances[to] ==  __verifier_old_uint ( _balances[to] ) && msg.sender  == to ) &&  success )   || !success
    /// @notice  postcondition forall (address addr) (addr == msg.sender || addr == to || __verifier_old_uint(_balances[addr]) == _balances[addr]) && success || (__verifier_old_uint(_balances[addr]) == _balances[addr]) && !success
    /// @notice  emits  Transfer 
    function transfer( address to, uint value) public stoppable returns (bool success) {
        return _transfer(to, value);
    }

    /// @notice  postcondition ( ( _balances[from] ==  __verifier_old_uint (_balances[from] ) - value  &&  from  != to ) ||   ( _balances[from] ==  __verifier_old_uint ( _balances[from] ) &&  from== to ) &&  success )   || !success
    /// @notice  postcondition ( ( _balances[to] ==  __verifier_old_uint ( _balances[to] ) + value  &&  from  != to ) ||   ( _balances[to] ==  __verifier_old_uint ( _balances[to] ) &&  from  ==to ) &&  success )   || !success
    /// @notice  postcondition  ( _approvals[from ][msg.sender] ==  __verifier_old_uint (_approvals[from ][msg.sender] ) - value && success) || ( _approvals[from ][msg.sender] ==  __verifier_old_uint (_approvals[from ][msg.sender] ) && !success) ||from  == msg.sender
    /// @notice  postcondition  _approvals[from ][msg.sender]  <= __verifier_old_uint (_approvals[from ][msg.sender] ) ||  from  == msg.sender
    /// @notice  postcondition forall (address addr) (addr == from || addr == to || __verifier_old_uint(_balances[addr]) == _balances[addr]) && success || (__verifier_old_uint(_balances[addr]) == _balances[addr]) && !success
    /// @notice  emits  Transfer
    function transferFrom( address from, address to, uint value) public stoppable returns (bool success) {
        return _transferFrom(from, to, value);
    }

    /// @notice  postcondition (_approvals[msg.sender ][ spender] ==  value  &&  success) || ( _approvals[msg.sender ][ spender] ==  __verifier_old_uint ( _approvals[msg.sender ][ spender] ) && !success )    
    /// @notice  emits  Approval
    function approve( address spender, uint value ) public stoppable returns (bool success) {
        return _approve(spender, value);
    }

    function burn(uint x) public stoppable {
        assert(_balances[msg.sender] - x <= _balances[msg.sender]);
        _balances[msg.sender] -= x;
        _supply -= x;
    }
    function mint(uint x) public stoppable {
        assert(_balances[msg.sender] + x >= _balances[msg.sender]);
        _balances[msg.sender] += x;
        _supply += x;
    }


    mapping( address => uint ) _balances;
    mapping( address => mapping( address => uint ) ) _approvals;
    uint _supply;

    event Transfer(address indexed _from, address indexed _to, uint _value);
    event Approval(address indexed _owner, address indexed _spender, uint _value);    

    constructor( uint initial_balance ) public {
        _balances[msg.sender] = initial_balance;
        _supply = initial_balance;
    }

    /// @notice postcondition supply == _supply
    function totalSupply() public returns (uint supply) {
        return _supply;
    }

    /// @notice postcondition _balances[who] == value
    function balanceOf( address who ) public returns (uint value) {
        return _balances[who];
    }

    /// @notice  postcondition ( ( _balances[msg.sender] ==  __verifier_old_uint (_balances[msg.sender] ) - value  && msg.sender  != to ) ||   ( _balances[msg.sender] ==  __verifier_old_uint ( _balances[msg.sender]) && msg.sender  == to ) &&  success )   || !success
    /// @notice  postcondition ( ( _balances[to] ==  __verifier_old_uint ( _balances[to] ) + value  && msg.sender  != to ) ||   ( _balances[to] ==  __verifier_old_uint ( _balances[to] ) && msg.sender  == to ) &&  success )   || !success
    /// @notice  emits  Transfer 
    function _transfer( address to, uint value) public returns (bool success) {
        if( _balances[msg.sender] < value ) {
            revert();
        }
        if( !safeToAdd(_balances[to], value) ) {
            revert();
        }
        _balances[msg.sender] -= value;
        _balances[to] += value;
        emit Transfer( msg.sender, to, value );
        return true;
    }

    /// @notice  postcondition ( ( _balances[from] ==  __verifier_old_uint (_balances[from] ) - value  &&  from  != to ) ||   ( _balances[from] ==  __verifier_old_uint ( _balances[from] ) &&  from== to ) &&  success )   || !success
    /// @notice  postcondition ( ( _balances[to] ==  __verifier_old_uint ( _balances[to] ) + value  &&  from  != to ) ||   ( _balances[to] ==  __verifier_old_uint ( _balances[to] ) &&  from  ==to ) &&  success )   || !success
    /// @notice  postcondition  _approvals[from ][msg.sender] ==  __verifier_old_uint (_approvals[from ][msg.sender] ) - value  ||  from  == msg.sender
    /// @notice  postcondition  _approvals[from ][msg.sender]  <= __verifier_old_uint (_approvals[from ][msg.sender] ) ||  from  == msg.sender
    /// @notice  emits  Transfer
    function _transferFrom( address from, address to, uint value) public returns (bool success) {
        // if you don't have enough balance, throw
        if( _balances[from] < value ) {
            revert();
        }
        // if you don't have approval, throw
        if( _approvals[from][msg.sender] < value ) {
            revert();
        }
        if( !safeToAdd(_balances[to], value) ) {
            revert();
        }
        // transfer and return true
        _approvals[from][msg.sender] -= value;
        _balances[from] -= value;
        _balances[to] += value;
        emit Transfer( from, to, value );
        return true;
    }

    /// @notice  postcondition (_approvals[msg.sender ][ spender] ==  value  &&  success) || ( _approvals[msg.sender ][ spender] ==  __verifier_old_uint ( _approvals[msg.sender ][ spender] ) && !success )    
    /// @notice  emits  Approval
    function _approve(address spender, uint value) public returns (bool success) {
        _approvals[msg.sender][spender] = value;
        emit Approval( msg.sender, spender, value );
        return true;
    }

    /// @notice postcondition _approvals[owner][spender] == _allowance
    function allowance(address owner, address spender) public returns (uint _allowance) {
        return _approvals[owner][spender];
    }
    function safeToAdd(uint a, uint b) internal returns (bool) {
        return (a + b >= a);
    }
}
