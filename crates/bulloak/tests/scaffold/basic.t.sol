// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract basic {
    function test_ShouldNeverRevert() external {
        // It should never revert.
    }

    modifier whenFirstArgIsSmallerThanSecondArg() {
        _;
    }

    function test_WhenFirstArgIsSmallerThanSecondArg() external whenFirstArgIsSmallerThanSecondArg {
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
    }

    function test_WhenFirstArgIsZero() external whenFirstArgIsSmallerThanSecondArg {
        // It should do something.
    }

    function test_WhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(b,a))`.
    }
}
