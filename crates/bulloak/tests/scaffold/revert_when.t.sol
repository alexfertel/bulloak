// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract revert_when {
    modifier whenStuffIsCalled() {
        _;
    }

    function test_RevertWhen_AConditionIsMet() external whenStuffIsCalled {
        // It should revert.
        //     Because we shouldn't allow it.
    }
}
