#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>

#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#include "service.h"

void send_to_svr(int, const char *);

int main(){
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if(sock == -1){
        perror("socket");
        exit(1);
    }
    struct sockaddr_in svraddr;
    bzero(&svraddr, sizeof(svraddr));
    svraddr.sin_family = AF_INET;
    in_addr_t s_addr = inet_addr(TCP_SRV_ADDR);
    if(s_addr == -1){
        perror("inet_addr");
        exit(1);
    }
    svraddr.sin_addr.s_addr = s_addr;
    svraddr.sin_port = htons(TCP_SRV_PORT);
    int err_code = connect(sock, (struct sockaddr *)&svraddr, sizeof(svraddr));
    if(err_code == -1){
        perror("connect");
        exit(1);
    }
    char sendbuf[REQMAXLEN+1];
    strcpy(sendbuf, "login\0");
    send_to_svr(sock, sendbuf);
    strcpy(sendbuf, "err cmd\0");
    send_to_svr(sock, sendbuf);
    strcpy(sendbuf, "logout\0");
    send_to_svr(sock, sendbuf);
    close(sock);
}

void send_to_svr(int sockfd, const char * sendbuf){
    char recvbuf[RESMAXLEN+1];
    size_t sendbuf_len = strlen(sendbuf);
    ssize_t length = write(sockfd, sendbuf, sendbuf_len);
    if(length != sendbuf_len){
        perror("write");
        close(sockfd);
        exit(1);
    }
    printf("send to server: %s\n", sendbuf);
    ssize_t rec_len = read(sockfd, recvbuf, RESMAXLEN);
    if(rec_len == -1){
        perror("read");
        close(sockfd);
        exit(1);
    }
    recvbuf[rec_len] = '\0';
    printf("recv from server: [%d] %s\n", rec_len, recvbuf);
}
