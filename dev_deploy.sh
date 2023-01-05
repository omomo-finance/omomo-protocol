# login
# near login

# build & test
./build.sh && ./test.sh

ADMIN_ACCOUNT=shared_admin.testnet
ORACLE_ID=oracle.omomo-finance.testnet

# Its instead of omomo-finance.testnet
echo -n "" > /tmp/empty
near dev-deploy -f /tmp/empty
OWNER_ID="$(cat neardev/dev-account)"

# deploy underlyings
near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
WETH_TOKEN="$(cat neardev/dev-account)"
near call $WETH_TOKEN new_default_meta '{"owner_id": "'$OWNER_ID'", "name": "Wrapped Ethereum", "symbol": "WETH", "total_supply": "1000000000000000000000000000"}' --account_id $OWNER_ID

near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
WNEAR_TOKEN="$(cat neardev/dev-account)"
near call $WNEAR_TOKEN new_default_meta '{"owner_id": "'$OWNER_ID'", "name": "Wrapped Near", "symbol": "WNEAR", "total_supply": "1000000000000000000000000000"}' --account_id $OWNER_ID

near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
USDT_TOKEN="$(cat neardev/dev-account)"
near call $USDT_TOKEN new_default_meta '{"owner_id": "'$OWNER_ID'", "name": "Tether", "symbol": "USDT", "total_supply": "1000000000000000000000000000"}' --account_id $OWNER_ID

near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/test_utoken.wasm
USDC_TOKEN="$(cat neardev/dev-account)"
near call $USDC_TOKEN new_default_meta '{"owner_id": "'$OWNER_ID'", "name": "USD Coin", "symbol": "USDC", "total_supply": "1000000000000000000000000000"}' --account_id $OWNER_ID

# deploy controller
near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/controller.wasm
CONTROLLER="$(cat neardev/dev-account)"
near call $CONTROLLER new_with_config '{
      "owner_id":"'$OWNER_ID'",
      "oracle_account_id":"'$ORACLE_ID'"
}' --account_id $OWNER_ID

# deploy markets
near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
WETH_MARKET="$(cat neardev/dev-account)"
near call $WETH_MARKET new_with_config '{
        "owner_id":"'$OWNER_ID'",
        "underlying_token_id":"'$WETH_TOKEN'",
        "controller_account_id":"'$CONTROLLER'",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
            "kink":"650000000000000000000000",
            "multiplier_per_block":"3044140030441400",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"38051750380517500",
            "reserve_factor":"10000000000000000000000"
        }
}' --account_id $OWNER_ID

near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
WNEAR_MARKET="$(cat neardev/dev-account)"
near call $WNEAR_MARKET new_with_config '{
        "owner_id":"'$OWNER_ID'",
        "underlying_token_id":"'$WNEAR_TOKEN'",
        "controller_account_id":"'$CONTROLLER'",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
            "kink":"650000000000000000000000",
            "multiplier_per_block":"3044140030441400",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"38051750380517500",
            "reserve_factor":"10000000000000000000000"
        }
}' --account_id $OWNER_ID

near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
USDT_MARKET="$(cat neardev/dev-account)"
near call $USDT_MARKET new_with_config '{
        "owner_id":"'$OWNER_ID'",
        "underlying_token_id":"'$USDT_TOKEN'",
        "controller_account_id":"'$CONTROLLER'",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
            "kink":"800000000000000000000000",
            "multiplier_per_block":"1522070015220700",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"28538812785388100",
            "reserve_factor":"10000000000000000000000"
        }
}' --account_id $OWNER_ID

near dev-deploy -f ./contracts/target/wasm32-unknown-unknown/release/dtoken.wasm
USDC_MARKET="$(cat neardev/dev-account)"
near call $USDC_MARKET new_with_config '{
        "owner_id":"'$OWNER_ID'",
        "underlying_token_id":"'$USDC_TOKEN'",
        "controller_account_id":"'$CONTROLLER'",
        "initial_exchange_rate":"1000000000000000000000000",
        "interest_rate_model":{
            "kink":"800000000000000000000000",
            "multiplier_per_block":"1522070015220700",
            "base_rate_per_block":"0",
            "jump_multiplier_per_block":"28538812785388100",
            "reserve_factor":"10000000000000000000000"
        }
}' --account_id $OWNER_ID

# fund weth_market.omomo-finance.testnet
near call $WETH_TOKEN storage_deposit '{"account_id": "'$WETH_MARKET'"}' --accountId  $OWNER_ID --amount 0.25
near call $WNEAR_TOKEN storage_deposit '{"account_id": "'$WNEAR_MARKET'"}' --accountId  $OWNER_ID --amount 0.25
# near call wrap.testnet storage_deposit '{"account_id": "wnear_market.omomo-finance.testnet"}' --accountId omomo-finance.testnet --amount 0.25
near call $USDT_TOKEN storage_deposit '{"account_id": "'$USDT_MARKET'"}' --accountId $OWNER_ID --amount 0.25
near call $USDC_TOKEN storage_deposit '{"account_id": "'$USDC_MARKET'"}' --accountId $OWNER_ID --amount 0.25

# near call weth.omomo-finance.testnet mint '{"account_id": "weth_market.omomo-finance.testnet", "amount": "1"}' --accountId omomo-finance.testnet 
# near call wnear.omomo-finance.testnet mint '{"account_id": "wnear_market.omomo-finance.testnet", "amount": "1"}' --accountId omomo-finance.testnet 
# near call usdt.omomo-finance.testnet mint '{"account_id": "usdt_market.omomo-finance.testnet", "amount": "1"}' --accountId omomo-finance.testnet 
# near call usdc.omomo-finance.testnet mint '{"account_id": "usdc_market.omomo-finance.testnet", "amount": "1"}' --accountId omomo-finance.testnet 

# register market
near call $CONTROLLER add_market '{"asset_id": "'$WETH_TOKEN'", "dtoken": "'$WETH_MARKET'", "ticker_id": "weth", "ltv": "0.4", "lth": "0.8"}' --accountId $OWNER_ID
near call $CONTROLLER add_market '{"asset_id": "'$WNEAR_TOKEN'", "dtoken": "'$WNEAR_MARKET'", "ticker_id": "wnear", "ltv": "0.4", "lth": "0.8"}' --accountId $OWNER_ID
near call $CONTROLLER add_market '{"asset_id": "'$USDT_TOKEN'", "dtoken": "'$USDT_MARKET'", "ticker_id": "usdt", "ltv": "0.8", "lth": "0.9"}' --accountId $OWNER_ID
near call $CONTROLLER add_market '{"asset_id": "'$USDC_TOKEN'", "dtoken": "'$USDC_MARKET'", "ticker_id": "usdc", "ltv": "0.8", "lth": "0.9"}' --accountId $OWNER_ID

near view $CONTROLLER view_markets '{}' --accountId $CONTROLLER

near view $CONTROLLER view_prices '{ "dtokens": [
    "'$WETH_MARKET'",
    "'$WNEAR_MARKET'",
    "'$USDT_MARKET'",
    "'$USDC_MARKET'"] }' --accountId $CONTROLLER


near call $WETH_TOKEN mint '{"account_id": "'$OWNER_ID'", "amount": "1000000000000000000000000000"}' --accountId $OWNER_ID
near call $WNEAR_TOKEN mint '{"account_id": "'$OWNER_ID'", "amount": "1000000000000000000000000000"}' --accountId $OWNER_ID
near call $USDT_TOKEN mint '{"account_id": "'$OWNER_ID'", "amount": "1000000000000000000000000000"}' --accountId $OWNER_ID
near call $USDC_TOKEN mint '{"account_id": "'$OWNER_ID'", "amount": "1000000000000000000000000000"}' --accountId $OWNER_ID

near call $WETH_TOKEN ft_transfer_call '{"receiver_id": "'$WETH_MARKET'", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId $OWNER_ID
near call $WNEAR_TOKEN ft_transfer_call '{"receiver_id": "'$WNEAR_MARKET'", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId $OWNER_ID
near call $USDT_TOKEN ft_transfer_call '{"receiver_id": "'$USDT_MARKET'", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId $OWNER_ID
near call $USDC_TOKEN ft_transfer_call '{"receiver_id": "'$USDC_MARKET'", "amount": "1000000000000000000000000000", "msg": "\"Reserve\""}' --depositYocto 1 --gas 300000000000000 --accountId $OWNER_ID

near view $WETH_TOKEN ft_balance_of '{"account_id": "'$WETH_MARKET'"}'
near view $WNEAR_TOKEN ft_balance_of '{"account_id": "'$WNEAR_MARKET'"}'
# near view wrap.testnet ft_balance_of '{"account_id": "wnear_market.omomo-finance.testnet"}'
near view $USDT_TOKEN ft_balance_of '{"account_id": "'$USDT_MARKET'"}'
near view $USDC_TOKEN ft_balance_of '{"account_id": "'$USDC_MARKET'"}'

# set shared admin as admin for dtokens
near call $WETH_MARKET set_admin '{"account": "'$ADMIN_ACCOUNT'"}' --gas 300000000000000 --accountId $OWNER_ID
near call $WNEAR_MARKET set_admin '{"account": "'$ADMIN_ACCOUNT'"}' --gas 300000000000000 --accountId $OWNER_ID
near call $USDT_MARKET set_admin '{"account": "'$ADMIN_ACCOUNT'"}' --gas 300000000000000 --accountId $OWNER_ID
near call $USDC_MARKET set_admin '{"account": "'$ADMIN_ACCOUNT'"}' --gas 300000000000000 --accountId $OWNER_ID


LG='\033[1;30m' # Arrows color (Dark gray)
TC='\033[0;33m' # Text color (Orange)
NC='\033[0m' # No Color

echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "ORACLE_ID=$ORACLE_ID"
echo -e "OWNER_ID=$OWNER_ID"
echo -e "WETH_TOKEN=$WETH_TOKEN"
echo -e "WNEAR_TOKEN=$WNEAR_TOKEN"
echo -e "USDT_TOKEN=$USDT_TOKEN"
echo -e "USDC_TOKEN=$USDC_TOKEN"
echo -e "WETH_MARKET=$WETH_MARKET"
echo -e "WNEAR_MARKET=$WNEAR_MARKET"
echo -e "USDT_MARKET=$USDT_MARKET"
echo -e "USDC_MARKET=$USDC_MARKET"
echo -e "CONTROLLER=$CONTROLLER"
