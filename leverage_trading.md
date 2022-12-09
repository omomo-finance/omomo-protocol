# Leverage Trading
OMOMO is a decentralized trading toolset over DeFi with unique characteristics.
Core functionality is decentralized Leveraged Trading that we are building with Omomo lending and Ref.finance DEX.
Key component is limit orders on top of concentrated liquidity of Ref V2 enabling 0 swap fees on the DEX.

![](https://i.imgur.com/MfduTp5.jpg)


## How Leverage Trading works in tandem with OMOMO lending:
Trader creates x5 Long position by depositing 1k USDT
OMOMO protocol borrows the rest from its lending LP and creates it's limit order

When order executed, Trader has on-chain leveraged position and looking for the moment to get the profits

![](https://i.imgur.com/H7mrHKv.jpg)


## How this limit order works:

Omomo protocol puts the liquidity to the tiny range in the pool where Asset A is 0 and all amount is in Asset B
When price on the DEX reaches target price - liquidity from Asset B moves to Asset A
Then, omOmo protocol gets liquidity back to the user. 
Order completed

![](https://i.imgur.com/Y9RDiYJ.jpg)