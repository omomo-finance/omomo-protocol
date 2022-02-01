# Liquidation

## Prerequisites

A liquidation is a process that happened when a borrower's health factor goes below 100% due to their collateral value, which not fully covering their borrows value. This might happen when the collateral decreases in value or the loan debt increases in value against each other. This collateral vs loan value ratio is shown in the health factor.

## Liquidation Threshold

The liquidation threshold is the percentage at which a position is defined as undercollateralised. For example, a Liquidation threshold of 95% means that if the value rises above 95% of the collateral, the position is undercollateralised and could be liquidated.

## Liquidation Bonus

Bonus on the price of assets of the collateral when liquidators purchase it as part of the liquidation of a loan that has passed the liquidation threshold.

## Health factor

The health factor is computed per account instead of per asset.

Each account may have multiple collateral asset supplies and may borrow multiple assets.

Each market has a configuration value volatility ratio which indicates the expected price stability factor. The higher the ratio, the higher expectation of the stability of the price of the corresponding asset.

To compute the current health factor for the account, we need to know the current prices of all collateral and borrowed assets. Firstly, we compute the affected for volatility sums of all collateral assets and borrowed assets.

$$
Collaterals_{affected} = \sum_{i=0}^{n}{Collaterals_i*Price_i* Volatility Ratio_i}\\
Borrows_{affected} = \sum_{i=0}^{n}{Borrows_i*Price_i* Volatility Ratio_i}\
$$

Now we can compute the health factor:

$$
H_{fact}= \frac{Collaterals_{affected}}{Borrows_{affected}}\
$$

If the health factor is higher than 100%, it means the account is in a good state and can't be liquidated. If the health factor is less than 100%, it means the account can be partially liquidated and can't borrow more without repaying some amount of the existing assets or providing more collateral assets.

## Liquidation flow

Contract liquidations are designed to make liquidators compete for the profit that they make during liquidations to minimize the loss taken by the unhealthy accounts. Instead of the fixed profit that is used in the legacy products, this contract introduces a variable discount with variable liquidation size.

> Liquidations rules:
>
> 1. the initial health factor of the liquidated accounts has to be below 100%
> 2. the discounted sum of the taken collateral should be less than the sum of repaid assets
> 3. the final health factor of the liquidated accounts has to stay below 100%

A liquidation action consists of the following:

> * account\_id - the account ID that is being liquidated
> * Assetsin - the assets and corresponding amounts to repay form borrowed assets
> * Assetsout - the assets and corresponding amounts to take from collateral assets

The discount is computed based on the initial health factor of the liquidated account:

$$
Discount = \frac{(1 - H_{fact})}{2}\
$$

Now we can compute the taken discounted collateral sum and the repaid borrowed sum:\


$$
Taken\_sum = \sum_{i=0}^{n}{(out\_asset_i * price_i)} \\ Discounted\_collateral\_sum = taken\_sum * (1 - discount) \\
Repaid\_sum =\sum_{i=0}^{n}{(in\_asset_i * price_i)}
$$

Once we action is completed, we can compute the final values and verify the liquidation rules:\


* health\_factor < 100%
* discounted\_collateral\_sum <= repaid\_sum
* new\_health\_factor < 100%

\
The first rule only allows to liquidate accounts in the unhealthy state. The second rule prevents from taking more collateral than the repaid sum (after discount). The third rule prevents the liquidator from repaying too much of the borrowed assets, only enough to bring closer to the 100%.
