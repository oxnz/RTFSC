## 周末代码 hack

### Repo Setup

#### Update Repo

```shell
git remote add upstream https://github.com/oxnz/RTFSC.git
git fetch upstream
git merge upstream/master
```

### 2016-11-16

#### tasks

* basic tcp server and client
	* service.h [provided]
	* tcpsrv.c [server]
	* tcpcli.c [client]
* multi-thread producer-consumer
	* 2 producer
	* 3 consumer
* multi-thread tcp server

## 笔记

### socket 编程

1.sem的使用，线程间通信。
2.mutex的使用，保护资源。
3.pthread 的使用。

1.client
* 创建socket 
 1. domin的问题，
 2. type IF_INET6 
 3. 协议 可以程序内部自己判断
* connect 连接目标地址
 1. 协议
 2. port addr
 3. socket_in 需要值零
 4. connect 在server端的accept之前。
* fd
* close

2.server
* socket
* bind 地址绑定
 1. 协议
 2. port addr（0.0.0.0 本地测试可以使用）,ip 地址注意转换。
 3. socket_in 需要值零。
 4. inet_addr 谨慎使用，不会返回错误。
 5. 端口0~1024需要root权限才能监听。
 6. bind 有重用选项.
* listen
 1. fd
 2. backlog 内核的队列。等待accept。此时三次握手已经完成。（未完成三次握手的队列可以在内核里面设置队列大小）
* accept
 1. 去backlog的队列里面取值。
 2. 会返回一个描述符，有限制（ulimit -n）。可以设置变大（ulimit -n XXX）
* fd
 1. read/write 错误处理
    连接出问题，返回负值，
    连接关闭，返回0。
    返回正值，数据已经发出去了。
    返回正值但是不等于预期值，需要重试。
    write 返回成功之后，不能确定已经成功发送给对方。
* close
 1. close两端都关闭。
 2. shutdown可以指定一段关闭。

### 多线程
* 注意死锁
* 注意线程安全
 1. 线程共享数据，加锁（mutex）。

### 多线程网络编程

### C 语言

* 标准库
 1. 分清楚c语言标准库和unix的库
 2. posix 标准（操作系统）
 3. C C89 、 C99 、 ANSI 、C11
 4. C语言是没有异常的。
 5. C语言注意指针的使用。（函数指针、指针的使用不要超过三级）
 6. C语言是一个静态强类型语言。
 7. 变量的scope，变量的声明和定义区别。
 8. 头文件里面不能定义，只能有声明。
 9. static的用法。
* 函数
 1. 头文件最好不要定义，inline 和 宏。
 2. c文件是编译单元。
 3. static函数。
* 编译过程
 1. 预处理。宏替换，宏展开
 2. 编译。 c->o 里面包含符号表和地址信息。不会检测符号表是否正确。
 3. 连接。 把多个o文件，连接为可执行文件。入口函数是可以指定的。符号表的检测和类型检测。优化等级，不优化，加调试信息-g选项，把编译地址转换为符号。连接是把外部库指定进去
 4. 可执行文件。全局变量会设置为0。段错误一般是引用了错误的指针。
 5. 段的属性，只读数据段（代码段）。地址随机化、栈的检查会有性能损失，可以通过编译器指定。
 6. ulimit -c XX(大小)。 gdb 程序名 core文件（core.进程号）异常结束一般都会有core。
 7. 程序里面尽量打日志，不要printf ，终端是很慢的，非常耗时。
 8. 写日志，多线程记得上锁。把日志串行化。
