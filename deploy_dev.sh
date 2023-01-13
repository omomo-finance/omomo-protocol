source ./deploy.sh

ROOT_ACCOUNT=dev.v1.omomo-finance.testnet
CONTROLLER_ACCOUNT=controller
ORACLE_ACCOUNT=oracle.omomo-finance.testnet
ETH_TOKEN=weth.dev.v1.omomo-finance.testnet
NEAR_TOKEN=wnear.dev.v1.omomo-finance.testnet
USDT_TOKEN=usdt.dev.v1.omomo-finance.testnet
USDC_TOKEN=usdc.dev.v1.omomo-finance.testnet


# build_and_test

clean_up_previous_deployment $ROOT_ACCOUNT
create_underlying_tokens_and_markets $ROOT_ACCOUNT &
create_controller $ROOT_ACCOUNT &
wait

deploy_underlyings $ROOT_ACCOUNT &
deploy_markets $ROOT_ACCOUNT &
deploy_controller $ROOT_ACCOUNT &
wait

create_account_on_underlyings_for_dtokens $ROOT_ACCOUNT $ROOT_ACCOUNT
register_markets_on_controller $ROOT_ACCOUNT &
setup_reserves $ROOT_ACCOUNT &
wait

configure_acl $ROOT_ACCOUNT &
wait

# view status
near view $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT view_markets '{}' --accountId $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT
near view $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT view_prices '{ "dtokens": ["wnear_market.'$ROOT_ACCOUNT'", "weth_market.'$ROOT_ACCOUNT'", "usdt_market.'$ROOT_ACCOUNT'", "usdc_market.'$ROOT_ACCOUNT'"] }' --accountId $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT

# view balances
near view weth.$ROOT_ACCOUNT ft_balance_of '{"account_id": "weth_market.'$ROOT_ACCOUNT'"}'
near view wnear.$ROOT_ACCOUNT ft_balance_of '{"account_id": "wnear_market.'$ROOT_ACCOUNT'"}'
near view usdt.$ROOT_ACCOUNT ft_balance_of '{"account_id": "usdt_market.'$ROOT_ACCOUNT'"}'
near view usdc.$ROOT_ACCOUNT ft_balance_of '{"account_id": "usdc_market.'$ROOT_ACCOUNT'"}'
