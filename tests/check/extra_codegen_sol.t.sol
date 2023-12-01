// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract HashPairTest {
  function test_ShouldNeverRevert() external {
    // It should never revert.
  }

  modifier thisisAnExtraModifier() {
    // It has a random comment inside.
    _;
  }

  modifier whenFirstArgIsSmallerThanSecondArg() {
    _;
  }

  function thisIsAnExtraFunction() {
    // It has a random comment inside.
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

  function thisIsAnotherExtraFunction() {
    // It has a random comment inside.
  }

  function test_WhenFirstArgIsBiggerThanSecondArg()
    external
    whenFirstArgIsBiggerThanSecondArg
  {
    // It should match the result of `keccak256(abi.encodePacked(b,a))`.
  }
}
