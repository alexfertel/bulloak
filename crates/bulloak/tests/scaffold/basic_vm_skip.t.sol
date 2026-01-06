// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

import {Test} from "forge-std/Test.sol";

contract basic is Test {
    function test_ShouldNeverRevert() external {
        // It should never revert.
        vm.skip(true);
    }

    modifier whenFirstArgIsSmallerThanSecondArg() {
        _;
    }

    function test_WhenFirstArgIsSmallerThanSecondArg() external whenFirstArgIsSmallerThanSecondArg {
        // It should match the result of `keccak256(abi.encodePacked(a,b))`.
        vm.skip(true);
    }

    function test_WhenFirstArgIsZero() external whenFirstArgIsSmallerThanSecondArg {
        // It should do something.
        vm.skip(true);
    }

    function test_WhenFirstArgIsBiggerThanSecondArg() external {
        // It should match the result of `keccak256(abi.encodePacked(b,a))`.
        vm.skip(true);
    }
}
