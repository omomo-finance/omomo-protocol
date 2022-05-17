Feature: User Borrow flow

Background:
	Given The user Bob
	And Underlying token contract WETH with minted 100 tokens for digital token DWETH
	And Underlying token contract WNEAR with minted 100 tokens for digital token DWNEAR
	And Underlying token contract WBTC with minted 100 tokens for digital token DWBTC
	And Digital token DWNEAR contract with supplied 10 tokens by user Bob
	And Digital token DWETH contract with supplied 10 tokens by user Bob
	And Underlying token contract WBTC with no tokens for user Bob
	And Setted price for tickers WETH, WNEAR, WBTC has to be the same and equal 1$
	And token = 10^24


Scenario: User Bob gonna borrow 11 WBTC tokens - postivie flow <Multi collateral check>
	Given User Bob and DWBTC contract
	When User Bob borrow 11 WBTC tokens
	Then Success flow expected
	And User balance is 11 WBTC


Scenario: User Bob gonna borrow 0 WBTC tokens - negative flow <Amount should be positive>
	Given User Bob and DWBTC contract
	When User Bob borrow 0 WBTC tokens
	Then Failure flow expected


Scenario: User Bob gonna borrow 21 WBTC tokens - negative flow <Not enough supplies>
	Given User Bob and DWBTC contract
	When User Bob borrow 21 WBTC tokens
	Then Failure flow expected


Scenario: Sequential test after failure - positive flow <Success flow>
	Given User Bob and DWBTC contract
	When User Bob borrow on digital token DWBTC 0 tokens
	Then Failure flow expected
	When After tries to borrow 11 tokens
	Then Success flow expected
	And User balance is 11 WBTC


Scenario: Concurrency test - simultaneous borrow on DWBTC contracts - negative flow  <Failure flow due to global action restriction>
	Given User Bob and DWBTC contract
	When User Bob simultaniosly make borrow to digital tokens DWETH and DWNEAR by 10 tokens
	Then Failure flow expected on call executed second
	And Failure flow expected message  is "failed to acquire borrow action mutex for account {user}"