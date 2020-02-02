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
    void process() {
      foreach (ev in events) {
      case ev & read: read_request();
      case ev & write: write_response();
      case ev & error: close_connection();
      }
      foreach (req in requests) {
        handler = dispatch(req);
        try {
          handler.process(req);
        } catch (ex) {
          // error handling
        }
      }
    }
    void dispatch(req) {
      foreach(pattern in patterns)
        if (req.path match pattern)
          return handlers[pattern];
        return default_handler;
    }
		response process(request& req) {
		}
}
```

### RPC

codec

* read -> decode -> request
* response -> encode -> write
