pragma solidity 0.8.0;

contract CancelTest {
  modifier whenDelegateCalled() {
    _;
  }

  function test_RevertWhen_DelegateCalled()
    external
    whenDelegateCalled
  {
    // it should revert
  }

  modifier whenNotDelegateCalled() {
    _;
  }

  modifier givenTheIdReferencesANullStream() {
    _;
  }

  function test_RevertGiven_TheIdReferencesANullStream()
    external
    whenNotDelegateCalled
    givenTheIdReferencesANullStream
  {
    // it should revert
  }

  modifier givenTheIdDoesNotReferenceANullStream() {
    _;
  }

  modifier givenTheStreamIsCold() {
    _;
  }

  modifier givenTheStreamsStatusIsDEPLETED() {
    _;
  }

  modifier givenTheStreamsStatusIsCANCELED() {
    _;
  }

  function test_RevertGiven_TheStreamsStatusIsCANCELED()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsCold
    givenTheStreamsStatusIsCANCELED
  {
    // it should revert
  }

  modifier givenTheStreamsStatusIsSETTLED() {
    _;
  }

  function test_RevertGiven_TheStreamsStatusIsSETTLED()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsCold
    givenTheStreamsStatusIsSETTLED
  {
    // it should revert
  }

  function test_RevertGiven_TheStreamsStatusIsDEPLETED()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsCold
    givenTheStreamsStatusIsDEPLETED
  {
    // it should revert
  }

  modifier givenTheStreamIsWarm() {
    _;
  }

  modifier whenTheCallerIsAuthorized() {
    _;
  }

  modifier givenTheStreamIsNotCancelable() {
    _;
  }

  function test_RevertGiven_TheStreamIsNotCancelable()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsAuthorized
    givenTheStreamIsNotCancelable
  {
    // it should revert
  }

  modifier givenTheSenderIsNotAContract() {
    _;
  }

  function test_GivenTheSenderIsNotAContract()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsAuthorized
    givenTheSenderIsNotAContract
  {
    // it should cancel the stream
    // it should mark the stream as canceled
  }

  modifier givenTheSenderIsAContract() {
    _;
  }

  modifier givenTheSenderDoesNotImplementTheHook() {
    _;
  }


  modifier whenTheSenderReverts() {
    _;
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
    givenTheSenderIsAContract
    givenTheSenderImplementsTheHook
    whenTheSenderReverts
  {
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the sender hook
    // it should ignore the revert
  }

  modifier whenTheSenderDoesNotRevert() {
    _;
  }

  modifier whenThereIsReentrancy() {
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
    whenThereIsReentrancy
  {
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the sender hook
    // it should ignore the revert
  }

  modifier whenThereIsNoReentrancy() {
    _;
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
    whenThereIsNoReentrancy
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
