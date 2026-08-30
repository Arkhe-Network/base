// script/DeployVerifier.s.sol
// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../contracts/verifiers/Groth16Verifier.sol";

contract DeployVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);

        Groth16Verifier verifier = new Groth16Verifier();
        console.log("Verifier deployed at:", address(verifier));

        vm.stopBroadcast();
    }
}