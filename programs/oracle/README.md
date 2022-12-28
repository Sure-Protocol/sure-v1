# Sure Prediction market / oracle

The Sure prediction market is meant to answer questions and bring new data on chain by using strong incentivization and stake to ensure the smallest possible attack surface.

For a more detailed presentation please see the medium article [An oracle, a prediction market](https://medium.com/@0xksure/an-oracle-a-prediction-market-84e124db6e55)

# Tests

There are mainly three types of tests in the repo.

### Unit tests

There are usuful unit tests scattered around the code base under src/. These are mainly used for testing calculation and state transition.

### Integration test - rust

The first part of integration tests are found in oracle/tests. These are written in rust and targets the `cargo-bpf`. This allows us to manipulate epochs and time, thus enabling us to run through realistic scenarios.

You can invoke the tests by running

```bash
> cargo test-bpf --test test -- --nocapture
```

You can drop the `-- --nocapture` if you like a less verbose testing env.

### Integration tests - web3.js, DEPRECATED

The old way of running integration tests was to use the web3.js lib. However, since there is not possible to manipulate time it is not possible to test scenarios beyond submitting a vote. Thus, it renders these tests rather useless. Please see the section above on integration tests written in rust inside the project folder.

# TODO

- [x] Add event names to all instructions
- [-] Validate all accounts
- [x] Add Oracle config that controls parameters such as vote time, reveal time, minimum stake, ...
- [-] whether oracle is optimistic or not. Added optimistic parameter that can be used (false by default)
- [x] Don't use name as seed. SOL: use hashed name
- [x] add protocol config to control parameters
- [ ] reduce size of proposal and structure proposal methods
- [ ] update unit tests for config
- [ ] test reward calculation
