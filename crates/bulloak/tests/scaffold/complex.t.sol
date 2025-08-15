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

    modifier givenTheStreamIsCold() {
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

    function test_RevertGiven_TheStreamsStatusIsCANCELED()
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

    modifier givenTheStreamIsWarm() {
        _;
    }

    modifier whenTheCallerIsUnauthorized() {
        _;
    }

    function test_RevertWhen_TheCallerIsAMaliciousThirdParty()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsUnauthorized
    {
        // It should revert.
    }

    function test_RevertWhen_TheCallerIsAnApprovedThirdParty()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsUnauthorized
    {
        // It should revert.
    }

    function test_RevertWhen_TheCallerIsAFormerRecipient()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsUnauthorized
    {
        // It should revert.
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

    modifier givenTheStreamIsCancelable() {
        _;
    }

    function test_GivenTheStreamsStatusIsPENDING()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
    {
        // It should cancel the stream.
        // It should mark the stream as depleted.
        // It should make the stream not cancelable.
    }

    modifier givenTheStreamsStatusIsSTREAMING() {
        _;
    }

    modifier whenTheCallerIsTheSender() {
        _;
    }

    function test_GivenTheRecipientIsNotAContract()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheSender
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
    }

    modifier givenTheRecipientIsAContract() {
        _;
    }

    function test_GivenTheRecipientDoesNotImplementTheHook()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheSender
        givenTheRecipientIsAContract
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should call the recipient hook.
        // It should ignore the revert.
    }

    modifier givenTheRecipientImplementsTheHook() {
        _;
    }

    function test_WhenTheRecipientReverts()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheSender
        givenTheRecipientIsAContract
        givenTheRecipientImplementsTheHook
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should call the recipient hook.
        // It should ignore the revert.
    }

    modifier whenTheRecipientDoesNotRevert() {
        _;
    }

    function test_WhenThereIsReentrancy1()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheSender
        givenTheRecipientIsAContract
        givenTheRecipientImplementsTheHook
        whenTheRecipientDoesNotRevert
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should call the recipient hook.
        // It should ignore the revert.
    }

    function test_WhenThereIsNoReentrancy1()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheSender
        givenTheRecipientIsAContract
        givenTheRecipientImplementsTheHook
        whenTheRecipientDoesNotRevert
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should make the stream not cancelable.
        // It should update the refunded amount.
        // It should refund the sender.
        // It should call the recipient hook.
        // It should emit a {CancelLockupStream} event.
        // It should emit a {MetadataUpdate} event.
    }

    modifier whenTheCallerIsTheRecipient() {
        _;
    }

    function test_GivenTheSenderIsNotAContract()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheRecipient
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
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheRecipient
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

    function test_WhenTheSenderReverts()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheRecipient
        givenTheSenderIsAContract
        givenTheSenderImplementsTheHook
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should call the sender hook.
        // It should ignore the revert.
    }

    modifier whenTheSenderDoesNotRevert() {
        _;
    }

    function test_WhenThereIsReentrancy2()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheRecipient
        givenTheSenderIsAContract
        givenTheSenderImplementsTheHook
        whenTheSenderDoesNotRevert
    {
        // It should cancel the stream.
        // It should mark the stream as canceled.
        // It should call the sender hook.
        // It should ignore the revert.
    }

    function test_WhenThereIsNoReentrancy2()
        external
        whenNotDelegateCalled
        givenTheIdDoesNotReferenceANullStream
        givenTheStreamIsWarm
        whenTheCallerIsAuthorized
        givenTheStreamIsCancelable
        givenTheStreamsStatusIsSTREAMING
        whenTheCallerIsTheRecipient
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
