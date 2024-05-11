# vodka
CLI password manager (wip)

## Commands
`vodka setup`

`vodka add <name> --password <password>` add a new password

`vodka copy <name>` copy a password to clipboard

`vodka change-master` change the master password

`vodka export` export all passwords to a csv file (Warning: will be unencrypted so delete the csv once you're done with it)

`vodka import` import passwords from a csv file. overwrites existing passwords

`vodka help`

## Todo
- searching for entries (table of results)
- editing/deleting individual entries (XD)
- GUI?

## Info
Argon2 + AES-256
