Feature: User Supply flow

Background:
	Given The user Bob
	And Underlying token contract WETH with minted 100 tokens for digital token weth_market
	And Underlying token contract WNEAR with minted 100 tokens for digital token wnear_market
	And Underlying token contract WETH with minted 100 tokens for user Bob
	And Underlying token contract WNEAR with 50 tokens for user Bob
	And Digital token weth_market contract
	And Digital token wnear_market contract
	And token = 10^24

Scenario: User Bob supplies to digital token weth_market 100 tokens - positive flow <Success flow>
	Given User Bob and weth_market contract
	When User Bob supplies to digital token weth_market 100 tokens
	Then Success flow expected
	And User balance is 100 weth_market
	And User balance is 0 WETH

Scenario: User Bob supplies to digital token wnear_market 100 tokens - negative flow <Not enough tokens>
	Given User Bob and wnear_market contract
	When User Bob supplies to digital token wnear_market 100 tokens
	Then Failure flow expected

Scenario: User Bob supplies to digital token wnear_market 0 tokens - negative flow  <Amount should be positive>
	Given User Bob and wnear_market contract
	When User Bob supplies to digital token wnear_market 0 tokens
	Then Failure flow expected	

Scenario:User Bob supplies to digital token wnear_market 100 tokens - negative flow <Digital token with no balance>
	Given User Bob and wnear_market contract
	When User Bob supplies to digital token wnear_market  100 tokens
	Then Failure flow expected <Cannot calculate utilization rate as denominator is equal 0>

Scenario: Sequential test after failure - positive flow <Success flow>
	Given User Bob and weth_market, wnear_market contracts
	When User Bob supplies to digital token wnear_market 0 tokens, receive failure, after supplies to digital token weth_market 100 tokens
	Then Success flow expected
	And User balance is 100 weth_market
	And User balance is 0 WETH

Scenario: Concurrency test - simultaneous deposit to weth_market and wnear_market contracts - negative flow  <Failure flow due to global action restriction>
	Given User Bob and weth_market, wnear_market contracts
	When User Bob simultaniosly make supply to digital tokens weth_market and wnear_market by 10 tokens
	Then Failure flow expected on call executed second
	And Failure flow expected message  is "failed to acquire supply action mutex for account {user}"



