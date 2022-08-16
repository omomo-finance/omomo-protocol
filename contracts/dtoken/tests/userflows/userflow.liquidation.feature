Feature: User liquidation flow

	Rule: Price for WETH, WNEAR are should be the same 5$ and volatility is 100%
		Background:
			Given The borrower, liquidator users, WETH, WNEAR, weth_market, wnear_market, Controller contracts
			And Digital token wnear_market contract with supplied  100 tokens by borrower
			And Digital token weth_market contract with borrowed 50 tokens by borrower
			And Utoken WETH contract with 50 tokens minted for liquidator account
			And liquidation_threshold should be equal 150% === 15000 in Ratio format
			And liquidation_incentive = 5%
			And Contracts accrued interests should be equal 0
			And token = 10^24


		Scenario: Liquidator tries to liquidate Borrower - negative flow  <Borrower HF in good condition>
			Given The borrower, liquidator users, WETH, WNEAR, weth_market, wnear_market, Controller contracts
			When Liquidator wants to liquidate weth_market with 25 WNEAR
			Then Failure flow expected  <User can't be liquidated as he has normal value of health factor>

	Rule: Price for WETH = 10$, WNEAR = 5$  and volatility is 100%
		Background:
			Given The borrower, liquidator users, WETH, WNEAR, weth_market, wnear_market, Controller contracts
			And Digital token wnear_market contract with supplied  100 tokens by borrower
			And Digital token weth_market contract with borrowed 50 tokens by borrower
			And Utoken WETH contract with 50 tokens minted for liquidator account
			And liquidation_threshold should be equal 150% === 15000 in Ratio format
			And liquidation_incentive = 5%
			And Contracts accrued interests should be equal 0
			And token = 10^24

		Scenario: Liquidator tries to liquidate Borrower - success flow
			Given The borrower, liquidator users, WETH, WNEAR, weth_market, wnear_market, Controller contracts
			When Liquidator wants to liquidate wnear_market with 10 WETH
			Then Success flow expected
			And Liquidator balance on WETH contract equal 40
			And Liquidator balance on wnear_market equal 21 // (1.05 * 10 * 10/5)
			And Borrower wnear_market supplies equal 79
			And Borrower Controller wnear_market supplies equal 79
			And Borrower weth_market borrows equal 40
			And Borrower Controller weth_market borrows equal 40


		Scenario: Liquidator tries to liquidate more than allowed - negative flow
			Given The borrower, liquidator users, WETH, WNEAR, weth_market, wnear_market, Controller contracts
			When Liquidator wants to liquidate wnear_market with 50 WETH
			Then Negative flow expected <Liquidation failed on controller, Max possible liquidation amount cannot be less than liquidation amount>

		Scenario: Liquidator tries to liquidate more than collaterals exist - negative flow
			Given The borrower, liquidator users, WETH, WNEAR, weth_market, wnear_market, Controller contracts
			When Liquidator wants to liquidate wnear_market with 50 weth_market
			Then Negative flow expected <Liquidation failed on controller, Borrower collateral amount is not enough to pay it to liquidator>

		Scenario: Liquidator and Borrower are the same persons - negative flow
			Given The borrower, liquidator users, WETH, WNEAR, weth_market, wnear_market, Controller contracts
			When Liquidator wants to liquidate his own borrow collaterals wnear_market with 20 weth_market
			Then Negative flow expected <Liquidation failed on controller, Liquidation cannot liquidate his on borrow>

		Scenario: Liquidator tries to liquidate ou wrong dtoken - negative flow
			Given The borrower, liquidator users, WETH, WNEAR, weth_market, wnear_market, Controller contracts
			When Liquidator wants to liquidate on WNEAR, where user doesn't have borrows
			Then Negative flow expected