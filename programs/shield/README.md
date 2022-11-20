# Sure Shield

aka sure v2

## about

shield is an insurance protocol powered by serum. It allows users to buy coverage and both sides to speculate on the risk of protocols on solana.

## proto

- Lps can provide liquidity at a premium range
- users can buy from the cheapest premium ranges
- market makers can place ask and bids at premiums they think reflect the risk of the underlying protocol

### Liquidity Provider flow - Providing Coverage
The positions are represented as coverage perps USDC-cProt


1. If a lp provides 1000USDC at 300-400bp then 1000usdc will be locked up at [300,310,...,400] if the tick size is 10bp. 
2. The LP will receive 1000*0.35 = 35 cProt
3. A user might want to buy insurance for $100 and will hit 300bp and pay $3 for  

4. If parts of the range is used then a premium will be locked up 
The 
 and the lp will receive a premium if the position is used. when a lp wants to exit the position the lp will get back the 1000usdc and will have to pay a premium to close the position.
if the premium has increased the lp will have to pay a higher premium since the position is hedged

## TODO

[ x ] create initialize pool instruction

[ x ] create provide_coverage instruction - allows LPs to post liquidity

[ ] create update_coverage instruction - allows LP to adjust their position

[ ] create buy_policy instruction - allow users to buy insurance from OB

[ ] create update_policy instruction - allows users to change their position

[ ] coverage market dimensions (smart contract,duration)

[ ] write unit test for coverage

[ ] write unit test for pool

[ ] (coverage perps)
