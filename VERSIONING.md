# Abstract and related versioning

This document describes the current versioning system for Abstract's various libraries and components. It is intended to be a living document that is updated as the project evolves.

## Contracts

1. Upgrade the versions in the [`contracts`](https://github.com/Abstract-OS/contracts) repository to the new version via global find + replace (but be careful).
2. Run just publish to publish the packages to the new version.
3. Run just schema

## Modules

1. Upgrade the Abstract packages in the [`apis`](https://github.com/Abstract-OS/apis) repository to the new version in the base Cargo.toml
2. Run just publish
3. Upgrade the Abstract packages in the [`apps`](https://github.com/Abstract-OS/apps) repository to the new version in the base Cargo.toml
4. Run just publish

## Abstract.js

1. Run the ts-codegen script