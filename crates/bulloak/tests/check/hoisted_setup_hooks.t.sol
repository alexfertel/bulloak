// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract hoisted_setup_hooks {
    modifier whenB() {
        _;
    }

    function test_Foo_WhenB() external whenB {
        // It should work.
    }

    function test_Foo_WhenC() external whenB {
        // It should also work.
    }

    function test_Bar_WhenB() external whenB {
        // It should produce a special side effect
    }

    function test_Bar_RevertWhen_C() external whenB {
        // It should revert.
    }
}
