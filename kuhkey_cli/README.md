# kuhkey cli

Command line client for KuhKey.

## Commands

### `gen`: generate a password

```sh
cargo r -- gen                    # 16 chars, all char types
cargo r -- gen --len 32 -u -n     # 32 chars, only upper + numbers
cargo r -- gen -c                 # copy to clipboard instead of printing
```

| flag | what it does |
|---|---|
| `--len <n>` | password length (default 16) |
| `-u`, `--upper` | include uppercase |
| `-l`, `--lower` | include lowercase |
| `-n`, `--numbers` | include numbers |
| `-s`, `--symbols` | include symbols |
| `--no-alikes` | leave out chars that look alike (`0Oo1Il`) |
| `-c`, `--copy` | put the password into the clipboard |

If you don't pass `-u`, `-l`, `-n` or `-s`, all four are used.

### `gen-user`: generate a username

```sh
cargo r -- gen-user                         # CafeteriaPias
cargo r -- gen-user -n                      # FerulaeToward7222
cargo r -- gen-user -n -f --num-len 2 -s    # 81Liches_Colleges
cargo r -- gen-user -w 3 --case lower       # three lowercase words
```

| flag | what it does |
|---|---|
| `--len <n>` | max length of the whole username (default 20) |
| `-w`, `--words <n>` | how many words (default 2) |
| `--case <case>` | `lower`, `capitalized` or `upper` (default `capitalized`) |
| `-s`, `--separator` | put a `_` between the words |
| `-n`, `--numbers` | add random numbers (at the end by default) |
| `-f`, `--front` | put the numbers in front instead |
| `--num-len <n>` | how many numbers (default 4) |
| `-c`, `--copy` | put the username into the clipboard |

