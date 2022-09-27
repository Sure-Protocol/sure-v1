# Sure Shield

aka sure v2

## about

shield is an insurance protocol powered by serum. It allows users to buy coverage and both sides to speculate on the risk of protocols on solana.

## proto

- Lps can provide liquidity at a premium range
- users can buy from the cheapest premium ranges
- market makers can place ask and bids at premiums they think reflect the risk of the underlying protocol

### lps

if a lp provides 1000USDC at 300-400bp then 1000usdc will be locked up and the lp will receive a premium if the position is used. when a lp wants to exit the position the lp will get back the 1000usdc and will have to pay a premium to close the position.
if the premium has increased the lp will have to pay a higher premium since the position is hedged

## TODO

[ ] create initialize pool instruction
[ ] create provide_coverage instruction - allows LPs to post liquidity  
[ ] create update_coverage instruction - allows LP to adjust their position
[ ] coverage perps
