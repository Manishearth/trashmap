## elsa

[![Build Status](https://travis-ci.org/Manishearth/trashmap.svg?branch=master)](https://travis-ci.org/Manishearth/trashmap)
[![Current Version](https://meritbadge.herokuapp.com/trashmap)](https://crates.io/crates/trashmap)
[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/trashmap.svg)](#license)

This crate provides `TrashMap` and `TrashSet` types, which allow you to directly use the key hash to operate with your entries. This is typically useful for when it's cheap to hold on to the hash value (e.g. within a single stack frame) and you don't want to incur the cost of rehashing on each access (but you can't use `Entry` as the map may change in the process)

The  `Trash` type is used to represent computed hashes, lookups via `Trash` are cheap.
