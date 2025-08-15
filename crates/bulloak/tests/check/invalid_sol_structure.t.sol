// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.0;

contract CancelTest {
    function test_RevertWhen_DelegateCalled() external {
        // It should revert.
    }

    modifier whenNotDelegateCalled() {
        _;
    }

    function test_RevertGiven_TheIdReferencesANullStream() external whenNotDelegateCalled {
        // It should revert.
    }

    modifier givenTheIdDoesNotReferenceANullStream() {
        _;
    }

    function test_RevertGiven_TheStreamsStatusIsDEPLETED()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsCold
    {
        // It should revert.
    }

    function test_RevertGiven_TheStreamsStatusIsSETTLED()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsCold
    {
        // It should revert.
    }

    function test_RevertGiven_TheStreamsStatusIsCANCELED()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsCold
    {
        // It should revert.
    }

    modifier givenTheStreamIsWarm() {
        _;
    }

    modifier whenTheCallerIsAuthorized() {
        _;
    }

    function test_RevertGiven_TheStreamIsNotCancelable()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
    {
        // It should revert.
    }

    function test_GivenTheSenderIsNotAContract()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
    }

    modifier givenTheSenderIsAContract() {
        _;
    }

    function test_GivenTheSenderDoesNotImplementTheHook()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheSenderIsAContract
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should call the sender hook.
        // It should ignore the revert.
    }

    modifier givenTheSenderImplementsTheHook() {
        _;
    }

    function test_WhenThereIsReentrancy()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheSenderIsAContract
        givenTheSenderImplementsTheHook
        whenTheSenderDoesNotRevert
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should call the sender hook.
        // It should ignore the revert.
    }

    function test_WhenTheSenderReverts()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheSenderIsAContract
        givenTheSenderImplementsTheHook
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should call the sender hook.
        // It should ignore the revert.
    }

    function test_WhenThereIsNoReentrancy()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheSenderIsAContract
        givenTheSenderImplementsTheHook
        whenTheSenderDoesNotRevert
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should make the stream not cancelable.
        // It should update the refunded amount.
        // It should refund the sender.
        // It should call the sender hook.
        // It should emit a {MetadataUpdate} event.
        // It should emit a {CancelLockupStream} event.
    }
}
