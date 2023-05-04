# Abstract IBC host

The Abstract IBC host is a contract designed to be deployed on an Abstract-deployed chain. It enables any Account to perform cross-chain actions on an account owned by the Account.

## Supported actions

### Register

Register an Account by creating a local account. Incoming requests and funds will be routed to this account.

### Dispatch

Proxy a set of execute messages to the Account's proxy on the host chain.