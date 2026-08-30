// script/DeploySTARKVerifier.s.sol
// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../contracts/verifiers/Verifier.sol";

contract DeploySTARKVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);

        GovernanceVerifierSimple verifier = new GovernanceVerifierSimple();
        console.log("STARK Verifier deployed at:", address(verifier));

        vm.stopBroadcast();
    }
}