// script/DeployBLSVerifier.s.sol
// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../contracts/verifiers/BLS12_381Verifier.sol";

contract DeployBLSVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);

        BLS12_381Verifier verifier = new BLS12_381Verifier();
        console.log("BLS12-381 Verifier deployed at:", address(verifier));

        vm.stopBroadcast();
    }
}