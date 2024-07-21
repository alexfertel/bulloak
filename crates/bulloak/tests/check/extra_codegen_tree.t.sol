// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract HashPairTest {
  modifier whenFirstArgIsSmallerThanSecondArg() {
    _;
  }

  function test_WhenFirstArgIsSmallerThanSecondArg()
    external
    whenFirstArgIsSmallerThanSecondArg
  {
    // It should match the result of `keccak256(abi.encodePacked(a,b))`.
  }

  modifier whenFirstArgIsBiggerThanSecondArg() {
    _;
  }

  function thisIsAnotherExtraFunction() external {
    // It has a random comment inside.
  }

  function test_WhenFirstArgIsBiggerThanSecondArg()
    external
    whenFirstArgIsBiggerThanSecondArg
  {
    // It should match the result of `keccak256(abi.encodePacked(b,a))`.
  }
}
