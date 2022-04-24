Feature: User Withdraw flow


Rule: No borrows were done
	Background:
		Given The user A
		And Underlying token contract WETH with 0 tokens for user A
		And Underlying token contract WNEAR with 0 tokens for user A
		And Underlying token contract WBTC with 0 tokens for user A
		And Digital token DWETH contract with supplied 100 tokens by user A
		And Digital token DWNEAR contract with supplied 99 tokens by user A
		And Digital token DWBTC contract with supplied 0 tokens by user A
		And Exchange_rate for contracts equal 1
		And Contracts accrued interests should be equal 0
		And token = 10^24

	Scenario: User A withdraw from DWETH  digital token 100 tokens - positive flow  <Success flow>
		Given User A and DWETH contract
		When User A withdraw from DWETH contract 100 tokens
		Then Success flow expected
		And User balance is 0 DWETH
		And User balance is 100 WETH

	Scenario: User A withdraw from DWETH  digital token 0 tokens - negative flow  <Amount should be positive>
		Given User A and DWETH contract
		When User A withdraw from DWETH contract 0 tokens
		Then Failure flow expected

	Scenario: User A withdraw from DWNEAR digital token 100 tokens - negative flow <Withdraw more than supplies>
		Given User A and DWNEAR contract
		When User A withdraw from DWNEAR contract 100 tokens
		Then Failure flow expected <Withdraw more than supplies were done>

	Scenario: User A withdraw from DWBTC digital token 100 tokens - negative flow <Withdraw with no supplies>
		Given User A and DWBTC contract
		When User A withdraw from DWBTC contract 100 tokens
		Then Failure flow expected <Cannot calculate utilization rate as denominator is equal 0>

	Scenario: Sequential test after failure - positive flow <Success flow>
		Given User A and DWETH, DWNEAR contracts
		When User A withdraw from DWNEAR contract 0 tokens, receive failure, after withdraw from DWETH contract 100 tokens
		Then Success flow expected
		And User balance is 0 DWETH
		And User balance is 100 WETH

	Scenario: Concurrency test - simultanious withdraw for DWETH and DWNEAR contracts - negative flow  <Failure flow due to global action restriction>
		Given User A and DWETH, DWNEAR contracts
		When User A simultaniosly make withdraw to on DWETH and DWNEAR contracts with 10 tokens
		Then Failure flow expected on one of calls
		And Expected message  -> "failed to acquire withdraw action mutex for account {user}"

Rule: Borrows are exist
	Given The user A
		And Underlying token contract WETH with 0 tokens for user A
		And Digital token DWETH contract with supplied 100 tokens by user A
		And Digital token DWBTC contract with borrow 50 tokens by user A
		And Price for WETH = 5$, WBTC = 5$
		And health_threshold should be equal 100% === 10000 in Ratio format
		And Exchange_rate for contracts equal 1
		And Contracts accrued interests should be equal 0
		And token = 10^24

	Scenario: Withdraw of deposited tokens that used partially as collaterals - positive flow  <Success flow>
		Given User A and DWETH contract
		When User A withdraw from DWETH contract 50 tokens
		Then Success flow expected
		And User balance is 50 DWETH
		And User balance is 50 WETH

	Scenario: Withdraw of deposited tokens that used partially as collaterals - negative flow  <Withdraw amount more than available supplies>
		Given User A and DWETH contract
		When User A withdraw from DWETH contract 70 tokens
		Then Failure flow expected <Withdraw amount more than available supplies>