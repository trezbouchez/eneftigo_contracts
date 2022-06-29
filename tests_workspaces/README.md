Run single test while in this (/tests) directory with:

cargo run --example TEST_NAME

*******************
IMPORTANT NOTE: Gas
*******************

Some important gas-related findings can be illustrated by analyzing this receipt in the testnet explorer:
ReceiptID: 58oawu7SbU16tAfGHmDLHz6Q61Q3RkGzCsovX5X1dx2Y
It was a simple function call, no storage, no x-calls.

The first thing which may seem wrong is the fact that the ATTACHED GAS is lower than GAS USED. There are two
causes:
1. GAS USED, as shown by the explorer, includes the gas burnt by the transfer refunding signer for the excess 
attached gas which was not burnt. While this transfer itself is burning some extra gas - Gas(223_182_562_500) 
at the time of writing -  this call is initiated by the system and this extra gas is not paid by the signer.
In the example this refund transfer was performed with ReceiptID AumgKiFvr4YDNMgyEx3WcvnP8nhjhvQj3C8NU4ro5yaj
2. The ATTACHED GAS is not the whole gas covered by signer. It is the gas reserved for function execution only. 
The system is buying some extra gas needed for turning the transaction into a receipt, which at the time of 
writing was Gas(2_428_073_043_512). In the example this txn->receipt gas is displayed in the first section 
labelled "Convert Transaction To Receipt".

The actual gas paid by user is:
- The fixed amount of gas prepaid by the system necessary for converting a transaction into a receipt, as
explained in pt. 2, which is Gas(2_428_073_043_512), plus
- The amount of gas burnt by actual function execution, which in the example was Gas(3_193_528_175_642) and
is shown with the ReceiptID of 58oawu7SbU16tAfGHmDLHz6Q61Q3RkGzCsovX5X1dx2Y

Even more confusion comes from the fact that the value of the excess gas refund, as described in pt. 1 does
not seem to fit any simple calculation. This is because the gas is purchased at the pessimistic price, not 
the nominal gas price at the time of execution. The price calculation attempts to predict the maximum number
of cross-contract calls that may be issued with the given amount of attached gas. Then, taking into account
the maximum allowed block-to-block gas price increase, the ultimate gas price is computed. This is the price
at which the attached gas gets purchased. The pseudo-code showing the calculation, as provided in:

https://github.com/near/NEPs/issues/67#:~:text=%40pmnoxx-,sender,-doesn%27t%20know%20the

is:

max_num_contract_calls := prepaid_gas / contract_call_fees
avg_worst_case_gas_cost := (1-1.01^max_num_contract_calls)/(1-1.01)/max_num_contract_calls
cost_in_tokens := avg_worst_case_gas_cost * prepaid_gas

So the gas is purchased at higher than nominal a price. This explains the greater-than-expected NEAR amount
being refunded.

TODO: Why we can attach less gas than the amount burnt by function call execution, as in this example:
testnet ReceiptID: 58oawu7SbU16tAfGHmDLHz6Q61Q3RkGzCsovX5X1dx2Y


Interesting gas-discussing insider topic worth mentioning are:
https://github.com/near/NEPs/issues/67
https://github.com/near/near-explorer/issues/904
https://github.com/near/nearcore/issues/6352
