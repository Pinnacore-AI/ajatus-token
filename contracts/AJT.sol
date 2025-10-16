// SPDX-License-Identifier: MIT
// Ajatuskumppani — built in Finland, by the free minds of Pinnacore.

pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title AjatusToken (AJT)
 * @dev The utility token for the Ajatuskumppani ecosystem.
 * 
 * Features:
 * - ERC-20 compliant
 * - Burnable (to prevent inflation)
 * - Mintable by owner (for Proof-of-Contribution rewards)
 * - 18 decimal places
 */
contract AjatusToken is ERC20, ERC20Burnable, Ownable {
    
    // Maximum supply: 1 billion tokens
    uint256 public constant MAX_SUPPLY = 1_000_000_000 * 10**18;
    
    // Events
    event TokensMinted(address indexed to, uint256 amount, string reason);
    event TokensBurned(address indexed from, uint256 amount);
    
    /**
     * @dev Constructor that gives msg.sender all of the initial supply.
     * @param initialSupply The initial supply of tokens (in whole tokens, not wei)
     */
    constructor(uint256 initialSupply) ERC20("AjatusToken", "AJT") Ownable(msg.sender) {
        require(initialSupply * 10**18 <= MAX_SUPPLY, "Initial supply exceeds max supply");
        _mint(msg.sender, initialSupply * 10**18);
    }
    
    /**
     * @dev Mint new tokens as rewards for Proof-of-Contribution.
     * Can only be called by the owner (the PoC contract).
     * 
     * @param to The address to receive the tokens
     * @param amount The amount of tokens to mint (in wei)
     * @param reason The reason for minting (e.g., "compute", "storage")
     */
    function mint(address to, uint256 amount, string memory reason) public onlyOwner {
        require(totalSupply() + amount <= MAX_SUPPLY, "Minting would exceed max supply");
        _mint(to, amount);
        emit TokensMinted(to, amount, reason);
    }
    
    /**
     * @dev Burn tokens from the caller's account.
     * @param amount The amount of tokens to burn (in wei)
     */
    function burn(uint256 amount) public override {
        super.burn(amount);
        emit TokensBurned(msg.sender, amount);
    }
    
    /**
     * @dev Get the current circulating supply.
     * @return The total supply of tokens
     */
    function circulatingSupply() public view returns (uint256) {
        return totalSupply();
    }
}

