// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./AlloPool.sol";

/// @title DedicatedDomainAllocation — DDA mechanism
/// @notice Delegates funding power to trusted stewards within specific domains
contract DedicatedDomainAllocation is AccessControl {
    bytes32 public constant STEWARD_ROLE = keccak256("STEWARD_ROLE");

    struct Domain {
        string name;
        string description;
        address steward;
        uint256 allocatedBudget;
        uint256 spentBudget;
        bool active;
        mapping(address => bool) approvedRecipients;
        address[] recipients;
    }

    AlloPool public pool;
    mapping(bytes32 => Domain) public domains;
    bytes32[] public domainIds;

    event DomainCreated(bytes32 indexed domainId, string name, address indexed steward);
    event DomainFunded(bytes32 indexed domainId, uint256 amount);
    event DomainAllocation(bytes32 indexed domainId, address indexed recipient, uint256 amount);
    event RecipientApproved(bytes32 indexed domainId, address indexed recipient);

    constructor(address _pool) {
        pool = AlloPool(payable(_pool));
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    /// @notice Create a new domain with a steward
    function createDomain(
        bytes32 domainId,
        string calldata name,
        string calldata description,
        address steward
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(domains[domainId].steward == address(0), "Domain exists");
        require(steward != address(0), "Invalid steward");

        Domain storage domain = domains[domainId];
        domain.name = name;
        domain.description = description;
        domain.steward = steward;
        domain.active = true;
        domainIds.push(domainId);

        _grantRole(STEWARD_ROLE, steward);
        emit DomainCreated(domainId, name, steward);
    }

    /// @notice Fund a domain
    function fundDomain(bytes32 domainId, uint256 amount) external payable {
        require(domains[domainId].active, "Domain inactive");
        require(msg.value == amount || amount == 0, "Invalid amount");

        Domain storage domain = domains[domainId];
        domain.allocatedBudget += amount;

        emit DomainFunded(domainId, amount);
    }

    /// @notice Approve a recipient for a domain (only steward)
    function approveRecipient(bytes32 domainId, address recipient)
        external
        onlyRole(STEWARD_ROLE)
    {
        require(domains[domainId].steward == msg.sender, "Not steward");
        require(!domains[domainId].approvedRecipients[recipient], "Already approved");

        Domain storage domain = domains[domainId];
        domain.approvedRecipients[recipient] = true;
        domain.recipients.push(recipient);

        emit RecipientApproved(domainId, recipient);
    }

    /// @notice Allocate funds to a recipient within a domain (only steward)
    function allocateToRecipient(
        bytes32 domainId,
        address recipient,
        uint256 amount
    ) external onlyRole(STEWARD_ROLE) {
        require(domains[domainId].steward == msg.sender, "Not steward");
        require(domains[domainId].active, "Domain inactive");
        require(domains[domainId].approvedRecipients[recipient], "Recipient not approved");
        require(domains[domainId].spentBudget + amount <= domains[domainId].allocatedBudget, "Insufficient budget");

        Domain storage domain = domains[domainId];
        domain.spentBudget += amount;

        pool.allocate(recipient, amount);
        emit DomainAllocation(domainId, recipient, amount);
    }

    /// @notice Get all domains
    function getDomains() external view returns (bytes32[] memory) {
        return domainIds;
    }

    /// @notice Get domain recipients
    function getDomainRecipients(bytes32 domainId) external view returns (address[] memory) {
        return domains[domainId].recipients;
    }
}