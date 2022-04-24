Feature: User liquidation flow

	Rule: Price for WETH, WNEAR are should be the same 5$
		Background:
			Given The borrower, liquidator users, WETH, WNEAR, DWETH, DWNEAR contracts
			And Digital token DWETH contract with supplied 100 tokens by borrower
			And Digital token DWNEAR contract with supplied 50 tokens by borrower
			And Digital token DWETH contract with supplied 100 tokens by liquidator
			And health_threshold should be equal 150% === 15000 in Ratio format
			And Contracts accrued interests should be equal 0
			And token = 10^24


		Scenario: Liquidator tries to liquidate Borrower - negative flow  <Borrower HF in good condition>
			Given Borrower, Liquidator, WETH, WNEAR, DWETH, DWNEAR contracts
			When Liquidator wants to liquidate 50 WNEAR with 50 DWETH
			Then Failure flow expected  <User can't be liquidated as he has normal value of health factor>

	Rule: Price for WETH = 5$, WNEAR = 10$
		Background:
			Given The borrower, liquidator users, WETH, WNEAR, DWETH, DWNEAR contracts
			And Digital token DWETH contract with supplied 100 tokens by borrower
			And Digital token DWNEAR contract with supplied 50 tokens by borrower
			And Digital token DWETH contract with supplied 100 tokens by liquidator
			And health_threshold should be equal 150% === 15000 in Ratio format
			And Contracts accrued interests should be equal 0
			And token = 10^24

		Scenario: Liquidator tries to liquidate Borrower - success flow
			Given Borrower, Liquidator, WETH, WNEAR, DWETH, DWNEAR contracts
			When Liquidator wants to liquidate 10 WNEAR with 20 DWETH
			Then Success flow expected

		Scenario: Liquidator tries to liquidate Borrower more than allowed - negative flow
			Given Borrower, Liquidator, WETH, WNEAR, DWETH, DWNEAR contracts
			When Liquidator wants to liquidate 25 WNEAR with 50 DWETH
			Then Negative flow expected <Liquidation failed on controller, Max possible liquidation amount cannot be less than liquidation amount>

		Scenario: Liquidator tries to liquidate Borrower and payed not enough for collateral liquidation - negative flow
			Given Borrower, Liquidator, WETH, WNEAR, DWETH, DWNEAR contracts
			When Liquidator wants to liquidate 10 WNEAR with 15 DWETH
			Then Negative flow expected <Liquidation failed on controller, Borrower collateral amount is not enough to pay it to liquidator>

		Scenario: Liquidator and Borrower are the same persons - negative flow
			Given Borrower, Liquidator, WETH, WNEAR, DWETH, DWNEAR contracts
			When Liquidator wants to liquidate his own borrow 10 WNEAR with 20 DWETH
			Then Negative flow expected <Liquidation failed on controller, Liquidation cannot liquidate his on borrow>