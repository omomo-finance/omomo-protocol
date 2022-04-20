Feature: User liquidation flow

Background:
	Given The user A
	And Underlying token contract WETH with 100 tokens for user A
	And Exchange_rate for contracts equal 1 
	And Contracts accrued interests should be equal 0
	And token = 10^24


