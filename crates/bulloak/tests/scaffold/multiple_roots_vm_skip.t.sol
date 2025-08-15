// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

import { Test } from "forge-std/src/Test.sol";

contract MultipleRootsTreeTest is Test {
    function test_Function1ShouldNeverRevert() external {
        // It should never revert.
        vm.skip(true);
    }

    function test_Function1WhenFirstArgIsBiggerThanSecondArg() external {
        // It is all good.
        vm.skip(true);
    }

    function test_Function2WhenStuffHappens() external {
        // It should do something simple.
        vm.skip(true);
    }
}
