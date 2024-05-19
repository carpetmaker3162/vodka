# vodka
CLI password manager (wip)

## Commands
```vodka setup```

```vodka add <name> --password <password> --comment <comment>```

> add a new password. Pass `--random` to generate a random password 24 chars long. Guarantees at least 1 capital letter, number, and special character

```vodka copy <fullname>```

> copy a password to clipboard

```vodka search <fullname>```

> search for an entry, displays a cli table of results

```vodka list```

> list all existing entries

```vodka export <csv-file>```

> export all passwords to a csv file (Warning: will be unencrypted so delete the csv once you're done with it)

```vodka import <csv-file>```

> import passwords from a csv file. overwrites existing passwords

```vodka change-master```

> change the master password

```vodka help```

## Todo
- editing/deleting individual entries (XD)
- operations by ID
- GUI?

## Info
Argon2 + AES-256
