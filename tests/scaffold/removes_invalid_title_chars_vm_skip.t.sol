// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract Foo {
    function test_CantDoX() external {
        vm.skip(true);
        // It can’t do, X.
    }
}
