# Just Join (来噻)

this rep is rebuild from https://github.com/zzhgithub/just_join. The goal is to make the original project support the online multiplayer game

> ~~Ka Le on windows i5 cpu. optimizing!~~

# Usage
For Server
```shell
cargo run --release --bin server
```


For Client
```shell
cargo run --release --bin client
```

# Controller
- W - forward
- S - backward
- A - left
- D - right
- Space - Jump
- ESC - toggle grab cursor
- T - toggle One/Thrid Person

# Feature List
- [x] Load unlimited maps
  - [ ] support more biomes
- [x] Loading different materials for voxels
  - [ ] More block support 
  - [ ] support more entities in game(like grass or animals)
- [x] Load water
  - [ ] better water displaying
- [x] online multiplayer game(fundationally)
  - [x] display username
  - [ ] sign in with username and password
  - [ ] chat system
  - [ ] range voice
- [x] load character 
  - [ ] better character models
  - [ ] actions and expression face system
  - [ ] Equipment system
  - [ ] knapsack system
  - [ ] Survival Mode
- [ ] UI
  - [ ] toolbar
  - [ ] knapsack system
  - [ ] multiplayer show
  - [ ] on time map
- [ ] composite system
- [ ] task system
- [ ] manor system
- [ ] wind zone
- [ ] support load mod and interface doc
  - [ ] furniture mod
- [ ] i18n
- [ ] Tools
  - [ ] tp commands
- [ ] optimize
  - [ ] net optimize (such as lz4)


# screenshot
![a](pic/a.png)
![b](pic/b.png)
![c](pic/c.png)
