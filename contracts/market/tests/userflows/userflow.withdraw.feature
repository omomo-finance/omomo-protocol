Feature: User Withdraw flow


Rule: No borrows were done
	Background:
		Given The user Bob
		And Underlying token contract WETH with minted 100 tokens for digital token weth_market
		And Underlying token contract WNEAR with minted 100 tokens for digital token wnear_market
		And Underlying token contract WBTC with minted 100 tokens for digital token DWBTC
		And Underlying token contract WETH with 100 tokens for user Bob
		And Underlying token contract WNEAR with 99 tokens for user Bob
		And Underlying token contract WBTC with 0 tokens for user Bob
		And Digital token weth_market contract with supplied 100 tokens by user Bob
		And Digital token wnear_market contract with supplied 99 tokens by user Bob
		And Digital token DWBTC contract with supplied 0 tokens by user Bob
		And Contracts accrued interests should be equal 0
		And token = 10^24

	Scenario: User Bob withdraw from weth_market  digital token 100 tokens - positive flow  <Success flow>
		Given User Bob and weth_market contract
		And Exchange_rate for weth_market contract equal 2
		When User Bob withdraw from weth_market contract 100 tokens
		Then Success flow expected
		And User balance is 0 weth_market
		And User balance is 50 WETH

	Scenario: User Bob withdraw from weth_market  digital token 0 tokens - negative flow  <Amount should be positive>
		Given User Bob and weth_market contract
		When User Bob withdraw from weth_market contract 0 tokens
		Then Failure flow expected

	Scenario: User Bob withdraw from wnear_market digital token 100 tokens - negative flow <Withdraw more than supplies>
		Given User Bob and wnear_market contract
		When User Bob withdraw from wnear_market contract 100 tokens
		Then Failure flow expected <Withdraw more than supplies were done>

	Scenario: Sequential test after failure - positive flow <Success flow>
		Given User Bob and weth_market, wnear_market contracts
		And Exchange_rate for weth_market contract equal 2
		When User Bob withdraw from wnear_market contract 0 tokens, receive failure, after withdraw from weth_market contract 100 tokens
		Then Success flow expected
		And User balance is 0 weth_market
		And User balance is 50 WETH

	Scenario: Concurrency test - simultaneous withdraw for weth_market and wnear_market contracts - negative flow  <Failure flow due to global action restriction>
		Given User Bob and weth_market, wnear_market contracts
		When User Bob simultaniosly make withdraw to on weth_market and wnear_market contracts with 10 tokens
		Then Failure flow expected on call executed second
		And Failure flow expected message  is "failed to acquire withdraw action mutex for account {user}"

Rule: Borrows are exist
	Given The user Bob
		And Underlying token contract WETH with 0 tokens for user Bob
		And Digital token weth_market contract with supplied 100 tokens by user Bob
		And Digital token DWBTC contract with borrow 50 tokens by user Bob
		And Price for WETH = 5$, WBTC = 5$
		And liquidation_threshold should be equal 100% === 10000 in Ratio format
		And Exchange_rate for contracts equal 1
		And Contracts accrued interests should be equal 0
		And token = 10^24

	Scenario: Withdraw of deposited tokens that used partially as collaterals - positive flow  <Success flow>
		Given User Bob and weth_market contract
		And Exchange_rate for weth_market contract equal 2
		When User Bob withdraw from weth_market contract 50 tokens
		Then Success flow expected
		And User balance is 50 weth_market
		And User balance is 25 WETH

	Scenario: Withdraw of deposited tokens that used partially as collaterals - negative flow  <Withdraw amount more than available supplies>
		Given User Bob and weth_market contract
		When User Bob withdraw from weth_market contract 70 tokens
		Then Failure flow expected <Withdraw amount more than available supplies>