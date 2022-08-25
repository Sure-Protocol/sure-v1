#!/usr/bin/env bash

PROGRAM=$1
mkdir -p temp/idl/

anchor idl parse --file programs/$PROGRAM/src/lib.rs > ./temp/idl/$PROGRAM.json || {
    echo "could not generate IDL"
    exit 1
}
