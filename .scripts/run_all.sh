#!/bin/sh

LOG=/var/log/all

touch $LOG

for file in /app/run/*; do
    $file >>$LOG &
done

tail -f $LOG
