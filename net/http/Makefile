LDFLAGS	:= -lpthread
OJBS	:= \
   	main.o \
	server.o \
	server_socket.o \
	ring_buffer.o \
	process_request.o
mhttpd: $(OJBS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

process_request.o: process_request.c process_request.h

main.o: main.c process_request.h ring_buffer.h

ring_buffer.o: ring_buffer.c ring_buffer.h

server.o: server.c server.h

clean:
	$(RM) $(OJBS)

.phony: clean
