/*
 * Filename:	dns.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-09-21 13:03:50 CST]
 * Last-update:	2015-09-21 13:03:50 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License: 
 * Copyright (c) 2013 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 *
 *
 * RFC 1035
 */

#include <sys/types.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <netinet/in.h>

#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <unistd.h>

#define REQMAXLEN 1024
#define RSPMAXLEN 1024

typedef struct sockaddr * SA;

struct req_header {
	uint16_t ID;
			uint8_t RD:1;
			uint8_t TC:1;
			uint8_t AA:1;
			uint8_t Opcode:4;
			uint8_t QR:1;

			uint8_t RCODE:4;
			uint8_t Z:3;
			uint8_t RA:1;
	uint16_t QDCOUNT;
	uint16_t ANCOUNT;
	uint16_t NSCOUNT;
	uint16_t ARCOUNT;
};

static void pheader(uint8_t * req, int len) {
	struct req_header *ph = (struct req_header *)req;
	uint8_t *p;
	uint8_t cnt;

	printf("len(%d)\n", len);
	printf("[ID:%d]\n", ntohs(ph->ID));
	printf("[QR:%d]", ph->QR);
	printf("[Opcode:%d]", ph->Opcode);
	printf("[AA:%d]", ph->AA);
	printf("[TC:%d]", ph->TC);
	printf("[RD:%d]", ph->RD);
	printf("[RA:%d]", ph->RA);
	printf("[Z:%d]", ph->Z);
	printf("[RCODE:%d]", ph->RCODE);
	putchar('\n');
	printf("[QDCOUNT:%d]\n[ANCOUNT:%d]\n[NSCOUNT:%d]\n[ARCOUNT:%d]\n",
			ntohs(ph->QDCOUNT),
			ntohs(ph->ANCOUNT),
			ntohs(ph->NSCOUNT),
			ntohs(ph->ARCOUNT));
	len = sizeof(struct req_header);
	p = (uint8_t *)req + len;
	for (cnt = *p; cnt; cnt = *p) {
		printf("[%hu]:", cnt);
		while (cnt--)
			putchar(*++p);
		++p;
		putchar(' ');
	}
	putchar('\n');
	printf("[type:%d]", *(uint16_t *)++p);
	printf("[class:%d]\n", *(uint16_t *)++p);
}

static int process(uint8_t *req, int len, uint8_t *rsp) {
	struct req_header *ph = (struct req_header *)rsp;
	uint8_t *p = rsp + sizeof(struct req_header);

	memcpy(rsp, req, len);
	ph->QR = 1;
	ph->AA = 1;
	ph->RA = 1;
	ph->Z = 0;
	ph->ANCOUNT = htons(1);
	ph->ARCOUNT = 0;
	pheader(req, len);
	pheader(rsp, len);

	while (*p) {
		printf("-->%d\n", *p);
		p += *p + 1;
	}
	++p;
	//	*(uint32_t *)p = 128;
	p += sizeof(uint32_t);
	*p = 1;
	++p;
	*p = 'x';
	++p;
	*p = 3;
	*++p = 'c';
	*++p = 'o';
	*++p = 'm';
	*++p = 0;
	++p;
	*(uint16_t *)p = htons(1);
	p += 2;
	*(uint16_t *)p = htons(1);
	p += 2;
	*(uint32_t *)p = htonl(127);
	p += 4;
	*(uint16_t *)p = htons(4);
	p += 2;
	*(uint32_t *)p = htonl(1234567);
	p += 4;
	return p - rsp;
}

static void phex(uint8_t *p, int len) {
	int i;
	uint8_t buf[18];

	buf[16] = '\n';
	buf[17] = '\0';
	for (i = 0; i < len; ++i) {
		if (i && i % 16 == 0)
			printf("  %s", buf);
		printf("[%02x]", p[i]);
		if (isalnum(p[i]))
			buf[i%16] = p[i];
		else
			buf[i%16] = '.';
	}
	putchar('\n');
}

int main(int argc, char *argv[]) {
	uint8_t req[REQMAXLEN], rsp[RSPMAXLEN];
	int sockfd, n;
	struct sockaddr_in server, client;
	socklen_t len = sizeof(client);

	server.sin_family = PF_INET;
	server.sin_port = htons(8000);
	server.sin_addr.s_addr = htonl(INADDR_ANY);

	if ((sockfd = socket(PF_INET, SOCK_DGRAM, IPPROTO_UDP)) < 0)
		err(1, "socket");
	if (bind(sockfd, (SA) &server, sizeof(server)) < 0)
		err(1, "bind");

	printf("server started on port: 8000\n");

	for (;;) {
		if ((n = recvfrom(sockfd, req, REQMAXLEN, 0, (SA) &client, &len)) < 0) {
			err(1, "recvfrom");
		}
		printf("request from [%s:%d]:\n", inet_ntoa(client.sin_addr), ntohs(client.sin_port));
		phex(req, n);
		n = process(req, n, rsp);
		phex(rsp, n);
		printf("n = %d\n", n);
		if ((n = sendto(sockfd, rsp, n, 0, (SA) &client, len)) != n)
			err(1, "sendto");
	}

	exit(0);
}
