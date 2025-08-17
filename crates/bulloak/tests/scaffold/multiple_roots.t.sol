// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract MultipleRootsTreeTest {
    function test_Function1_ShouldNeverRevert() external {
        // It should never revert.
    }

    function test_Function1When_FirstArgIsBiggerThanSecondArg() external {
        // It is all good
    }

    function test_Function2RevertWhen_StuffDoesNotHappen() external {
        // it should revert
    }

    function test_Function2When_StuffHappens() external {
        // it should do something simple
    }
}