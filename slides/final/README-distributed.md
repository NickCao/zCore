对分布式操作系统内核提供接口的网络框架

##接口说明

详见linux-object/src/net/distributed.rs

主要功能接口见下：

- connect()：连接master获取分布式结构中唯一标识id，返回运行结果
- getid()：获取本机在分布式结构中的唯一标识id
- disconnect()：断开与master的连接
- set_block&set_nonblock：设置接受消息为阻塞&非阻塞模式。
-  send(dest_id: usize, data: &[u8]) -> SysResult : 给标识为dest_id的操作系统内核发送data消息，返回运行结果
- recv(source_id: &mut usize, data: &mut [u8]) -> SysResult : 接受一条消息，获得消息来源source_id以及消息内容data，返回运行结果。

## 网络拓扑及环境配置

##### 1 server.py作为master服务器

这是目前zCore支持的拓扑结构，见下图：

![image1](.\images\image1.png)

网卡配置：

```bash
-netdev user,id=net1,guestfwd=tcp:10.0.2.16:1234-tcp:127.0.0.1:1234
-device e1000e,netdev=net1
```

默认master服务器端口为10.0.2.16:1234，该配置会将连接10.0.2.16:1234的包转发到ubuntu 的127.0.0.1:1234端口，供server.py使用。

运行方式：

```bash
python server.py
cargo qemu --arch=riscv64
```

注意，不同zCore的IP和mac配置不应相同，具体而言需要在`xtask/src/build.rs` 中更改qemu参数的`IP` 字段，范围为0~15

##### 2 虚拟以太网VDE

使用VDE工具模拟交换机，实现网络拓扑如下图：

![image2](.\images\image2.png)

VDE工具文档见：https://github.com/virtualsquare/vde-2

网卡配置：

```bash
-netdev vde,id=net1,sock=/tmp/myswitch
-device e1000e,netdev=net1
```

运行方式：

```bash
vde_switch -F -sock /tmp/myswitch
cargo qemu --arch=riscv64
```

注意，不同zCore的IP和mac配置仍然不应相同，而且依然需要指定master的zCore的IP。

这种方式会将zCore的网卡连接到虚拟交换机上，实现真正意义上的以太网通讯，但目前zCore之间的tcp通讯会有Illegal报错，故最终未采用，但经过dump网络包的查验（见“调试方式”），网络拓扑结构实现没问题。

## qemu + VDE环境配置方式

该配置主要用于模拟交换机后将qemu网卡与交换机连接进而实现虚拟以太网的模拟。

1. 安装vde2

```
apt install vde2
```

2. 查看qemu的riscv64架构是否支持vde，运行

```
qemu-system-riscv64 -netdev help
```

如果存在vde选项则支持，否则不支持。

3. 如果不支持，则需要重新安装一个打开--enable-vde编译开关的qemu，见官网https://www.qemu.org/download/，运行时增加编译选项

```
./configure --enable-vde
make
```

注：如果遇到报错`C header 'libvdeplug.h not found'` 需要先安装libvdeplug-dev。

4. 重新运行`qemu-system-riscv64 -netdev help` 发现已经支持vde
5. 开启vde_switch，配置qemu网卡、IP、mac等，连接vde_switch，具体见“网络拓扑及环境配置”。

## 调试方式

参见：https://wiki.qemu.org/Documentation/Networking

可以在网卡配置中增加下述选项以实现将网卡的包dump到“dump.dat”中的功能，之后可以通过wireshark等工具实现网络包的分析。

```
-object filter-dump,id=f1,netdev=net1,file=dump.dat
```

