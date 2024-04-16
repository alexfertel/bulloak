// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

import {Test} from "forge-std/Test.sol";

contract Foo is Test {
    function test_CantDoX() external {
        vm.skip(true);
        // It can’t do, X.
    }
}
