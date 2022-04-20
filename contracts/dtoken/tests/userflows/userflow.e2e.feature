Feature: User e2e flow


Background:
	Given The user A
	And Underlying token contract WETH with 1000 tokens for user A
	And Underlying token contract WNEAR with 0 tokens for user A
	And Underlying token contract WBTC with 0 tokens for user A


	And Exchange_rate for contracts equal 1 
	And Contracts accrued interests should be equal 0
	And token = 10^24


Scenario: User A Supply 1000 tokens to DWETH, then borrow WNEAR 500 tokens from DWNEAR, repay WNEAR 500, withdraw 1000 WETH tokens - positive flow
		Given The user A WETH, WNEAR, DWETH, DWNEAR
		When User A supply 1000 tokens to DWETH, 
		Then Borrow 250 WNEAR tokens from DWNEAR, 
		Then Borrow 250 WBTC tokens from DWBTC,
		Then Repay 250 WNEAR tokens from DWNEAR,
		Then Repay 250 WBTC tokens from DWBTC,
		Then Withdraw 1000 WETH tokens

		Then Success flow expected