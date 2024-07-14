// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract Utils {
    function test_HashPairShouldNeverRevert() external {
        // It should never revert.
    }

    function test_HashPairWhenFirstArgIsSmallerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
    }

    function test_HashPairWhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(b,a))`.
    }

    function test_MinShouldNeverRevert() external {
        // It should never revert.
    }

    function test_MinWhenFirstArgIsSmallerThanSecondArg() external {
        // It should match the value of `a`.
    }

    function test_MinWhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the value of `b`.
    }

    function test_MaxShouldNeverRevert() external {
        // It should never revert.
    }

    function test_MaxWhenFirstArgIsSmallerThanSecondArg() external {
        // It should match the value of `b`.
    }

    function test_MaxWhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the value of `a`.
    }
}
