#!/bin/sh

set -e # Make script exit when a command fail.
set -u # Exit on usage of undeclared variable.
# set -x # Trace what gets executed.
set -o pipefail # Catch failures in pipes.

# Building elm-test-rs
cargo build --release
export PATH="$(pwd)/target/release:$PATH"

# Checking passing tests
for example_dir in tests-example-projects/passing/*
do
  echo "checking ${example_dir}"
  elm-test-rs --project ${example_dir} > /dev/null 2>&1
done

# Checking erroring tests
set +e
for example_dir in tests-example-projects/erroring/*
do
  echo "checking ${example_dir}"
  elm-test-rs --project ${example_dir} > /dev/null 2>&1
  exit_status=$?
  if [ $exit_status -ne 1 ]; then
    echo "${example_dir} did not errored as it should have!"
    echo "Exit status: ${exit_status}"
    exit 1
  fi
done

# Checking failing tests
set +e
for example_dir in tests-example-projects/failing/*
do
  echo "checking ${example_dir}"
  elm-test-rs --project ${example_dir} > /dev/null 2>&1
  exit_status=$?
  if [ $exit_status -ne 2 ]; then
    echo "${example_dir} did not fail as it should have!"
    echo "Exit status: ${exit_status}"
    exit 1
  fi
done
