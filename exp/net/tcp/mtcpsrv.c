#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <semaphore.h>
#include <pthread.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include "../service.h"

#define FACTORY_NUM 2
#define CONSUME_NUM 3
#define BUF_SIZE 10

typedef struct
{
    int head;
    int end;
    int buf[BUF_SIZE];
    sem_t REQ_SEM;
    sem_t AVA_SEM;
    pthread_mutex_t MUTEX_BUFF;
} SHARE_BUF;

typedef struct
{
    int serv_fd;
    SHARE_BUF * p_PIP;
} FACT_PARAM;

int init_socket(struct sockaddr_in *);
int init_thread(pthread_t * arr, void *(*func)(void *), int num, void * param);
void * f_factory(void * p_param);
void * f_consume(void * p_PIP);
int ring_buf_pop(SHARE_BUF * p_PIP);
int ring_buf_push(int ins, SHARE_BUF * p_PIP);
void process_request(int client_fd);
void wait_pthread(pthread_t * parr, int len);

int main()
{
    //init tcp
    struct sockaddr_in serv_addr;
    int serv_fd = init_socket(&serv_addr);

    //init ring buf
    SHARE_BUF PIP;
    PIP.end = 0;
    PIP.head = 0;

    pthread_t factory[FACTORY_NUM], consume[CONSUME_NUM];

    //init sem
    if (sem_init(&(PIP.REQ_SEM), 0, 0) == -1)
    {
        perror("sem_init");
        close(serv_fd);
        return -1;
    }
    if (sem_init(&(PIP.AVA_SEM), 0, BUF_SIZE) == -1)
    {
        perror("sem_init");
        close(serv_fd);
        return -1;
    }

    //init mutex
    if (pthread_mutex_init(&(PIP.MUTEX_BUFF), NULL) != 0)
    {
        perror("pthread_mutex_init");
        close(serv_fd);
        return -1;
    }
    //init thread
    FACT_PARAM fact_param;
    fact_param.serv_fd = serv_fd;
    fact_param.p_PIP = &PIP;
    int rs_factory = init_thread(factory, f_factory, FACTORY_NUM, (void *)(&fact_param));
    int rs_consume = init_thread(consume, f_consume, CONSUME_NUM, (void *)(&PIP));
    if (rs_factory == 0 && rs_consume == 0)
    {
        wait_pthread(factory, FACTORY_NUM);
        wait_pthread(consume, CONSUME_NUM);
    }
    close(serv_fd);
}

void wait_pthread(pthread_t * parr, int len)
{
    int i;
    for (i = 0; i < len; i++)
    {
        pthread_join(parr[i], NULL);
    }
}

int init_socket(struct sockaddr_in * p_serv_addr )
{
    int serv_fd;
    if ((serv_fd = socket(PF_INET, SOCK_STREAM, 0)) < 0)
    {
        perror("socket");
        exit(-1);
    }
    memset(p_serv_addr, 0, sizeof(*p_serv_addr));
    in_addr_t tcp_srv_addr = inet_addr(TCP_SRV_ADDR);
    if (tcp_srv_addr <= 0)
    {
        perror("inet_addr");
        close(serv_fd);
        exit(-1);
    }
    p_serv_addr->sin_addr.s_addr = tcp_srv_addr;
    p_serv_addr->sin_port = htons(TCP_SRV_PORT);
    p_serv_addr->sin_family = AF_INET;

    if (bind(serv_fd, (struct sockaddr *)p_serv_addr, sizeof(*p_serv_addr)) < 0)
    {
        perror("bind");
        close(serv_fd);
        exit(-1);
    }
    printf("bind success.\n");

    if (listen(serv_fd, 20) < 0)
    {
        perror("listen");
        close(serv_fd);
        exit(-1);
    }
    printf("listen success.\n");

    return serv_fd;
}

int init_thread(pthread_t * arr, void * (*func)(void *), int num, void * param)
{
    int i, rc;
    for (i = 0; i < num; i++)
    {
        rc = pthread_create(&arr[i], NULL, func, param);
        if (rc)
        {
            perror("pthread_create");
            return -1;
        }
    }
    return 0;
}

void * f_factory(void * p_param)
{
    FACT_PARAM * p_fact_param = (FACT_PARAM *)p_param;
    int serv_fd = p_fact_param->serv_fd;
    SHARE_BUF * p_PIP = p_fact_param->p_PIP;
    struct sockaddr_in client_addr;
    int client_fd;
    socklen_t len = sizeof(client_addr);

    printf("create a factory thread.\n");

    int flag = 1;
    while(flag)
    {
        if ((client_fd = accept(serv_fd, (struct sockaddr *) &client_addr, &len)) < 0)
        {
            perror("accept");
            close(serv_fd);
            exit(-1);
        }
        if (ring_buf_push(client_fd, p_PIP))
        {
            perror("f_factory");
            close(serv_fd);
            exit(-1);
        }
    }
}

int ring_buf_push(int ins, SHARE_BUF * p_PIP)
{
    sem_wait(&(p_PIP->AVA_SEM));
    if (!pthread_mutex_lock(&(p_PIP->MUTEX_BUFF)))
    {
        p_PIP->buf[p_PIP->end] = ins;
        p_PIP->end = ((p_PIP->end)+1)%10;
        printf("end %d\n", p_PIP->end);
        if(!pthread_mutex_unlock(&(p_PIP->MUTEX_BUFF)))
        {
            sem_post(&(p_PIP->REQ_SEM));
        }
        else
        {
            perror("pthread_mutex_unlock");
            return -1;
        }
    }
    return 0;
}

void * f_consume(void * p_PIP)
{
    SHARE_BUF * ring_buf_p = (SHARE_BUF *)p_PIP;
    int flag = 1;
    printf("create a consume thread.\n");
    while(flag)
    {
        int client_fd = ring_buf_pop(ring_buf_p);
        process_request(client_fd);
        close(client_fd);
    }
}

int ring_buf_pop(SHARE_BUF * p_PIP)
{
    int ins;
    sem_wait(&(p_PIP->REQ_SEM));
    if (!pthread_mutex_lock(&(p_PIP->MUTEX_BUFF)))
    {
        ins = p_PIP->buf[p_PIP->head];
        p_PIP->head = ((p_PIP->head)+1)%10;
        if(!pthread_mutex_unlock(&(p_PIP->MUTEX_BUFF)))
        {
            sem_post(&(p_PIP->AVA_SEM));
        }
        else
        {
            perror("pthread_mutex_unlock");
            exit(1);
        }
    }

    return ins;
}

void process_request(int client_fd)
{
    char recv_buffer[REQMAXLEN + 1], send_buffer[REQMAXLEN];
    ssize_t len = read(client_fd, recv_buffer, sizeof(recv_buffer));
    if (len < 0)
    {
        perror("read");
        close(client_fd);
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
        close(client_fd);
        exit(-1);
    }
}
