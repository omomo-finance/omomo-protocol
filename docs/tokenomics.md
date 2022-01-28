#Tokenomics

---

## Interest rate model

Interest rate model in Nearlend is similar to Compound protocol.

### The utilization rate
Interest rate in Nearlend is determined as a function of a metric known as the utilization rate. 
The utilization rate `U` for a money market a is defined as:

> U<sub>a</sub> = Borrows<sub>a</sub> / (Cash<sub>a</sub> + Borrows<sub>a</sub> - Reserves<sub>a</sub>) 
> - Borrows<sub>a</sub> refers to the amount of a borrowed.
> - Cash<sub>a</sub> refers to the amount of a left in the system.
> - Reserves<sub>a</sub> refers to the amount of a that Nearlend keeps as profit.

Intuitively speaking, this is the percentage of money borrowed out of the total money supplied.


A high ratio signifies that a lot of borrowing is taking place, so interest rates go up to get more people to inject cash into the system. A low ratio signifies that demand for borrowing is low, so interest rates go down to encourage more people to borrow cash from the system. This follows economic theory's idea of price (the "price" of money is its interest rate) relative to supply and demand.


### Borrow & Supply rates

Borrow and supply rates are calculated using the utilization rate and several arbitrary constants.

The supply rate is calculated as follows:

> Supply Interest Rate<sub>a</sub> = Borrowing Interest Rate<sub>a</sub> * U<sub>a</sub> * ( 1−Reserve Factor<sub>a</sub> )


### Standard Interest Rate Model

The borrowing rate's calculation depends on something called an interest rate model -- the algorithmic model to determine a money market's borrow and supply rates. 

This standart interest rate model takes in two parameters:

> * Base rate per year, the minimum borrowing rate
> * Multiplier per year, the rate of increase in interest rate with respect to utilization

> Borrow Interest Rate = Multiplier * Utilization Rate + Base Rate

### The Jump Rate model

Some markets follow what is known as the "Jump Rate" model. This model has the standard parameters:

* Base rate per year, the minimum borrowing rate
* Multiplier per year, the rate of increase in interest rate with respect to utilization

but it also introduces two new parameters:

> * Kink, the point in the model in which the model follows the jump multiplier
> * Jump Multiplier per year, the rate of increase in interest rate with respect to utilization after the "kink"

The borrow rate of the jump rate model is defined as follows:

> Borrow Interest Rate = Multiplier * min(U<sub>a</sub>, Kink) + Jump Multiplier *
max(0, U<sub>a</sub> - Kink) + Base Rate;


### Example:

|  	|  	|
|---	|---	|
| Supplies 	| 200M 	|
| Borrows 	| 180M 	|
| Cash 	| 20M 	|
| Base rate, annual 	| 0% 	|
| Multiplier 	| 5% 	|
| Kink 	| 80% 	|
| Jump multiplier 	| 109% 	|

Doing the math:

>
> U<sub>a</sub> = $180M / ( $180M + $20M ) = 90% 
>
> Borrow Interest Rate = 5% * 80% + 109% * ( 90% − 80% ) + 0% = 14.9%
>
> Supply Interest Rate<sub>a</sub> = 14.9% * 90% * ( 1 − 7% ) = 12.5%
> 
