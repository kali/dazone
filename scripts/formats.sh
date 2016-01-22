#!/bin/sh

SET=1node


for comp in "" gz snz
do
    for enc in csv bincode cbor rmp cap
    do
        format=$enc-$comp
        echo
        echo "################# $format ###################"
        echo
        ./target/release/pack -s $SET uservisits $format
        du -hs data/$format/$SET/uservisits
        du -s data/$format/$SET/uservisits
        /usr/local/bin/purge
        ./target/release/query2 -s $SET -i $format
        rm -rf data/$format/$SET/uservisits
    done
done
