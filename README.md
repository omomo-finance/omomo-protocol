---
description: Welcome to the Nearlend guide documentation site.
cover: .gitbook/assets/mountains_bg.svg
coverY: 0
---

# Overview

## What is Nearlend?

NEARLEND is non-custodial lending and borrowing liquidity protocol on the NEAR blockchain.<br>
The idea of the protocol was firstly implemented on the [Near Hackathon]() on November 2021. The team presented the initial solution of the decentralized non-custodial lending protocol with multi-collaterallized loans.<br>
The protocol is aimed to resolve several problems:
<li>provide non-custodial loans service
<li>provide support of multi-collaterization of loans for the Near ecosystem - more flexible mechanism than isolated pairs
<li>provide flexible liquidation mechanism
<li>provide reliable price feeds update for correct collaterization calculation
<li>provide moduled structure for the protocol to support DeFi strategies building upon existing markets
<li>provide support for the interaction with 3rd-party protocol within complex DeFi strategies
<li>provide foundation for the support of fixed rate loans
<br><br>
The main goal of the Nearland is to correspond requirements of DeFi2.0 protocol:
<li>become modular
<li>have active management of the liqudity (by user and by the platform in the decentralized way)
<li>have liquidity actively working
<li>follow up automatization road for most of operations
<li>become trully decentralized 
<li>allow user-based operations, including the registration of the user based markets and including them into the collaterization calculations
<br><br>

## What makes Nearland different?
NEARLAND aims to bring several DeFi2.0 usecases into the existing NEAR ecosystem:
* use all liquidity - no isolated pairs, the whole liquidity participates in the collaterisation
* modularity - the protocol provides building blocks which can and should be included into DeFi strategies built upon opend markets
* all liquidity should work - provide active management of the locked liquidity and leave the control for the user
* support customization at the higher level - allow custom markets creation and custom interest rate models and allow users to utilize this functionality

## Core contracts
The protocol has gone through several development iterations in order to correspond to the Near requirements for smart-contracts development. Thus, the architecture of the protocol now relies on the minimal number of independant contracts:
<li>Controller as an entry point for the user and the central entity of the protocol
<li>DToken contracs which represent markets and provide the token ready to be included into DeFi strategies
<li>Oracles connector
<br>
Check further documentation to get more info
![Overal look](.gitbook/assets/general.png)
