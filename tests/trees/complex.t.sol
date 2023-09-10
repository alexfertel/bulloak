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

  function test_RevertGiven_TheStreamsStatusIsDEPLETED()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsCold
    givenTheStreamsStatusIsDEPLETED
  {
    // it should revert
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

  modifier givenTheStreamIsWarm() {
    _;
  }

  modifier whenTheCallerIsUnauthorized() {
    _;
  }

  modifier whenTheCallerIsAMaliciousThirdParty() {
    _;
  }

  function test_RevertWhen_TheCallerIsAMaliciousThirdParty()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsUnauthorized
    whenTheCallerIsAMaliciousThirdParty
  {
    // it should revert
  }

  modifier whenTheCallerIsAnApprovedThirdParty() {
    _;
  }

  function test_RevertWhen_TheCallerIsAnApprovedThirdParty()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsUnauthorized
    whenTheCallerIsAnApprovedThirdParty
  {
    // it should revert
  }

  modifier whenTheCallerIsAFormerRecipient() {
    _;
  }

  function test_RevertWhen_TheCallerIsAFormerRecipient()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsUnauthorized
    whenTheCallerIsAFormerRecipient
  {
    // it should revert
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

  modifier givenTheStreamIsCancelable() {
    _;
  }

  modifier givenTheStreamsStatusIsPENDING() {
    _;
  }

  function test_GivenTheStreamsStatusIsPENDING()
    external
    whenNotDelegateCalled
    givenTheIdDoesNotReferenceANullStream
    givenTheStreamIsWarm
    whenTheCallerIsAuthorized
    givenTheStreamIsCancelable
    givenTheStreamsStatusIsPENDING
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

  modifier givenTheRecipientIsNotAContract() {
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
    givenTheRecipientIsNotAContract
  {
    // it should cancel the stream
    // it should mark the stream as canceled
  }

  modifier givenTheRecipientIsAContract() {
    _;
  }

  modifier givenTheRecipientDoesNotImplementTheHook() {
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
    givenTheRecipientDoesNotImplementTheHook
  {
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the recipient hook
    // it should ignore the revert
  }

  modifier givenTheRecipientImplementsTheHook() {
    _;
  }

  modifier whenTheRecipientReverts() {
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
    whenTheRecipientReverts
  {
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the recipient hook
    // it should ignore the revert
  }

  modifier whenTheRecipientDoesNotRevert() {
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
    givenTheStreamIsCancelable
    givenTheStreamsStatusIsSTREAMING
    whenTheCallerIsTheSender
    givenTheRecipientIsAContract
    givenTheRecipientImplementsTheHook
    whenTheRecipientDoesNotRevert
    whenThereIsReentrancy
  {
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the recipient hook
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
    givenTheStreamIsCancelable
    givenTheStreamsStatusIsSTREAMING
    whenTheCallerIsTheSender
    givenTheRecipientIsAContract
    givenTheRecipientImplementsTheHook
    whenTheRecipientDoesNotRevert
    whenThereIsNoReentrancy
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

  modifier givenTheSenderIsNotAContract() {
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
    givenTheSenderDoesNotImplementTheHook
  {
    // it should cancel the stream
    // it should mark the stream as canceled
    // it should call the sender hook
    // it should ignore the revert
  }

  modifier givenTheSenderImplementsTheHook() {
    _;
  }

  modifier whenTheSenderReverts() {
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
    givenTheStreamIsCancelable
    givenTheStreamsStatusIsSTREAMING
    whenTheCallerIsTheRecipient
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
    givenTheStreamIsCancelable
    givenTheStreamsStatusIsSTREAMING
    whenTheCallerIsTheRecipient
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
