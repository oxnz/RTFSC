#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>

int main(void) {
	in_addr_t addr = inet_addr("255.255.255.255");
	assert(addr == -1);
	exit(0);
}
