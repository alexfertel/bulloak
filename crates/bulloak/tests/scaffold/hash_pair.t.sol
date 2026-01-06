// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract hash_pair {
    function test_HashPair_ShouldNeverRevert() external {
        // It should never revert.
    }

    function test_HashPair_WhenFirstArgIsSmallerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
    }

    function test_HashPair_WhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(b,a))`.
    }

    function test_Min_ShouldNeverRevert() external {
        // It should never revert.
    }

    function test_Min_WhenFirstArgIsSmallerThanSecondArg() external {
        // It should match the value of `a`.
    }

    function test_Min_WhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the value of `b`.
    }

    function test_Max_ShouldNeverRevert() external {
        // It should never revert.
    }

    function test_Max_WhenFirstArgIsSmallerThanSecondArg() external {
        // It should match the value of `b`.
    }

    function test_Max_WhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the value of `a`.
    }
}
