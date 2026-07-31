// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./AlloPool.sol";

/// @title RetroFundingStrategy — Retroactive funding mechanism
/// @notice Rewards projects after they've demonstrated impact
contract RetroFundingStrategy is AccessControl {
    bytes32 public constant EVALUATOR_ROLE = keccak256("EVALUATOR_ROLE");

    struct Project {
        address projectAddress;
        string metadataURI;
        uint256 impactScore;
        uint256 requestedAmount;
        uint256 allocatedAmount;
        bool evaluated;
        bool funded;
        address[] evaluators;
        mapping(address => uint256) evaluatorScores;
    }

    AlloPool public pool;
    uint256 public totalPool;
    uint256 public evaluationPeriodEnd;
    mapping(address => Project) public projects;
    address[] public projectAddresses;

    event ProjectRegistered(address indexed project, string metadataURI);
    event ProjectEvaluated(address indexed project, uint256 score);
    event ProjectFunded(address indexed project, uint256 amount);

    constructor(address _pool, uint256 _evaluationPeriod) {
        pool = AlloPool(payable(_pool));
        evaluationPeriodEnd = block.timestamp + _evaluationPeriod;
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(EVALUATOR_ROLE, msg.sender);
    }

    /// @notice Register a project for retroactive funding
    function registerProject(address project, string calldata metadataURI)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        require(projects[project].projectAddress == address(0), "Project exists");
        require(block.timestamp < evaluationPeriodEnd, "Registration closed");

        Project storage p = projects[project];
        p.projectAddress = project;
        p.metadataURI = metadataURI;
        projectAddresses.push(project);

        emit ProjectRegistered(project, metadataURI);
    }

    /// @notice Evaluate a project (called by evaluators)
    function evaluateProject(address project, uint256 score)
        external
        onlyRole(EVALUATOR_ROLE)
    {
        require(projects[project].projectAddress != address(0), "Project not found");
        require(!projects[project].evaluated, "Already evaluated");
        require(score <= 100, "Score must be <= 100");

        Project storage p = projects[project];
        p.evaluatorScores[msg.sender] = score;
        p.evaluators.push(msg.sender);

        // If enough evaluations, calculate average
        if (p.evaluators.length >= 3) {
            uint256 totalScore = 0;
            for (uint256 i = 0; i < p.evaluators.length; i++) {
                totalScore += p.evaluatorScores[p.evaluators[i]];
            }
            p.impactScore = totalScore / p.evaluators.length;
            p.evaluated = true;
            emit ProjectEvaluated(project, p.impactScore);
        }
    }

    /// @notice Fund projects based on impact scores
    function fundProjects() external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(block.timestamp >= evaluationPeriodEnd, "Evaluation period not ended");

        uint256 totalScore = 0;
        uint256 eligibleCount = 0;

        for (uint256 i = 0; i < projectAddresses.length; i++) {
            Project storage p = projects[projectAddresses[i]];
            if (p.evaluated && p.impactScore > 0) {
                totalScore += p.impactScore;
                eligibleCount++;
            }
        }

        require(eligibleCount > 0, "No eligible projects");

        for (uint256 i = 0; i < projectAddresses.length; i++) {
            Project storage p = projects[projectAddresses[i]];
            if (p.evaluated && p.impactScore > 0) {
                uint256 amount = (totalPool * p.impactScore) / totalScore;
                if (amount > 0) {
                    p.allocatedAmount = amount;
                    p.funded = true;
                    pool.allocate(p.projectAddress, amount);
                    emit ProjectFunded(p.projectAddress, amount);
                }
            }
        }
    }

    /// @notice Set total pool amount
    function setTotalPool(uint256 amount) external onlyRole(DEFAULT_ADMIN_ROLE) {
        totalPool = amount;
    }

    /// @notice Get project evaluators
    function getEvaluators(address project) external view returns (address[] memory) {
        return projects[project].evaluators;
    }
}