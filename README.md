# wBAN Total Supply monitoring tool

This tool checks that the wBAN hot and cold wallets have more BAN available than wBAN total supply.

If it were not the case anymore, it sends Reddit DM to administrators.

## Settings

This tool expect the following environment variables:

| Env Name                   | Env Description     | Example               |
|----------------------------|---------------------|-----------------------|
| `BAN_RPC_API`              | Host and port of the BAN RPC API | `10.60.0.70:7072` |
| `BAN_HOT_WALLET`           | Banano address of the hot wallet | `ban_1...` |
| `BAN_COLD_WALLET`          | Banano address of the cold wallet | `ban_1...` |
| `REDDIT_BOT_USERNAME`      | Reddit username sending DM messages | `wban-banano`
| `REDDIT_BOT_PASSWORD`      | Reddit password of the user sending DM | `<my_password> ` |
| `REDDIT_BOT_CLIENT_ID`     | Reddit bot client ID | `<...>` |
| `REDDIT_BOT_CLIENT_SECRET` | Reddit bot client secret | `<...>` |
| `REDDIT_BOT_DM_USERS`      | List (whitespace separated) of users to send Reddit DM to | `user1 user2 user3` | 