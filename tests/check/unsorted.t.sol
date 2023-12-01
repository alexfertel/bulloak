// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract HashPairTest {
    function test_ShouldNeverRevert() external {
        // It should never revert.
    }

    function an_extra_function() external {
        // That may have some content.
    }

    function test_WhenFirstArgIsSmallerThanSecondArg() external whenFirstArgIsSmallerThanSecondArg {
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
    }

    // A comment.
    function test_WhenFirstArgIsZero() external whenFirstArgIsSmallerThanSecondArg {
        // It should do something.
    }

    modifier whenFirstArgIsSmallerThanSecondArg() {
        _;
    }

    function test_WhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(b,a))`.
    }
}
