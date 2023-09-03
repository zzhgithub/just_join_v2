# Just Join (来噻)

此仓库是 https://github.com/zzhgithub/just_join 的网络化重构施工中。敬请期待。


> ~~现在Windows i5 cpu 机器都很卡。在优化中~~

# 启动方法
For Server
```shell
cargo run --release --bin server
# or
cargo run --release --no-default-features --features headless --bin server
```


For Client
```shell
cargo run --release --bin client
```

# 控制
- W 前
- S 后
- A 左
- D 右
- Space 跳
- ESC 光标
- T 切换视角

# 功能列表

- [x] 加载无限的地图
  - [ ] 更多的群系支撑
- [x] 加载不同材质
  - [ ] 更多的方块支撑 
  - [ ] 支撑游戏内的其他实体
- [x] 加载水体
  - [ ] 更好的水体 
- [x] 网络多人游戏(基础功能)
  - [x] 展示用户名
  - [x] 保存用户位置等信息
  - [ ] 用户名密码登录
  - [ ] 多人的聊天系统
  - [ ] 范围语音
- [x] 加载人物 
  - [ ] 更好的人物模型
  - [ ] 人物的动作和表情系统
  - [ ] 人装备系统
  - [ ] 背包
  - [ ] 生存模式
- [ ] UI
  - [x] toolbar
  - [ ] 背包
  - [ ] 人物展示
  - [ ] 地图
- [x] 合成系统
  - [ ] 增强交互（显示当前背包）
  - [ ] 支撑合成公式搜索
  - [ ] 添加更多游戏内的公式
- [ ] 任务系统
- [ ] 领地系统
- [ ] 风场
- [ ] 游戏mod化
  - [ ] 家具mod
- [ ] 国际化支撑
- [ ] 工具
  - [ ] tp命令
- [ ] 优化
  - [ ] 网络优化项

# 游戏截图
![a](pic/a.png)
![b](pic/b.png)
<!-- ![c](pic/c.png) -->
![d](pic/d.png)
![e](pic/e.png)


# 游戏理念