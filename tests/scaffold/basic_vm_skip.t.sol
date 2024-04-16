// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

import {Test} from "forge-std/Test.sol";

contract HashPairTestSanitize is Test {
    function test_ShouldNeverRevert() external {
        vm.skip(true);
        // It should never revert.
    }

    modifier whenFirstArgIsSmallerThanSecondArg() {
        _;
    }

    function test_WhenFirstArgIsSmallerThanSecondArg() external whenFirstArgIsSmallerThanSecondArg {
        vm.skip(true);
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
    }

    function test_WhenFirstArgIsZero() external whenFirstArgIsSmallerThanSecondArg {
        vm.skip(true);
        // It should do something.
    }

    function test_WhenFirstArgIsBiggerThanSecondArg() external {
        vm.skip(true);
        // It should match the result of `keccak256(abi.encodePacked(b,a))`.
    }
}
