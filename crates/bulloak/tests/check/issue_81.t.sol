// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.22;

import { Integration_Test } from "./Integration.t.sol";

contract RecentAmountOf_Integration_Concrete_Test is Integration_Test {
    function test_RevertGiven_Null() external {
        bytes memory callData = abi.encodeCall(flow.recentAmountOf, nullStreamId);
        expectRevert_Null(callData);
    }

    function test_GivenPaused() external givenNotNull {
        flow.pause(defaultStreamId);

        // It should return zero.
        uint128 recentAmount = flow.recentAmountOf(defaultStreamId);
        assertEq(recentAmount, 0, "recent amount");
    }

    function test_WhenLastUpdatedTimeInPresent() external givenNotNull givenNotPaused {
        // Update the last time to the current block timestamp.
        updateLastTimeToBlockTimestamp(defaultStreamId);

        // It should return zero.
        uint128 recentAmount = flow.recentAmountOf(defaultStreamId);
        assertEq(recentAmount, 0, "recent amount");
    }

    function test_BlahBlahBlah() external view givenNotNull givenNotPaused {
        // It should return the correct recent amount.
        uint128 recentAmount = flow.recentAmountOf(defaultStreamId);
        assertEq(recentAmount, ONE_MONTH_STREAMED_AMOUNT, "recent amount");
    }
}
