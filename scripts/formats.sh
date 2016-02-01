#!/bin/sh

SET=5nodes

for comp in "" lz4 snz
do
    for enc in cap pbuf pcap mcap
    do
        if [ -z "$comp" ]
        then
            format="$enc"
        else
            format="$enc-$comp"
        fi
        echo
        echo "################# $format ###################"
        echo
        ./target/release/pack -s $SET uservisits $format
        du -hs data/$format/$SET/uservisits
        du -s data/$format/$SET/uservisits
        for i in 1 2 3
        do
            if [ `uname` == Darwin ]
            then
                /usr/local/bin/purge
            else
                sync && echo 3 > /proc/sys/vm/drop_caches
            fi
            ./target/release/query2 -s $SET -i $format
        done
        rm -rf data/$format/$SET/uservisits
    done
done
