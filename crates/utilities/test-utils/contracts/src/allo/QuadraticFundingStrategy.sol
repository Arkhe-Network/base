// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./AlloPool.sol";
import "@openzeppelin/contracts/utils/math/Math.sol";

/// @title QuadraticFundingStrategy — QF matching mechanism
/// @notice Matches small contributions with larger pool funds
contract QuadraticFundingStrategy is AccessControl {
    using Math for uint256;

    bytes32 public constant CONTRIBUTOR_ROLE = keccak256("CONTRIBUTOR_ROLE");

    struct Contribution {
        address contributor;
        uint256 amount;
        uint256 timestamp;
    }

    struct RecipientData {
        uint256 totalContributions;
        uint256 uniqueContributors;
        mapping(address => uint256) contributions;
        Contribution[] contributionHistory;
    }

    AlloPool public pool;
    uint256 public matchingPool;
    uint256 public roundStart;
    uint256 public roundEnd;
    bool public isFinalized;

    mapping(address => RecipientData) public recipientData;
    mapping(address => bool) public isRecipient;
    address[] public recipients;

    event ContributionMade(address indexed recipient, address indexed contributor, uint256 amount);
    event MatchingCalculated(address indexed recipient, uint256 matchingAmount);
    event RoundFinalized();

    constructor(address _pool, uint256 _matchingPool, uint256 _duration) {
        pool = AlloPool(payable(_pool));
        matchingPool = _matchingPool;
        roundStart = block.timestamp;
        roundEnd = block.timestamp + _duration;
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(CONTRIBUTOR_ROLE, msg.sender);
    }

    /// @notice Contribute to a recipient
    function contribute(address recipient) external payable {
        require(block.timestamp >= roundStart && block.timestamp <= roundEnd, "Round not active");
        require(msg.value > 0, "Contribution must be > 0");
        require(isRecipient[recipient], "Recipient not registered");

        RecipientData storage data = recipientData[recipient];
        if (data.contributions[msg.sender] == 0) {
            data.uniqueContributors++;
        }
        data.totalContributions += msg.value;
        data.contributions[msg.sender] += msg.value;
        data.contributionHistory.push(Contribution({
            contributor: msg.sender,
            amount: msg.value,
            timestamp: block.timestamp
        }));

        emit ContributionMade(recipient, msg.sender, msg.value);
    }

    /// @notice Register a recipient
    function registerRecipient(address recipient) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(!isRecipient[recipient], "Already registered");
        isRecipient[recipient] = true;
        recipients.push(recipient);
    }

    /// @notice Calculate matching amounts using quadratic funding formula
    /// @dev Matching = matchingPool * (sqrt(totalContributions))² / Σ(sqrt(totalContributions))²
    function calculateMatching() external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(block.timestamp > roundEnd, "Round not ended");
        require(!isFinalized, "Already finalized");

        uint256 totalSqrtSum = 0;
        uint256[] memory sqrtValues = new uint256[](recipients.length);

        for (uint256 i = 0; i < recipients.length; i++) {
            address recipient = recipients[i];
            uint256 sqrtVal = Math.sqrt(recipientData[recipient].totalContributions);
            sqrtValues[i] = sqrtVal;
            totalSqrtSum += sqrtVal;
        }

        for (uint256 i = 0; i < recipients.length; i++) {
            address recipient = recipients[i];
            uint256 matchingAmount = totalSqrtSum > 0
                ? (matchingPool * sqrtValues[i]) / totalSqrtSum
                : 0;

            if (matchingAmount > 0) {
                recipientData[recipient].totalContributions += matchingAmount;
                pool.allocate(recipient, matchingAmount);
                emit MatchingCalculated(recipient, matchingAmount);
            }
        }

        isFinalized = true;
        emit RoundFinalized();
    }

    /// @notice Distribute funds to a recipient
    function distribute(address recipient, uint256 amount) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(isFinalized, "Round not finalized");
        pool.distribute(recipient, amount);
    }

    /// @notice Get contribution history for a recipient
    function getContributions(address recipient) external view returns (Contribution[] memory) {
        return recipientData[recipient].contributionHistory;
    }

    /// @notice Get total contributions for a recipient
    function getTotalContributions(address recipient) external view returns (uint256) {
        return recipientData[recipient].totalContributions;
    }
}