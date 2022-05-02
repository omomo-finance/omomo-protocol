Feature: User Supply flow

Background:
	Given The user Bob
	And Underlying token contract WETH with minted 100 tokens for user Bob
	And Underlying token contract WNEAR with 50 tokens  for user Bob
	And Digital token DWETH contract
	And Digital token DWNEAR contract
	And token = 10^24

Scenario: User Bob supplies to digital token DWETH 100 tokens - positive flow <Success flow>
	Given User Bob and DWETH contract
	When User Bob supplies to digital token DWETH 100 tokens
	Then Success flow expected
	And User balance is 100 DWETH
	And User balance is 0 WETH

Scenario: User Bob supplies to digital token DWNEAR 100 tokens - negative flow <Not enough tokens>
	Given User Bob and DWNEAR contract
	When User Bob supplies to digital token DWNEAR 100 tokens
	Then Failure flow expected

Scenario: User Bob supplies to digital token DWNEAR 0 tokens - negative flow  <Amount should be positive>
	Given User Bob and DWNEAR contract
	When User Bob supplies to digital token DWNEAR 0 tokens
	Then Failure flow expected	

Scenario: Sequential test after failure - positive flow <Success flow>
	Given User Bob and DWETH, DWNEAR contracts
	When User Bob supplies to digital token DWNEAR 0 tokens, receive failure, after supplies to digital token DWETH 100 tokens
	Then Success flow expected
	And User balance is 100 DWETH
	And User balance is 0 WETH

Scenario: Concurrency test - simultaneous deposit to DWETH and DWNEAR contracts - negative flow  <Failure flow due to global action restriction>
	Given User Bob and DWETH, DWNEAR contracts
	When User Bob simultaniosly make supply to digital tokens DWETH and DWNEAR by 10 tokens
	Then Failure flow expected on call executed second
	And Failure flow expected message  is "failed to acquire supply action mutex for account {user}"



