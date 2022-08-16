Feature: User Repay flow

Background:
	Given The user Bob
	And Underlying token contract WETH with minted 100 tokens for digital token weth_market
	And Underlying token contract WNEAR with minted 100 tokens for digital token wnear_market
	And Underlying token contract WBTC with minted 100 tokens for digital token DWBTC
	And Underlying token contract WNEAR with 100 tokens for user Bob
	And Underlying token contract WETH with 100 tokens for user Bob
	And Underlying token contract WBTC with 100 tokens for user Bob
	And Digital token wnear_market contract with supplied 70 tokens by user Bob
	And Digital token weth_market contract with supplied 60 tokens by user Bob
	And Digital token DWBTC contract with supplied 0 tokens by user Bob
	And Digital token wnear_market contract with borrows of 40 tokens by user Bob
	And Digital token weth_market contract with borrows of 30 tokens by user Bob
	And Digital token DWBTC contract with borrows of 0 tokens by user Bob
	And Exchange_rate for contracts equal 1 
	And Contracts accrued interests should be equal 0
	
	And token = 10^24

Scenario: User Bob repay for wnear_market borrow - positive flow  <Success flow>
	Given The User Bob and wnear_market contract
	Then After the User Bob receives repay_value (40) from view_repay_info method by wnear_market borrow, he makes repay
	Then Success flow expected
	And User balance is 30 on WNEAR contract

Scenario: User Bob repay less than expected for wnear_market borrow - negative flow <Not enough tokens>
	Given The User Bob and wnear_market contract
	When The User Bob makes repay for wnear_market borrow 10 WNEAR tokens
	Then Failure flow expected - <repay amount 10 is less than actual debt 70>

Scenario: User Bob repay to digital token wnear_market 0 tokens - negative flow  <Amount should be positive>
	Given User Bob and wnear_market contract
	When User Bob repay to digital token wnear_market 0 tokens
	Then Failure flow expected	

Scenario: User Bob repay 100 WBTC to digital token DWBTC with no borrow - positive flow  <Success flow>
	Given User Bob and DWBTC contract
	When User Bob repay to digital token DWBTC 100 tokens
	Then Success flow expected
	And User balance is 100 WBTC - <No funds have been debited>

Scenario: Sequential test after failure - positive flow <Success flow>
	Given User Bob and weth_market, wnear_market contracts
	When User Bob repay to digital token weth_market 0 tokens, receive failure, after repay to digital token wnear_market 50 WNEAR
	Then Success flow expected
	And User balance is 50 WNEAR
	And Borrows for User Bob on wnear_market contract is 0

Scenario: Concurrency test - simultaneous repay on DWBTC contracts - negative flow  <Failure flow due to global action restriction>
	Given User Bob and weth_market,wnear_market contract
	When User Bob simultaniosly make repay to digital tokens weth_market and wnear_market by 10 tokens
	Then Failure flow expected on call executed second
	And Failure flow expected message  is "failed to acquire repay action mutex for account {user}"