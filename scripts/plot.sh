
M3XL="2016-02-09T1[56].*timely"
M32XL="2016-02-10T.*timely"
C38XL="2016-02-11T.*timely"

function median {
    FILE=$1
    count=`grep '^0 ' $FILE | cut -f 2 -d ' ' | wc -l`
    head -n $(($count/2)) $FILE | cut -f 2 -d ' ' | tail -1
}

function draw_scalability {
    NAME=$1
    RE=$2
    COST=$3
    grep -r "$RE" data/ | grep -v " 0$" | grep "length:  *8" | perl -pe "s/ +/ /g" | cut -f 11,21 -d ' ' > /tmp/a
    grep -r "$RE" data/ | grep -v " 0$" | grep "length: *10" | perl -pe "s/ +/ /g" | cut -f 11,21 -d ' ' > /tmp/b
    grep -r "$RE" data/ | grep -v " 0$" | grep "length: *12" | perl -pe "s/ +/ /g" | cut -f 11,21 -d ' ' > /tmp/c
    med_a=$(median /tmp/a)
    med_b=$(median /tmp/b)
    med_c=$(median /tmp/c)
    echo $NAME median time 2A $med_a
    echo $NAME median time 2B $med_b
    echo $NAME median time 2C $med_c
    cat <<- PLOT | gnuplot
        set terminal pngcairo size 640,320
        set yrange [0:]
        set output "$NAME.png"
        plot "/tmp/a" title "A. X=8", "/tmp/b" title "B. X=10", "/tmp/c" title "C. X=12"
        set yrange [0:]
        max(x,y) = x<y ? y : x;
        set output "$NAME-cost.png"
        plot "/tmp/a" using 1:($COST*max(\$1,1)*\$2) title "A. X=8", \
             "/tmp/b" using 1:($COST*max(\$1,1)*\$2) title "B. X=10", \
             "/tmp/c" using 1:($COST*max(\$1,1)*\$2) title "C. X=12"
        set yrange [0:]
        set output "$NAME-scalability.png"
        plot "/tmp/a" using 1:($med_a/max(\$1,1)/\$2) title "A. X=8", \
             "/tmp/b" using 1:($med_b/max(\$1,1)/\$2) title "B. X=10", \
             "/tmp/c" using 1:($med_c/max(\$1,1)/\$2) title "C. X=12"
PLOT
}

draw_scalability m3xl $M3XL 0.266
draw_scalability m32xl $M32XL 0.532
draw_scalability c38xl $C38XL 1.680
