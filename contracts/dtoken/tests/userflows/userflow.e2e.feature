Feature: User e2e flow


Background:
	Given The user Bob
	And Underlying token contract WETH with 1000 tokens for user Bob
	And Underlying token contract WNEAR with 0 tokens for user Bob
	And Underlying token contract WBTC with 0 tokens for user Bob


	And Exchange_rate for contracts equal 1 
	And Contracts accrued interests should be equal 0
	And token = 10^24


	Scenario: User Bob Supply 1000 tokens to DWETH, then borrow WNEAR 500 tokens from DWNEAR, repay WNEAR 500, withdraw 1000 WETH tokens - positive flow
			Given The User Bob WETH, WNEAR, DWETH, DWNEAR
			When User Bob supply 1000 tokens to DWETH,
			Then Borrow 250 WNEAR tokens from DWNEAR,
			Then Borrow 250 WBTC tokens from DWBTC,
			Then Repay 250 WNEAR tokens from DWNEAR,
			Then Repay 250 WBTC tokens from DWBTC,
			Then Withdraw 1000 WETH tokens

			Then Success flow expected