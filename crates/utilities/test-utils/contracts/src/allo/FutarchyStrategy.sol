// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./AlloPool.sol";

/// @title FutarchyStrategy — Market-based governance mechanism
/// @notice Uses prediction markets to decide which proposals to fund
contract FutarchyStrategy is AccessControl {
    bytes32 public constant MARKET_MAKER_ROLE = keccak256("MARKET_MAKER_ROLE");

    struct Proposal {
        string description;
        bytes32 metricHash;
        uint256 yesShares;
        uint256 noShares;
        uint256 totalShares;
        bool resolved;
        bool accepted;
        uint256 createdAt;
        uint256 resolutionTime;
        mapping(address => uint256) yesBalance;
        mapping(address => uint256) noBalance;
    }

    AlloPool public pool;
    mapping(bytes32 => Proposal) public proposals;
    bytes32[] public proposalIds;
    uint256 public marketFee; // basis points

    event ProposalCreated(bytes32 indexed proposalId, string description, bytes32 metricHash);
    event TradeExecuted(bytes32 indexed proposalId, address indexed trader, bool isYes, uint256 amount);
    event ProposalResolved(bytes32 indexed proposalId, bool accepted);

    constructor(address _pool, uint256 _marketFee) {
        pool = AlloPool(payable(_pool));
        marketFee = _marketFee;
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(MARKET_MAKER_ROLE, msg.sender);
    }

    /// @notice Create a new proposal
    function createProposal(bytes32 proposalId, string calldata description, bytes32 metricHash)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        require(proposals[proposalId].createdAt == 0, "Proposal exists");

        Proposal storage p = proposals[proposalId];
        p.description = description;
        p.metricHash = metricHash;
        p.createdAt = block.timestamp;
        p.resolutionTime = block.timestamp + 7 days;
        proposalIds.push(proposalId);

        emit ProposalCreated(proposalId, description, metricHash);
    }

    /// @notice Trade shares in a proposal
    function trade(bytes32 proposalId, bool isYes, uint256 amount) external payable {
        Proposal storage p = proposals[proposalId];
        require(p.createdAt > 0, "Proposal not found");
        require(!p.resolved, "Proposal resolved");
        require(block.timestamp < p.resolutionTime, "Trading closed");
        require(msg.value == amount, "Invalid amount");

        uint256 fee = (amount * marketFee) / 10000;
        uint256 tradeAmount = amount - fee;

        if (isYes) {
            p.yesShares += tradeAmount;
            p.yesBalance[msg.sender] += tradeAmount;
        } else {
            p.noShares += tradeAmount;
            p.noBalance[msg.sender] += tradeAmount;
        }
        p.totalShares += tradeAmount;

        emit TradeExecuted(proposalId, msg.sender, isYes, tradeAmount);
    }

    /// @notice Resolve a proposal (called by market maker)
    function resolveProposal(bytes32 proposalId, bool accepted)
        external
        onlyRole(MARKET_MAKER_ROLE)
    {
        Proposal storage p = proposals[proposalId];
        require(!p.resolved, "Already resolved");
        require(block.timestamp >= p.resolutionTime, "Resolution time not reached");

        p.resolved = true;
        p.accepted = accepted;

        if (accepted) {
            // Allocate pool funds to the proposal's recipients
            // (Implementation dependent on how proposals map to recipients)
            pool.allocate(address(this), p.totalShares);
        }

        emit ProposalResolved(proposalId, accepted);
    }

    /// @notice Get proposal status
    function getProposal(bytes32 proposalId)
        external
        view
        returns (string memory, bytes32, uint256, uint256, bool, bool)
    {
        Proposal storage p = proposals[proposalId];
        return (
            p.description,
            p.metricHash,
            p.yesShares,
            p.noShares,
            p.resolved,
            p.accepted
        );
    }

    /// @notice Get trader balance
    function getTraderBalance(bytes32 proposalId, address trader)
        external
        view
        returns (uint256 yesBalance, uint256 noBalance)
    {
        Proposal storage p = proposals[proposalId];
        return (p.yesBalance[trader], p.noBalance[trader]);
    }
}