# AMM (Automated Marker Maker)



## Concepts

**Liquidity**:

**Liquidity pool**:

**Slipage**: When price change at the moment user wants to trade or provide liquidity,
the tx is not instant so there's a difference in the amount of lp/x/y tokens received;
and with this value the amm sets boundaries and user don't loss too much.

**Constant Product Curve**:

## Functions

- Init
- Deposit
- Swap
- Withdraw

## Macros

- 

## Important notes

In the config account when init
> In the seeds we can add mint for 'x' or 'y' because they're unique for the pool.
> This allows to avoid the need of 'has_one' for the seed added

> If we add 'initializer' acc then we don't need the 'authority' in the state, so won't be able to be optional

Fees as *u16*
> When using **business basis points** they can only go up to 10,000; so there's no need to
> use a large number to store large data type that stores the values.

> This allows to have up to 2 decimals for precision, and is enough for storing 10k

In swap
> User has to have one of the tokens (x's ATA, or y) but not the other already (y's ATA. or x), so
> 'init_if_needed' is used

Burn lp Tokens
> To keep the price of the lp tokens consistent with the amount in the pool
