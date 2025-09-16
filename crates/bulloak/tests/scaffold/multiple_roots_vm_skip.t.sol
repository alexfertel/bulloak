// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

import {Test} from "forge-std/Test.sol";

contract MultipleRootsTreeTest is Test {
    function test_Function1_ShouldNeverRevert() external {
        // It should never revert.
        vm.skip(true);
    }

    function test_Function1_WhenFirstArgIsBiggerThanSecondArg() external {
        // It is all good
        vm.skip(true);
    }

    function test_Function2_RevertWhen_StuffDoesNotHappen() external {
        // it should revert
        vm.skip(true);
    }

    function test_Function2_WhenStuffHappens() external {
        // it should do something simple
        vm.skip(true);
    }
}