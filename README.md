# RTFSC

Read The Forking Source Code

# FreeBSD

## Env setup

1. Enable su

	pw groupmod wheel -m <username>

or

	pw user mod <username> -g wheel

or edit /etc/group

	wheel:*:0:root,username

## Setup git

	pkg install git-lite

## Setup vim

	pkg install vim-lite

## Hier

	man hier

FreeBSD的kernel构成文件在/usr/src/sys的目录下面。

```
/usr/src/sys
  |
  | --   compile     //编译核心的目录。
  | --   conf        //configure的目录。
  | --   ddb         //核心调试的sounre code的目录。
  | --   dev         //一部分的drivers的source code的目录。
  | --   gnu         //浮点运算的仿真以及ex2fs文件系统的source code目录。
  | --   i386        //依赖于pc/at机器的目录，以下介绍它的字目录。
           | --  apm     //suspend一些节电程序。
           | --  boot    //不是kernel本身的东西，只是一些怎么开机到读入kernel的boot program source code。
           | --  conf    //config的一些依赖data。
           | --  isa     //isa bus的驱动程序类的source code。
           | --  eisa    //eisa bus的驱动程序类的source code。
           | --  include //对pc/at的一些include files
           | --  i386    //对pc/at的一些核心code
           | --  ibcs2   //使各类的os的执行文件在freebsd上执行的code
  | --   isofs/cd9660    //cd-rom在unix文件系统上操作的的有关code
  | --   kern     //核心code
  | --   libkern  //核心库的source code
  | --   miscfs   //实现unix文件系统的code
  | --   msdosfs  //在unix上操作ms-dos文件系统的有关code
  | --   net      //实现network功能的基本部分code
  | --   netatalk //实现appletalk network功能code
  | --   netinet  //实现internet network功能的code
  | --   netipx   //实现ipx功能的code
  | --   netns    //实现ns network的code
  | --   netkey   //实现网络加密部分的功能的code
  | --   nfs      //实现nfs服务
  | --   pc98     //对于pc98的支持
  | --   pccard   //对pcmcia的支持
  | --   pci      //对pci bus的驱动程序的source code
  | --   scsi     //对cd-rom，hard disk,tape 等的scsi驱动程序的source code
  | --   sys      //独立于机器体系结构的一部分code
  | --   ufs      //unix file system 的支持code
  | --   vm       //虚拟内存管理的部分
```
