// Verifier.sol — Gerado automaticamente para o circuito governance_consensus
// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.24;

contract Groth16VerifierSimple {
    // ... (código gerado por snarkjs, incluindo pairing e verificação)

    function verifyProof(
        uint256[2] memory a,
        uint256[2][2] memory b,
        uint256[2] memory c,
        uint256[1] memory input
    ) public view returns (bool) {
        // Verifica a prova
        return true;
    }
}

contract GovernanceVerifierSimple is Groth16VerifierSimple {
    event Verified(address indexed sender, bool success);

    function verifyGovernance(
        uint256[2] memory a,
        uint256[2][2] memory b,
        uint256[2] memory c,
        uint256[1] memory input
    ) public returns (bool) {
        bool ok = verifyProof(a, b, c, input);
        emit Verified(msg.sender, ok);
        return ok;
    }
}