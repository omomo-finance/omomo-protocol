source ./deploy.sh

ROOT_ACCOUNT=v1.omomo-finance.testnet
CONTROLLER_ACCOUNT=controller
ORACLE_ACCOUNT=oracle.omomo-finance.testnet
ETH_TOKEN=eth.fakes.testnet
NEAR_TOKEN=wrap.testnet
USDT_TOKEN=usdt.fakes.testnet
USDC_TOKEN=usdc.fakes.testnet

build_and_test
clean_up_previous_deployment $ROOT_ACCOUNT
create_markets $ROOT_ACCOUNT &
create_controller $ROOT_ACCOUNT &
wait

deploy_markets $ROOT_ACCOUNT &
# redeploy_markets $ROOT_ACCOUNT &
deploy_controller $ROOT_ACCOUNT &
# redeploy_controller $ROOT_ACCOUNT &
wait

create_account_on_underlyings_for_dtokens $ROOT_ACCOUNT
register_markets_on_controller $ROOT_ACCOUNT &
setup_reserves $ROOT_ACCOUNT &
wait

configure_acl $ROOT_ACCOUNT &
wait

# login
# near login

# view status
near view $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT view_markets '{}' --accountId $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT
near view $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT view_prices '{ "dtokens": ["wnear_market.'$ROOT_ACCOUNT'", "weth_market.'$ROOT_ACCOUNT'", "usdt_market.'$ROOT_ACCOUNT'", "usdc_market.'$ROOT_ACCOUNT'"] }' --accountId $CONTROLLER_ACCOUNT.$ROOT_ACCOUNT

# view balances
near view $ETH_TOKEN ft_balance_of '{"account_id": "weth_market.'$ROOT_ACCOUNT'"}'
near view $NEAR_TOKEN ft_balance_of '{"account_id": "wnear_market.'$ROOT_ACCOUNT'"}'
near view $USDT_TOKEN ft_balance_of '{"account_id": "usdt_market.'$ROOT_ACCOUNT'"}'
near view $USDC_TOKEN ft_balance_of '{"account_id": "usdc_market.'$ROOT_ACCOUNT'"}'
