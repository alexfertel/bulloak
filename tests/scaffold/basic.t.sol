// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.0;

contract HashPairTest {
  function test_ShouldNeverRevert() external {
    // It should never revert.
  }

  function test_WhenFirstArgIsSmallerThanSecondArg() external {
    // It should match the result of `keccak256(abi.encodePacked(a,b))`.
  }

  function test_WhenFirstArgIsBiggerThanSecondArg() external {
    // It should match the result of `keccak256(abi.encodePacked(b,a))`.
  }
}
