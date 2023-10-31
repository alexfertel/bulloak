// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.0;

contract CancelTest {
  function test_RevertWhen_DelegateCalled() external {
    // it should revert
  }

  modifier whenNotDelegateCalled() {
    _;
  }

  function test_RevertGiven_TheIdReferencesANullStream()
    external
    whenNotDelegateCalled
  {
    // it should revert
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
    // it should revert
  }

  function test_RevertGiven_TheStreamsStatusIsCANCELED()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsCold
  {
    // it should revert
  }

  function test_RevertGiven_TheStreamsStatusIsSETTLED()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsCold
  {
    // it should revert
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
    // it should revert
  }

  function test_RevertWhen_TheCallerIsAnApprovedThirdParty()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsUnauthorized
  {
    // it should revert
  }

  function test_RevertWhen_TheCallerIsAFormerRecipient()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsUnauthorized
  {
    // it should revert
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
    // it should revert
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
    // it should cancel the stream
    // it should mark the stream as depleted
    // it should make the stream not cancelable
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
    // it should cancel the stream
    // it should mark the stream as canceled
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the recipient hook
    // it should ignore the revert
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the recipient hook
    // it should ignore the revert
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the recipient hook
    // it should ignore the revert
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should make the stream not cancelable
    // it should update the refunded amount
    // it should refund the sender
    // it should call the recipient hook
    // it should emit a {CancelLockupStream} event
    // it should emit a {MetadataUpdate} event
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
    // it should cancel the stream
    // it should mark the stream as canceled
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the sender hook
    // it should ignore the revert
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the sender hook
    // it should ignore the revert
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the sender hook
    // it should ignore the revert
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should make the stream not cancelable
    // it should update the refunded amount
    // it should refund the sender
    // it should call the sender hook
    // it should emit a {MetadataUpdate} event
    // it should emit a {CancelLockupStream} event
  }
}