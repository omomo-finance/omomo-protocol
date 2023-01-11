Feature: User e2e flow


Background:
	Given The user Bob
	And Underlying token contract WETH with 1000 tokens for user Bob
	And Underlying token contract WNEAR with 0 tokens for user Bob
	And Underlying token contract WBTC with 0 tokens for user Bob


	And Exchange_rate for contracts equal 1 
	And Contracts accrued interests should be equal 0
	And token = 10^24


	Scenario: User Bob Supply 1000 tokens to weth_market, then borrow WNEAR 500 tokens from wnear_market, repay WNEAR 500, withdraw 1000 WETH tokens - positive flow
			Given The User Bob WETH, WNEAR, weth_market, wnear_market
			When User Bob supply 1000 tokens to weth_market,
			Then Borrow 250 WNEAR tokens from wnear_market,
			Then Borrow 250 WBTC tokens from DWBTC,
			Then Repay 250 WNEAR tokens from wnear_market,
			Then Repay 250 WBTC tokens from DWBTC,
			Then Withdraw 1000 WETH tokens

			Then Success flow expected