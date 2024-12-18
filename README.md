# Santa's Amex
![Santa](assets/santa.jpg)

This project is designed to keep Santa's expenses in check. Due to unforeseen circumstances,
the North Pole has been subjected to a 25% tariff on all toys created by elves in Santa's workshop that
are being shipped to children in the United States. To save on cost, he will purchase all American 
children's toys from department stores within the US. 

Each child has a total amount that can be spent on them based on their deeds. Good deeds have 
been recorded as deposits, while bad deeds are withdrawals from their credit limit. Santa hired
a legal team to assist him with his taxes, but is also using their services to allow children
to dispute claims of bad deeds that may be lowering their available funds for Christmas toys.

### Getting Started
There is a set of 50 transactions that can be used to run the program as follows:
- `cd accountant`
- `cargo install`
- `cargo build`
- `cd assets`
- `cargo run -- transactions.csv > accounts.csv`


### Generated Files
An accounts.csv file will be generated with the following headers:
client, available, held, total, locked

A transactions.log file which tracks all transactions as well as 
logs and errors encountered in the program


### Rules of Account
* All new clients are initialized with a starting value of $0
* Withdrawals that exceed the amount of available funds are rejected. NO OVERDRAFTS!


## Testing
To test the client functions use the following:
`cargo test`

The test-case crate is used to test each potential transaction along with the expected output
in both success and failure cases.