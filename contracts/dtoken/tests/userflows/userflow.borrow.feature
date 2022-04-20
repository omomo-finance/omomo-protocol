Feature: User Borrow flow

Background:
	Given The user A
	And Digital token DWNEAR contract with supplied 10 tokens by user A
	And Digital token DWETH contract with 10 tokens by user A
	And Underlying token contract WBTC with no tokens for user A
	And Setted price for tickers WETH, WNEAR, WBTC has to be the same and equal 1$
	And token = 10^24


Scenario: User A gonna borrow 11 WBTC tokens - postivie flow <Multi collateral check>
	Given User A and DWBTC contract
	When User A borrow 11 WBTC tokens
	Then Success flow expected
	And User balance is 11 WBTC


Scenario: User A gonna borrow 0 WBTC tokens - negative flow <Amount should be positive>
	Given User A and DWBTC contract
	When User A borrow 0 WBTC tokens
	Then Failure flow expected


Scenario: User A gonna borrow 21 WBTC tokens - negative flow <Not enough tokens>
	Given User A and DWBTC contract
	When User A borrow 21 WBTC tokens
	Then Failure flow expected


Scenario: Sequential test after failure - positive flow <Success flow>
	Given User A and DWBTC contract
	When User A borrow on digital token DWBTC 0 tokens, receive failure, after tries to borrow 11 tokens
	Then Success flow expected
	And User balance is 11 WBTC


Scenario: Concurrency test - simultunious borrow on DWBTC contracts - negative flow  <Failure flow due to global action restriction>
	Given User A and DWBTC contract
	When User A simultaniosly make borrow to digital tokens DWETH and DWNEAR by 10 tokens
	Then Failure flow expected on one of calls
	And Expected message  -> "failed to acquire supply action mutex for account {user}"