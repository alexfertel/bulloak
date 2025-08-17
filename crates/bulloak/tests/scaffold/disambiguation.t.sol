// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract Disambiguation {
    modifier whenAIsEven() {
        _;
    }

    function test_RevertGiven_Zero() external whenAIsEven {
        // it should revert
    }

    function test_GivenNotZero() external whenAIsEven {
        // it should work
    }

    modifier whenBIsEven() {
        _;
    }

    function test_RevertGiven_ZeroWhenBIsEven() external whenBIsEven {
        // it should revert
    }

    function test_GivenNotZero_WhenBIsEven() external whenBIsEven {
        // it should work
    }
}
