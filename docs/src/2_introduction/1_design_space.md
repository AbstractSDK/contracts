# The Abstract SDK Design Space

The Abstract SDK design space is a superset of the classical smart-contract design space, meaning that any application built with stand-alone smart-contracts can be implemented with the SDK as well. However, Abstract's design space is unique in that it allows for a level of code re-usability that would not be possible with stand-alone smart-contracts. Additionally the Abstract design space allows for software distribution that is unparalleled in the smart-contract space. With the Abstract SDK you can write your code once, deploy it to any blockchain that supports CosmWasm and let other developers use it within minutes.

Understanding the different approaches to exploiting this design space should be your first step when learning how to use the Abstract SDK. This section will give you a high-level overview of the different approaches and how they can be used to build your application.

## Hosted Applications

Hosted applications are traditionally applications that are built using stand-alone smart-contracts. Examples of these types of applications are dexes, lending markets, yield aggregators, etc. What makes these applications *hosted* is that they are deployed by the maintainers of the application and often require the user to transfer funds to the application's smart-contract in order to use it.

```mermaid

```

## Self-Hosted Applications

Abstract Accounts are the core of the Abstract platform. They are the main building block of any Abstract application. An Abstract Account is a smart-contract that can be installed on any blockchain that supports the CosmWasm smart-contract standard. Abstract Accounts are highly programmable and can be used to build a wide variety of applications.

### Abstract Accounts vs. Smart-Contracts

Abstract Accounts are a superset of smart-contracts. They are designed to be more flexible and more powerful than traditional smart-contracts. This is achieved by abstracting away the underlying blockchain and providing a set of tools that allow you to interact with the blockchain in a more flexible way.
