---
description: Welcome to the OMOMO guide documentation site.
cover: .gitbook/assets/header (1).jpg
coverY: 0
---

# Overview

## What is OMOMO?

OMOMO is a money market protocol on the NEAR blockchain.
Core features: 
- lending/borrowing pools, 
- leverage trading with covering positions from the borrowing pool.

[![Demo](https://img.youtube.com/vi/4ryH4u7hKzk/maxresdefault.jpg)](https://www.youtube.com/watch?v=4ryH4u7hKzk)

## Protocol benefits

- Permissionless listing on the money market
- All operations executed on-chain and managed by smart contracts 
- No fees on the DEX - (got it back as a liquidity provider)
- No slippage while swapping on the DEX
- DAO-governed asset tiers and protocol parameters

## What makes OMOMO different?

OMOMO aims to bring several DeFi2.0 usecases into the existing NEAR ecosystem:

* use all liquidity - no isolated pairs, the whole liquidity participates in the collaterisation
* modularity - the protocol provides building blocks which can and should be included into DeFi strategies built upon opend markets
* all liquidity should work - provide active management of the locked liquidity and leave the control for the user
* support customization at the higher level - allow custom markets creation and custom interest rate models and allow users to utilize this functionality

## Core contracts

The protocol has gone through several development iterations in order to correspond to the Near requirements for smart-contracts development. Thus, the architecture of the protocol now relies on the minimal number of independant contracts:
* Controller as an entry point for the user and the central entity of the protocol
* Market contracs which represent markets and provide the token ready to be included into DeFi strategies
* Oracles connector