#!/bin/sh
TARGET=`ps wwwaux | egrep sighandle | awk 'NR==1{print $2}'`
for COUNT in `seq 1000`
do
    kill -INT $TARGET
done
