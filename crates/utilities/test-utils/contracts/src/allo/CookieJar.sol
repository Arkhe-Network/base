// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./AlloPool.sol";

/// @title CookieJar — Micro-grants mechanism
/// @notice Fast, low-friction grants for trusted contributors
contract CookieJar is AccessControl {
    bytes32 public constant TRUSTED_ROLE = keccak256("TRUSTED_ROLE");

    struct Contribution {
        address contributor;
        string description;
        uint256 amount;
        uint256 timestamp;
        bool approved;
    }

    AlloPool public pool;
    uint256 public maxGrantAmount;
    uint256 public cooldownPeriod;
    mapping(address => uint256) public lastClaim;
    mapping(address => Contribution[]) public contributions;
    Contribution[] public allContributions;

    event GrantClaimed(address indexed contributor, uint256 amount, string description);
    event GrantApproved(address indexed contributor, uint256 index);

    constructor(address _pool, uint256 _maxGrantAmount, uint256 _cooldownPeriod) {
        pool = AlloPool(payable(_pool));
        maxGrantAmount = _maxGrantAmount;
        cooldownPeriod = _cooldownPeriod;
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(TRUSTED_ROLE, msg.sender);
    }

    /// @notice Claim a micro-grant
    function claimGrant(string calldata description, uint256 amount) external {
        require(amount <= maxGrantAmount, "Amount exceeds max");
        require(block.timestamp >= lastClaim[msg.sender] + cooldownPeriod, "Cooldown active");
        require(amount <= address(pool).balance, "Insufficient pool balance");

        lastClaim[msg.sender] = block.timestamp;

        Contribution memory contrib = Contribution({
            contributor: msg.sender,
            description: description,
            amount: amount,
            timestamp: block.timestamp,
            approved: false
        });

        contributions[msg.sender].push(contrib);
        allContributions.push(contrib);

        // Direct allocation (can be approved later)
        pool.allocate(msg.sender, amount);
        emit GrantClaimed(msg.sender, amount, description);
    }

    /// @notice Approve a grant (by trusted role)
    function approveGrant(address contributor, uint256 index) external onlyRole(TRUSTED_ROLE) {
        require(index < contributions[contributor].length, "Invalid index");
        Contribution storage contrib = contributions[contributor][index];
        require(!contrib.approved, "Already approved");

        contrib.approved = true;
        emit GrantApproved(contributor, index);
    }

    /// @notice Get contributor's grants
    function getContributions(address contributor)
        external
        view
        returns (Contribution[] memory)
    {
        return contributions[contributor];
    }

    /// @notice Get all grants
    function getAllContributions() external view returns (Contribution[] memory) {
        return allContributions;
    }
}