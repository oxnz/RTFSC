# Micro Http Server (mhttpd)

## Arch

### Listener

* listener -> accept -> connection

### Worker

* connection -> epoll/kqueue -> read -> request(s)
* request -> handler -> response
* response -> aio -> write

```cpp
class worker {
		response process(request& req) {
		}
}
```

### RPC

codec

* read -> decode -> request
* response -> encode -> write
