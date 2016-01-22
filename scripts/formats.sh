#!/bin/sh

SET=5nodes

for format in csv text-deflate csv-snz \
              rmp rmp-gz rmp-snz \
              cap cap-gz cap-snz
do
    echo
    echo "################# $format ###################"
    echo
    if [ $format != 'text-deflate' ]
    then
        ./target/release/pack -s $SET uservisits $format
    fi
    du -hs data/$format/$SET/uservisits
    du -s data/$format/$SET/uservisits
    /usr/local/bin/purge
    ./target/release/query2 -s $SET -i $format
    if [ $format != 'text-deflate' ]
    then
        rm -rf data/$format/$SET/uservisits
    fi
done
