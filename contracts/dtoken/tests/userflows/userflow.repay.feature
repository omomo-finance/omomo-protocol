Feature: User Repay flow

Background:
	Given The user A
	And Underlying token contract WNEAR with 100 tokens for user A
	And Underlying token contract WETH with 100 tokens for user A
	And Underlying token contract WBTC with 100 tokens for user A
	And Digital token DWNEAR contract with supplied 70 tokens by user A
	And Digital token DWETH contract with supplied 60 tokens by user A
	And Digital token DWBTC contract with supplied 0 tokens by user A
	And Digital token DWNEAR contract with borrows of 50 tokens by user A
	And Digital token DWETH contract with borrows of 40 tokens by user A
	And Digital token DWBTC contract with borrows of 0 tokens by user A
	And Exchange_rate for contracts equal 1 
	And Contracts accrued interests should be equal 0
	
	And token = 10^24

Scenario: User A repay for DWNEAR borrow - positive flow  <Success flow>
	Given The user A and DWNEAR contract
	Then After the user A receives repay_value (50) from view_repay_info method by DWNEAR borrow, he makes repay
	Then Success flow expected
	And User balance is 50 on WNEAR contract


Scenario: User A repay less than expected for DWNEAR borrow - negative flow <Not enough tokens>
	Given The user A and DWNEAR contract
	When The user A makes repay for DWNEAR borrow 10 WNEAR tokens
	Then Failure flow expected - <repay amount 10 is less than actual debt 70>

Scenario: User A repay to digital token DWNEAR 0 tokens - negative flow  <Amount should be positive>
	Given User A and DWNEAR contract
	When User A repay to digital token DWNEAR 0 tokens
	Then Failure flow expected	

Scenario: User A repay 100 WETH to digital token DWBTC with no borrow - positive flow  <Success flow>
	Given User A and DWBTC contract
	When User A repay to digital token DWBTC 0 tokens
	Then Success flow expected
	And User balance is 100 WBTC - <No funds have been debited>

Scenario: Sequential test after failure - positive flow <Success flow>
	Given User A and DWETH, DWNEAR contracts
	When User A repay to digital token DWETH 0 tokens, receive failure, after repay to digital token DWNEAR 50 WNEAR
	Then Success flow expected
	And User balance is 50 WNEAR
	And Borrows for User A on DWNEAR contract is 0

Scenario: Concurrency test - simultunious repay on DWBTC contracts - negative flow  <Failure flow due to global action restriction>
	Given User A and DWETH,DWNEAR contract
	When User A simultaniosly make repay to digital tokens DWETH and DWNEAR by 10 tokens
	Then Failure flow expected on one of calls
	And Expected message  -> "failed to acquire repay action mutex for account {user}"