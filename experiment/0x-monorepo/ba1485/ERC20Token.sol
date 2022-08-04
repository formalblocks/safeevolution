
pragma solidity >=0.4.24 <0.9.0;

/// @notice  invariant  totalSupply  ==  __verifier_sum_uint(balances)
contract ERC20Token  {

    string  INSUFFICIENT_BALANCE = "ERC20_INSUFFICIENT_BALANCE";
    string  INSUFFICIENT_ALLOWANCE = "ERC20_INSUFFICIENT_ALLOWANCE";
    string  OVERFLOW = "Transfer would result in an overflow.";

    mapping (address => uint256) balances;
    mapping (address => mapping (address => uint256)) allowed;

    uint256 public totalSupply;

    /// @notice  postcondition ( ( balances[msg.sender] ==  __verifier_old_uint (balances[msg.sender] ) - _value  && msg.sender  != _to ) || ( balances[msg.sender] ==  __verifier_old_uint ( balances[msg.sender]) && msg.sender  == _to ) &&  success ) || !success
    /// @notice  postcondition ( ( balances[_to] ==  __verifier_old_uint ( balances[_to] ) + _value  && msg.sender  != _to ) || ( balances[_to] ==  __verifier_old_uint ( balances[_to] ) && msg.sender  == _to ) &&  success ) || !success
    /// @notice  postcondition forall (address addr) addr == msg.sender || addr == _to || __verifier_old_uint(balances[addr]) == balances[addr]
    /// @notice  emits Transfer 
    function transfer(address _to, uint256 _value)
        public
        returns (bool success)
    {
        require(
            balances[msg.sender] >= _value,
            INSUFFICIENT_BALANCE
        );
        require(
            balances[_to] + _value >= balances[_to],
            OVERFLOW
        );
        balances[msg.sender] -= _value;
        balances[_to] += _value;
        emit Transfer(msg.sender, _to, _value);
        return true;
    }

    /// @notice  postcondition ( ( balances[_from] ==  __verifier_old_uint (balances[_from] ) - _value  &&  _from  != _to ) || ( balances[_from] ==  __verifier_old_uint ( balances[_from] ) &&  _from == _to ) &&  success ) || !success
    /// @notice  postcondition ( ( balances[_to] ==  __verifier_old_uint ( balances[_to] ) + _value  &&  _from  != _to ) || ( balances[_to] ==  __verifier_old_uint ( balances[_to] ) &&  _from  == _to ) &&  success ) || !success
    /// @notice  postcondition ( allowed[_from ][msg.sender] ==  __verifier_old_uint (allowed[_from ][msg.sender] ) - _value ) || ( allowed[_from ][msg.sender] ==  __verifier_old_uint (allowed[_from ][msg.sender] ) && !success) ||  _from  == msg.sender
    /// @notice  postcondition  allowed[_from ][msg.sender]  <= __verifier_old_uint (allowed[_from ][msg.sender] ) ||  _from  == msg.sender
    /// @notice  postcondition forall (address addr) addr == _from || addr == _to || __verifier_old_uint(balances[addr]) == balances[addr]
    /// @notice  emits  Transfer
    function transferFrom(address _from, address _to, uint256 _value)
        public
        returns (bool success)
    {
        require(
            balances[_from] >= _value,
            INSUFFICIENT_BALANCE
        );
        require(
            allowed[_from][msg.sender] >= _value,
            INSUFFICIENT_ALLOWANCE
        );
        require(
            balances[_to] + _value >= balances[_to],
            OVERFLOW
        );
        balances[_to] += _value;
        balances[_from] -= _value;
        allowed[_from][msg.sender] -= _value;
        emit Transfer(_from, _to, _value);
        return true;
    }

    /// @notice  postcondition (allowed[msg.sender ][ _spender] ==  _value  &&  success) || ( allowed[msg.sender ][ _spender] ==  __verifier_old_uint ( allowed[msg.sender ][ _spender] ) && !success )    
    /// @notice  emits  Approval
    function approve(address _spender, uint256 _value)
        public
        returns (bool success)
    {
        allowed[msg.sender][_spender] = _value;
        emit Approval(msg.sender, _spender, _value);
        return true;
    }

     /// @notice postcondition balances[_owner] == balance
    function balanceOf(address _owner)
        public view
        returns (uint256 balance)
    {
        return balances[_owner];
    }

    /// @notice postcondition allowed[_owner][_spender] == remaining
    function allowance(address _owner, address _spender)
        public
        view
        returns (uint256 remaining)
    {
        return allowed[_owner][_spender];
    }

      event Transfer(
        address indexed _from,
        address indexed _to,
        uint256 _value
    );
    
    event Approval(
        address indexed _owner,
        address indexed _spender,
        uint256 _value
    );
}
