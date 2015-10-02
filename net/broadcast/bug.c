#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <sys/types.h>
#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

int main(void) {
	const char *broadcast_addr = "255.255.255.255";
	in_addr_t addr;
	
	assert(inet_addr(broadcast_addr) == -1);

	char buf[INET_ADDRSTRLEN];

	if (inet_pton(AF_INET, broadcast_addr, &addr) <= 0)
		err(1, "inet_pton");
	assert(addr == addr);
	if (inet_ntop(AF_INET, &addr, buf, INET_ADDRSTRLEN) == NULL)
		err(1, "inet_ntop");
	assert(strcmp(buf, broadcast_addr) == 0);

	exit(0);
}
