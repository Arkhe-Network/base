// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./AlloPool.sol";

/// @title DirectToContractIncentives — Activity-based funding
/// @notice Routes funding directly to smart contracts based on onchain activity
contract DirectToContractIncentives is AccessControl {
    bytes32 public constant ORACLE_ROLE = keccak256("ORACLE_ROLE");

    struct IncentiveRule {
        address targetContract;
        bytes4 functionSelector;
        uint256 rewardPerUnit;
        uint256 maxReward;
        uint256 totalRewarded;
        bool active;
        mapping(address => uint256) claimed;
    }

    AlloPool public pool;
    mapping(bytes32 => IncentiveRule) public rules;
    bytes32[] public ruleIds;

    event RuleCreated(bytes32 indexed ruleId, address targetContract, bytes4 selector);
    event IncentiveClaimed(bytes32 indexed ruleId, address indexed caller, uint256 amount);

    constructor(address _pool) {
        pool = AlloPool(payable(_pool));
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ORACLE_ROLE, msg.sender);
    }

    /// @notice Create an incentive rule
    function createRule(
        bytes32 ruleId,
        address targetContract,
        bytes4 functionSelector,
        uint256 rewardPerUnit,
        uint256 maxReward
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(rules[ruleId].targetContract == address(0), "Rule exists");

        IncentiveRule storage rule = rules[ruleId];
        rule.targetContract = targetContract;
        rule.functionSelector = functionSelector;
        rule.rewardPerUnit = rewardPerUnit;
        rule.maxReward = maxReward;
        rule.active = true;
        ruleIds.push(ruleId);

        emit RuleCreated(ruleId, targetContract, functionSelector);
    }

    /// @notice Claim rewards based on onchain activity
    function claimReward(bytes32 ruleId, uint256 units) external {
        IncentiveRule storage rule = rules[ruleId];
        require(rule.active, "Rule inactive");
        require(rule.rewardPerUnit > 0, "No reward");
        require(units > 0, "Units must be > 0");

        uint256 reward = units * rule.rewardPerUnit;
        require(rule.totalRewarded + reward <= rule.maxReward, "Max reward exceeded");

        rule.totalRewarded += reward;
        rule.claimed[msg.sender] += reward;

        pool.allocate(msg.sender, reward);
        emit IncentiveClaimed(ruleId, msg.sender, reward);
    }

    /// @notice Oracle-based verification of activity (simplified)
    function verifyAndClaim(
        bytes32 ruleId,
        address caller,
        uint256 units,
        bytes calldata /* proof */
    ) external onlyRole(ORACLE_ROLE) {
        // In production, this would verify the proof
        // For now, we trust the oracle
        IncentiveRule storage rule = rules[ruleId];
        require(rule.active, "Rule inactive");

        uint256 reward = units * rule.rewardPerUnit;
        require(rule.totalRewarded + reward <= rule.maxReward, "Max reward exceeded");

        rule.totalRewarded += reward;
        rule.claimed[caller] += reward;

        pool.allocate(caller, reward);
        emit IncentiveClaimed(ruleId, caller, reward);
    }

    /// @notice Get rule details
    function getRule(bytes32 ruleId)
        external
        view
        returns (address, bytes4, uint256, uint256, uint256, bool)
    {
        IncentiveRule storage rule = rules[ruleId];
        return (
            rule.targetContract,
            rule.functionSelector,
            rule.rewardPerUnit,
            rule.maxReward,
            rule.totalRewarded,
            rule.active
        );
    }
}