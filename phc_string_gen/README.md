# pch_string_gen

```bash
go run phc_string_gen.go --help
Usage of phc_string_gen:
  -l uint
        [l]ength (default 32)
  -m uint
        [m]emory (default 19456)
  -p string
        [p]assword
        Automatically generated if not entered
  -s string
        [s]alt
        Automatically generated if not entered
  -t uint
        [t]ime (default 2)
  -th uint
        [th]reads (default 1)
```

```bash
go run phc_string_gen.go       
Password   : HGwvjxamsk5XU6gp0uEOlf2WH/QzXDAR
Salt       : DP2TYJ3S
PHC string : $argon2id$v=19$m=19456,t=2,p=1$RFAyVFlKM1M$teuhWoIu+5YaMFaujWgdUmmnKP0NL1vklacOUwI3WL8
```