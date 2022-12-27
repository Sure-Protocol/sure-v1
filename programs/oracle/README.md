# Sure Prediction market / oracle

The Sure prediction market is meant to answer questions and bring new data on chain by using strong incentivization and stake to ensure the smallest possible attack surface.

For a more detailed presentation please see the medium article [An oracle, a prediction market](https://medium.com/@0xksure/an-oracle-a-prediction-market-84e124db6e55)

### TODO

- [x] Add event names to all instructions
- [-] Validate all accounts
- [x] Add Oracle config that controls parameters such as vote time, reveal time, minimum stake, ...
- [-] whether oracle is optimistic or not. Added optimistic parameter that can be used (false by default)
- [x] Don't use name as seed. SOL: use hashed name
- [x] add protocol config to control parameters
- [ ] reduce size of proposal and structure proposal methods
- [ ] update unit tests for config
- [ ] test reward calculation
