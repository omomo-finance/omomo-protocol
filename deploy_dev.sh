# setup
source ./deploy.sh

ROOT_ACCOUNT=develop.v1.omomo-finance.testnet
CONTROLLER_ACCOUNT=controller
ORACLE_ACCOUNT=oracle.omomo-finance.testnet

ETH_TOKEN=weth.develop.v1.omomo-finance.testnet
ETH_TOKEN_DECIMALS=24

NEAR_TOKEN=wnear.develop.v1.omomo-finance.testnet
NEAR_TOKEN_DECIMALS=24

USDT_TOKEN=usdt.develop.v1.omomo-finance.testnet
USDT_TOKEN_DECIMALS=24

USDC_TOKEN=usdc.develop.v1.omomo-finance.testnet
USDC_TOKEN_DECIMALS=24

# deployment steps
build_and_test

clean_up_previous_deployment $ROOT_ACCOUNT &
clean_up_tokens $ROOT_ACCOUNT &
wait

create_underlying_tokens $ROOT_ACCOUNT &
create_markets $ROOT_ACCOUNT &
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
near view $ETH_TOKEN  ft_balance_of '{"account_id": "weth_market.'$ROOT_ACCOUNT'"}'
near view $NEAR_TOKEN ft_balance_of '{"account_id": "wnear_market.'$ROOT_ACCOUNT'"}'
near view $USDT_TOKEN ft_balance_of '{"account_id": "usdt_market.'$ROOT_ACCOUNT'"}'
near view $USDC_TOKEN ft_balance_of '{"account_id": "usdc_market.'$ROOT_ACCOUNT'"}'

# mark commit with deploy-tag
LATEST_TAG=`git tag -l "v*" | sort -r | head -n1`
echo $LATEST_TAG

DEPLOY_TAG="dev_deploy_"${LATEST_TAG}_`date +%s`
echo $DEPLOY_TAG

git tag $DEPLOY_TAG
git push origin $DEPLOY_TAG