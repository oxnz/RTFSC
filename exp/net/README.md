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
* client
  	* socket
  	* connect
  		return before accept
  	* write
  	* read
  
* server
	* socket
		domain(AF_INET, PF_INET, AF_INET6)
     		type(SOCK_STREAM, RAW, SOCK_DGRAM)
		protocol(0 determined by domain & type, TCP, )
  	* bind
		socket
		sockaddr
			bzero set 0
			sin_family(AF_INET)
			port
				> 1024
				htons
			address(IN_ADDR_ANY int type)
				init_addr donot return error
				init_pt..
	* set_socket_option
		reuse port
  	* listen
		backlog(three hand shaking completed queue)
		proc/sys/cole/net/max_tcp_sink_.. (on hand shaking queue)
 	* accept 
		get from backlog queue return client sock_fd
		`ulimit -n` return current tcp max fd 
		`ulimit -n newsize` set max tcp connection counts
  	* read
		error handler
		return negative if failed
		return 0 if connection closed
		< user_buf if kernel buf is full
  	* write
		return does not mean data send success
  	* close
	* shutdown
		close specified directions of connection(write or read)
  
  * std
  	stdio
	stdlib
	accoss platform support
	
	POSIX 2003 2007
	C c89 c99 c11 ansi
 * no exception
 * pointer usage:
 	* variable
	* array
	* function
	no larger than 3-level
	
 * C strong type language, static type language
 
 * variable 
 	* claim
		can not use claim to allocate memory
 	* definition
		can not in header file unless use macro or extern
	* usage
	
 * function
 	* claim
		static related [offline]
	* definition
		can not in header file unless inline or macro
	* usage
	
 * compile
 	* pre process
		macro replace
	* compile
		.c --> .o(linux)/.obj(windows)
		generate simble table
	* link
		simble table linkage
		entry point
		option: optimize level, `-g` for debug info,
		third-library linkage
		segment merge: text, data, bss(0 byte)
		segment property: text -- read only
				  data -- read/write
		aslr
		stack check: canery
		gdb usage:
		`ulimit -c size`
		gdb program core.pid(current)
 
 * standarize
 	* log replace printf
	* lock when write log
	* thread safe: shared data, 
	* thread local: register, stack, signal process
