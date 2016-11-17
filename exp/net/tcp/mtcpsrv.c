#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <semaphore.h>

#include "../service.h"

sem_t SEM_FULL;
sem_t SEM_EMPTY;

#define QUEUE_SIZE 10

struct RingBuf{
    pthread_mutex_t lock;
    int * clientfd_q;
    int q_head;
    int q_tail;
};

struct RingBuf CONN_BUF;

void * producer(void *);
void * consumer();
void process_request(int);

int main(){
    sem_init(&SEM_EMPTY, 0, QUEUE_SIZE);
    sem_init(&SEM_FULL, 0, 0);
    pthread_mutex_init(&(CONN_BUF.lock), NULL);
    CONN_BUF.clientfd_q = (int *)malloc(QUEUE_SIZE*sizeof(int));
    CONN_BUF.q_head = 0;
    CONN_BUF.q_tail = 0;

    struct sockaddr_in serv_addr;
    int serv_fd, client_fd;
    socklen_t len = sizeof(serv_addr);
    if ((serv_fd = socket(PF_INET, SOCK_STREAM, 0)) < 0)
    {
        perror("socket");
        return -1;
    }
    memset(&serv_addr, 0, sizeof(serv_addr));
    in_addr_t tcp_srv_addr = inet_addr(TCP_SRV_ADDR);
    if (tcp_srv_addr <= 0)
    {
        perror("inet_addr");
        return -1;
    }
    serv_addr.sin_addr.s_addr = tcp_srv_addr;
    serv_addr.sin_port = htons(TCP_SRV_PORT);
    serv_addr.sin_family = AF_INET;
    if (bind(serv_fd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0)
    {
        perror("bind");
        return -1;
    }
    printf("bind success.\n");

    if (listen(serv_fd, 20) < 0)
    {
        perror("listen");
        return -1;
    }
    printf("listen success.\n");
    pthread_t producers[2], consumers[3];
    int rc;
    for(int i=0; i<3; i++){
    	if((rc = pthread_create(&consumers[i], NULL, (void *)consumer, NULL))){
            perror("pthread_create");
            exit(1);
    	}
    }
    for(int i=0; i<2; i++){
    	if((rc = pthread_create(&producers[i], NULL, (void *)producer, (void*)&serv_fd))){
            perror("pthread_create");
            exit(1);
    	}
    }
    for(int i=0; i<2; i++){
        pthread_join(producers[i], (void**)0);
    }
    for(int i=0; i<3; i++){
        pthread_join(consumers[i], (void**)0);
    }
}

void * producer(void * serv_fd){
    int client_fd;
    struct sockaddr_in client_addr;
    socklen_t len = sizeof(client_addr);
    int retcode, islocked, isunlocked;
    while(1){
        if ((client_fd = accept(*(int*)serv_fd, (struct sockaddr *)&client_addr, &len)) < 0){
            perror("accept");
            pthread_exit((void*)1);
        }
        retcode = sem_wait(&SEM_EMPTY);
        if(retcode != 0){
	    perror("sem_wait");
	    pthread_exit((void*)1);
        }
        islocked = pthread_mutex_lock(&CONN_BUF.lock);
        if(islocked != 0){
	    perror("pthread_mutex_lock");
	    pthread_exit((void*)1);
        }
        CONN_BUF.clientfd_q[CONN_BUF.q_tail] = client_fd;
        CONN_BUF.q_tail = ++CONN_BUF.q_tail % QUEUE_SIZE;
        isunlocked = pthread_mutex_unlock(&CONN_BUF.lock);
        if(isunlocked != 0){
	    perror("pthread_mutex_unlock");
	    pthread_exit((void*)1);
        }
        retcode = sem_post(&SEM_FULL);
        if(retcode != 0){
	    perror("sem_post");
	    pthread_exit((void*)1);
        }
        
    }
    return (void*)0;
}

void * consumer(){
    int retcode, islocked, isunlocked;
    int client_fd;
    while(1){
        retcode = sem_wait(&SEM_FULL);
        if(retcode != 0){
	    perror("sem_wait");
	    pthread_exit((void*)1);
        }
        islocked = pthread_mutex_lock(&CONN_BUF.lock);
        if(islocked != 0){
	    perror("pthread_mutex_lock");
	    pthread_exit((void*)1);
        }
        client_fd = CONN_BUF.clientfd_q[CONN_BUF.q_head];
	CONN_BUF.q_head = ++CONN_BUF.q_head % QUEUE_SIZE;
        isunlocked = pthread_mutex_unlock(&CONN_BUF.lock);
        if(isunlocked != 0){
	    perror("pthread_mutex_unlock");
	    pthread_exit((void*)1);
        }
        retcode = sem_post(&SEM_EMPTY);
        if(retcode != 0){
	    perror("sem_post");
	    pthread_exit((void*)1);
        }
        process_request(client_fd);
        close(client_fd);
    }
    return (void*)0;
}

void process_request(int client_fd)
{
    char recv_buffer[REQMAXLEN + 1], send_buffer[REQMAXLEN];
    ssize_t len = read(client_fd, recv_buffer, sizeof(recv_buffer));
    if (len < 0)
    {
        perror("read");
        exit(-1);
    }
    else
    {
        recv_buffer[len] = '\0';
    }
    printf("recv data: %s.\nrecv len : %zd\n", recv_buffer, len);
    if (strcmp(recv_buffer, "login") == 0)
    {
        strcpy(send_buffer, "welcome");
    }
    else if (strcmp(recv_buffer, "logout") == 0)
    {
        strcpy(send_buffer, "bye");
    }
    else
    {
        strcpy(send_buffer, "error");
    }
    ssize_t wnum = write(client_fd, send_buffer, strlen(send_buffer));
    if (wnum != strlen(send_buffer))
    {
        perror("write");
        exit(-1);
    }
}

