// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

import { Test } from "forge-std/src/Test.sol";

contract Foo is Test {
    function test_CantDoX() external {
        // It can’t do, X.
        vm.skip(true);
    }
}
