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

  function test_RevertGiven_TheStreamsStatusIsDEPLETED()
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

  function test_RevertGiven_TheStreamsStatusIsCANCELED()
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

  function test_GivenTheSenderIsNotAContract()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsAuthorized
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the sender hook
    // it should ignore the revert
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
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the sender hook
    // it should ignore the revert
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
