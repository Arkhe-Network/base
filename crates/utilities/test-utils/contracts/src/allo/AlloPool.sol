// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

/// @title AlloPool — Base contract for capital allocation pools
/// @notice Manages funds, recipients, and delegates allocation to a strategy
contract AlloPool is AccessControl, ReentrancyGuard {
    using EnumerableSet for EnumerableSet.AddressSet;

    bytes32 public constant POOL_MANAGER_ROLE = keccak256("POOL_MANAGER_ROLE");
    bytes32 public constant STRATEGY_ROLE = keccak256("STRATEGY_ROLE");

    uint256 public totalAllocated;
    uint256 public totalDistributed;
    address public strategy;
    EnumerableSet.AddressSet private _recipients;

    event PoolCreated(address indexed strategy, address indexed manager);
    event FundsAllocated(address indexed recipient, uint256 amount);
    event FundsDistributed(address indexed recipient, uint256 amount);
    event StrategyUpdated(address indexed oldStrategy, address indexed newStrategy);

    constructor(address _strategy, address _manager) {
        strategy = _strategy;
        _grantRole(DEFAULT_ADMIN_ROLE, _manager);
        _grantRole(POOL_MANAGER_ROLE, _manager);
        _grantRole(STRATEGY_ROLE, _strategy);
        emit PoolCreated(_strategy, _manager);
    }

    /// @notice Allocate funds to a recipient (called by strategy)
    function allocate(address recipient, uint256 amount)
        external
        onlyRole(STRATEGY_ROLE)
        nonReentrant
    {
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be > 0");
        require(amount <= address(this).balance, "Insufficient balance");

        _recipients.add(recipient);
        totalAllocated += amount;
        emit FundsAllocated(recipient, amount);
    }

    /// @notice Distribute funds to a recipient (called by strategy)
    function distribute(address recipient, uint256 amount)
        external
        onlyRole(STRATEGY_ROLE)
        nonReentrant
    {
        require(_recipients.contains(recipient), "Recipient not found");
        require(amount > 0, "Amount must be > 0");
        require(amount <= address(this).balance, "Insufficient balance");

        totalDistributed += amount;
        (bool success, ) = payable(recipient).call{value: amount}("");
        require(success, "Transfer failed");
        emit FundsDistributed(recipient, amount);
    }

    /// @notice Update the strategy contract
    function setStrategy(address newStrategy) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _revokeRole(STRATEGY_ROLE, strategy);
        strategy = newStrategy;
        _grantRole(STRATEGY_ROLE, newStrategy);
        emit StrategyUpdated(strategy, newStrategy);
    }

    /// @notice Get all recipients
    function getRecipients() external view returns (address[] memory) {
        return _recipients.values();
    }

    /// @notice Get recipient count
    function getRecipientCount() external view returns (uint256) {
        return _recipients.length();
    }

    receive() external payable {}
}