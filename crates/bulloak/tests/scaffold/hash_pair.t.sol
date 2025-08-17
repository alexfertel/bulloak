// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract Utils {
    function test_HashPair_ShouldNeverRevert() external {
        // It should never revert.
    }

    function test_HashPairWhen_FirstArgIsSmallerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
    }

    function test_HashPairWhen_FirstArgIsBiggerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(b,a))`.
    }

    function test_Min_ShouldNeverRevert() external {
        // It should never revert.
    }

    function test_MinWhen_FirstArgIsSmallerThanSecondArg() external {
        // It should match the value of `a`.
    }

    function test_MinWhen_FirstArgIsBiggerThanSecondArg() external {
        // It should match the value of `b`.
    }

    function test_Max_ShouldNeverRevert() external {
        // It should never revert.
    }

    function test_MaxWhen_FirstArgIsSmallerThanSecondArg() external {
        // It should match the value of `b`.
    }

    function test_MaxWhen_FirstArgIsBiggerThanSecondArg() external {
        // It should match the value of `a`.
    }
}
