---
description: Interest rate model in Nearlend is similar to Compound protocol.
---

# Interest rate model

## The utilization rate

The interest rate in Nearlend is determined as a function of a metric known as the utilization rate. It signifies the percentage of money borrowed out of the total amount supplied.

The utilization rate is calculated with the following formula:

$$
U_a = \frac{Borrows_a}{(Cash_a + Borrows_a - Reserves_a)}\
$$

> * U\_a **** the utilization rate
> * Borrows\_a refers to the amount of a borrowed.
> * Cash\_a refers to the amount of a left in the system.
> * Reserves\_a refers to the amount of a that Nearlend keeps as profit.

A high ratio signifies that a lot of borrowing is taking place, so interest rates go up to get more people to inject cash into the system. A low ratio signifies that the demand for borrowing is low, so interest rates go down to encourage more people to borrow cash from the system. This follows economic theory's idea of price (the "price" of money is its interest rate) relative to supply and demand.

## Borrow & Supply rates

Borrow and supply rates are calculated using the utilization rate and several arbitrary constants.

The supply rate is calculated as follows:

$$
Supply Interest Rate_a = Borrowing Interest Rate_a * U_a * ( 1−Reserve Factor_a )\
$$

The borrow interest rate calculation is described in the following part.

## Standard Interest Rate Model

The borrowing rate calculation depends on something called an interest rate model – the algorithmic model to determine borrow and supply rates at the money market.

This standard interest rate model takes in two parameters:&#x20;

> * Base rate per year – the minimum borrowing rate;
> * Multiplier per year – the rate of increase in interest rate with respect to utilization.

$$
Borrow Interest Rate = Multiplier * Utilization Rate + Base Rate
$$

## The Jump Rate model

Some markets follow what is known as the "Jump Rate" model. This model has the standard parameters:

* Base rate per year - the minimum borrowing rate
* Multiplier per year - the rate of increase in interest rate with respect to utilization after the kink.

Yet, it also introduces two new parameters:

> * Kink, the point in the model in which the model follows the jump multiplier
> * Jump Multiplier per year, the rate of increase in interest rate with respect to utilization after the "kink"

The borrowing rate of the jump rate model is defined as follows:

$$
Borrow Interest Rate = Multiplier * min(U_a, Kink) + \\Jump Multiplier * max(0, U_a - Kink) +\\ Base Rate\
$$

## Example

| Parameter:        | Value |
| ----------------- | :---: |
| Supplies          |  200M |
| Borrows           |  180M |
| Cash              |  20M  |
| Base rate, annual |   0%  |
| Multiplier        |   5%  |
| Kink              |  80%  |
| Jump multiplier   |  109% |

Doing the math:

$$
U_a = \frac{$180M}{$180M+$20M} = 90\%
$$

$$
BorrowInterestRate = 5\% * 80\% + 109\% * ( 90\% − 80\% ) + 0\% = 14.9\%
$$

$$
SupplyInterestRate = 14.9\% * 90\% * (1-7\%)=12.5\%
$$

