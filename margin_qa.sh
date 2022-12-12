near call usdt_market.qa.v1.nearlend.testnet set_eligible_to_borrow_uncollateralized_account '{ "account": "limit_orders.v1.nearlend.testnet" }' --accountId shared_admin.testnet
near view usdt_market.qa.v1.nearlend.testnet get_eligible_to_borrow_uncollateralized_account '{ "account": "limit_orders.v1.nearlend.testnet" }' 

near call controller.qa.v1.nearlend.testnet set_eligible_to_borrow_uncollateralized_account '{ "account": "limit_orders.v1.nearlend.testnet" }' --accountId controller.qa.v1.nearlend.testnet
near view controller.qa.v1.nearlend.testnet get_eligible_to_borrow_uncollateralized_account '{ "account": "limit_orders.v1.nearlend.testnet" }'